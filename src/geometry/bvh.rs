use std::{alloc::Layout, ops::Range};

use bumpalo::Bump;
use ctor::ctor;

use crate::{
    ray::Ray,
    vector::{Dimension, Vector, Vector3x8},
};

use super::{aabb::Aabb, Hit, Object, ObjectList};

#[derive(Copy, Clone)]
enum Node {
    Leaf { offset: u32, length: u16 },
    Branch { idx: u32 },
}

#[repr(align(64))]
struct Branch {
    aabb_min: Vector3x8,
    aabb_max: Vector3x8,
    children: [Node; 8],
}

pub struct Bvh<L> {
    object_list: L,
    bounding_box: Aabb,
    branches: Box<[Branch]>,
    root: Node,
    max_depth: usize,
}

struct ObjectInfo {
    centroid: Vector,
    bounds: Aabb,
    idx: usize,
}

impl<L: ObjectList<Object = O>, O> Bvh<L> {
    pub fn new(mut object_list: L) -> Self {
        let mut obj_infos: Vec<_> = (0..object_list.len())
            .map(|idx| ObjectInfo {
                centroid: object_list.centroid(idx),
                bounds: object_list.bounding_box(idx),
                idx,
            })
            .collect();

        let mut branches = Vec::new();

        let (root, aabb, max_depth) = build(obj_infos.as_mut_slice(), 0, &mut branches);

        // Reorder objects
        let objects = object_list.objects_mut();
        for i in 0..objects.len() {
            let mut j = obj_infos[i].idx;
            while j < i {
                // everything before i has been swapped, so we need to follow the chain of swaps to
                // find our actual target object
                j = obj_infos[j].idx;
            }
            obj_infos[i].idx = j;
            objects.swap(i, j);
        }

        Self {
            object_list,
            bounding_box: aabb,
            branches: branches.into_boxed_slice(),
            root,
            max_depth,
        }
    }
}

type IntersectionsTest = unsafe fn(Ray, &Vector3x8, &Vector3x8, Range<f32>) -> u8;

#[ctor]
static INTERSECTIONS_TEST: IntersectionsTest = {
    #[allow(unused_mut)]
    let mut func: IntersectionsTest = intersections_generic;

    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx") {
        func = intersections_x86_avx;
    } else if is_x86_feature_detected!("sse") {
        func = intersections_x86_sse;
    }

    func
};

impl<L: ObjectList<Object = O> + Send + Sync, O> Object for Bvh<L> {
    fn hit(&self, ray: Ray, mut t_range: Range<f32>, arena: &Bump) -> Option<Hit> {
        let pending_nodes_cap = self.max_depth * 7 + 1;
        let pending_nodes = arena
            .alloc_layout(Layout::array::<Node>(pending_nodes_cap).unwrap())
            .as_ptr()
            .cast::<Node>();
        unsafe {
            pending_nodes.add(0).write(self.root);
        }
        let mut pending_nodes_len = 1;

        let mut nearest_hit = None;

        loop {
            if pending_nodes_len == 0 {
                break;
            }
            let node = unsafe {
                pending_nodes_len -= 1;
                pending_nodes.add(pending_nodes_len).read()
            };

            match node {
                Node::Leaf { offset, length } => {
                    let start = offset as usize;
                    let end = start + length as usize;

                    for i in start..end {
                        if let Some(hit) = self.object_list.hit(ray, t_range.clone(), i, arena) {
                            t_range.end = hit.t;
                            nearest_hit = Some(hit);
                        }
                    }
                }
                Node::Branch { idx, .. } => {
                    let branch = &self.branches[idx as usize];

                    let mut intersections = unsafe {
                        INTERSECTIONS_TEST(ray, &branch.aabb_min, &branch.aabb_max, t_range.clone())
                    };

                    loop {
                        let idx = intersections.trailing_zeros();
                        if idx == u8::BITS {
                            break;
                        }
                        intersections ^= 1 << idx;
                        let child = branch.children[idx as usize];
                        unsafe {
                            pending_nodes.add(pending_nodes_len).write(child);
                            pending_nodes_len += 1;
                        }
                    }
                }
            }
        }

        nearest_hit
    }

    fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }
}

fn gamma(n: i32) -> f32 {
    (n as f32 * f32::EPSILON) / (1.0 - n as f32 * f32::EPSILON)
}

fn intersections_generic(
    ray: Ray,
    aabb_min: &Vector3x8,
    aabb_max: &Vector3x8,
    t_range: Range<f32>,
) -> u8 {
    let mut intersections = 0;

    let vel_rcp = 1.0 / ray.direction;

    for i in 0..8 {
        let aabb_min = Vector::from(aabb_min.get_vec(i));
        let aabb_max = Vector::from(aabb_max.get_vec(i));

        let t0 = (aabb_min - ray.origin) * vel_rcp;
        let t1 = (aabb_max - ray.origin) * vel_rcp;

        let mut tmin = t_range.start;
        tmin = f32::min(f32::max(t0.x(), tmin), f32::max(t1.x(), tmin));
        tmin = f32::min(f32::max(t0.y(), tmin), f32::max(t1.y(), tmin));
        tmin = f32::min(f32::max(t0.z(), tmin), f32::max(t1.z(), tmin));

        let mut tmax = t_range.end;
        tmax = f32::max(f32::min(t0.x(), tmax), f32::min(t1.x(), tmax));
        tmax = f32::max(f32::min(t0.y(), tmax), f32::min(t1.y(), tmax));
        tmax = f32::max(f32::min(t0.z(), tmax), f32::min(t1.z(), tmax));

        tmax *= 1.0 + 2.0 * gamma(3);

        if tmin <= tmax {
            intersections |= 1 << i;
        }
    }

    intersections
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse")]
unsafe fn intersections_x86_sse(
    ray: Ray,
    aabb_min: &Vector3x8,
    aabb_max: &Vector3x8,
    t_range: Range<f32>,
) -> u8 {
    use std::arch::x86_64::*;

    let vel_rcp = 1.0 / ray.direction;
    let vel_rcp_x = _mm_set1_ps(vel_rcp.x());
    let vel_rcp_y = _mm_set1_ps(vel_rcp.y());
    let vel_rcp_z = _mm_set1_ps(vel_rcp.z());

    let origin_x = _mm_set1_ps(ray.origin.x());
    let origin_y = _mm_set1_ps(ray.origin.y());
    let origin_z = _mm_set1_ps(ray.origin.z());

    let t_start = _mm_set1_ps(t_range.start);
    let t_end = _mm_set1_ps(t_range.end);

    let mut intersections = 0;

    for off in (0..8).step_by(4) {
        let aabb_min_x = _mm_load_ps(aabb_min.x().as_ptr().add(off));
        let aabb_min_y = _mm_load_ps(aabb_min.y().as_ptr().add(off));
        let aabb_min_z = _mm_load_ps(aabb_min.z().as_ptr().add(off));

        let aabb_max_x = _mm_load_ps(aabb_max.x().as_ptr().add(off));
        let aabb_max_y = _mm_load_ps(aabb_max.y().as_ptr().add(off));
        let aabb_max_z = _mm_load_ps(aabb_max.z().as_ptr().add(off));

        let t0_x = _mm_mul_ps(_mm_sub_ps(aabb_min_x, origin_x), vel_rcp_x);
        let t0_y = _mm_mul_ps(_mm_sub_ps(aabb_min_y, origin_y), vel_rcp_y);
        let t0_z = _mm_mul_ps(_mm_sub_ps(aabb_min_z, origin_z), vel_rcp_z);

        let t1_x = _mm_mul_ps(_mm_sub_ps(aabb_max_x, origin_x), vel_rcp_x);
        let t1_y = _mm_mul_ps(_mm_sub_ps(aabb_max_y, origin_y), vel_rcp_y);
        let t1_z = _mm_mul_ps(_mm_sub_ps(aabb_max_z, origin_z), vel_rcp_z);

        let mut tmin = t_start;
        tmin = _mm_min_ps(_mm_max_ps(t0_x, tmin), _mm_max_ps(t1_x, tmin));
        tmin = _mm_min_ps(_mm_max_ps(t0_y, tmin), _mm_max_ps(t1_y, tmin));
        tmin = _mm_min_ps(_mm_max_ps(t0_z, tmin), _mm_max_ps(t1_z, tmin));

        let mut tmax = t_end;
        tmax = _mm_max_ps(_mm_min_ps(t0_x, tmax), _mm_min_ps(t1_x, tmax));
        tmax = _mm_max_ps(_mm_min_ps(t0_y, tmax), _mm_min_ps(t1_y, tmax));
        tmax = _mm_max_ps(_mm_min_ps(t0_z, tmax), _mm_min_ps(t1_z, tmax));

        tmax = _mm_mul_ps(tmax, _mm_set1_ps(1.0 + 2.0 * gamma(3)));
        let mask = _mm_cmp_ps(tmin, tmax, _CMP_LE_OQ);

        intersections |= (_mm_movemask_ps(mask) as u8) << off;
    }

    intersections
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn intersections_x86_avx(
    ray: Ray,
    aabb_min: &Vector3x8,
    aabb_max: &Vector3x8,
    t_range: Range<f32>,
) -> u8 {
    use std::arch::x86_64::*;

    let vel_rcp = 1.0 / ray.direction;
    let vel_rcp_x = _mm256_set1_ps(vel_rcp.x());
    let vel_rcp_y = _mm256_set1_ps(vel_rcp.y());
    let vel_rcp_z = _mm256_set1_ps(vel_rcp.z());

    let origin_x = _mm256_set1_ps(ray.origin.x());
    let origin_y = _mm256_set1_ps(ray.origin.y());
    let origin_z = _mm256_set1_ps(ray.origin.z());

    let aabb_min_x = _mm256_load_ps(aabb_min.x().as_ptr());
    let aabb_min_y = _mm256_load_ps(aabb_min.y().as_ptr());
    let aabb_min_z = _mm256_load_ps(aabb_min.z().as_ptr());

    let aabb_max_x = _mm256_load_ps(aabb_max.x().as_ptr());
    let aabb_max_y = _mm256_load_ps(aabb_max.y().as_ptr());
    let aabb_max_z = _mm256_load_ps(aabb_max.z().as_ptr());

    let t0_x = _mm256_mul_ps(_mm256_sub_ps(aabb_min_x, origin_x), vel_rcp_x);
    let t0_y = _mm256_mul_ps(_mm256_sub_ps(aabb_min_y, origin_y), vel_rcp_y);
    let t0_z = _mm256_mul_ps(_mm256_sub_ps(aabb_min_z, origin_z), vel_rcp_z);

    let t1_x = _mm256_mul_ps(_mm256_sub_ps(aabb_max_x, origin_x), vel_rcp_x);
    let t1_y = _mm256_mul_ps(_mm256_sub_ps(aabb_max_y, origin_y), vel_rcp_y);
    let t1_z = _mm256_mul_ps(_mm256_sub_ps(aabb_max_z, origin_z), vel_rcp_z);

    let mut tmin = _mm256_set1_ps(t_range.start);
    tmin = _mm256_min_ps(_mm256_max_ps(t0_x, tmin), _mm256_max_ps(t1_x, tmin));
    tmin = _mm256_min_ps(_mm256_max_ps(t0_y, tmin), _mm256_max_ps(t1_y, tmin));
    tmin = _mm256_min_ps(_mm256_max_ps(t0_z, tmin), _mm256_max_ps(t1_z, tmin));

    let mut tmax = _mm256_set1_ps(t_range.end);
    tmax = _mm256_max_ps(_mm256_min_ps(t0_x, tmax), _mm256_min_ps(t1_x, tmax));
    tmax = _mm256_max_ps(_mm256_min_ps(t0_y, tmax), _mm256_min_ps(t1_y, tmax));
    tmax = _mm256_max_ps(_mm256_min_ps(t0_z, tmax), _mm256_min_ps(t1_z, tmax));

    tmax = _mm256_mul_ps(tmax, _mm256_set1_ps(1.0 + 2.0 * gamma(3)));
    let mask = _mm256_cmp_ps(tmin, tmax, _CMP_LE_OQ);

    _mm256_movemask_ps(mask) as u8
}

fn build(
    objects: &mut [ObjectInfo],
    offset: usize,
    branches: &mut Vec<Branch>,
) -> (Node, Aabb, usize) {
    if objects.len() <= 1 {
        build_leaf(objects, offset)
    } else {
        build_branch(objects, offset, branches)
    }
}

fn build_leaf(objects: &mut [ObjectInfo], offset: usize) -> (Node, Aabb, usize) {
    let aabb = objects
        .iter()
        .map(|obj| obj.bounds)
        .reduce(|a, b| a.merge(&b))
        .unwrap_or(Aabb::ZERO);

    let child = Node::Leaf {
        offset: offset as u32,
        length: objects.len() as u16,
    };
    (child, aabb, 0)
}

fn build_branch(
    objects: &mut [ObjectInfo],
    mut offset: usize,
    branches: &mut Vec<Branch>,
) -> (Node, Aabb, usize) {
    let splits = split8(objects);

    let own_idx = branches.len();
    branches.push(Branch {
        aabb_min: Vector3x8::ZERO,
        aabb_max: Vector3x8::ZERO,
        children: [Node::Leaf {
            offset: 0,
            length: 0,
        }; 8],
    });

    let mut max_depth = 0;
    let mut aabb: Option<Aabb> = None;
    for (i, split) in splits
        .into_iter()
        .filter(|split| !split.is_empty())
        .enumerate()
    {
        let (child, child_aabb, child_max_depth) = build(split, offset, branches);

        let branch = &mut branches[own_idx];
        branch.aabb_min.set_vec(i, child_aabb.minimum.into());
        branch.aabb_max.set_vec(i, child_aabb.maximum.into());
        branch.children[i] = child;

        offset += split.len();
        aabb = aabb
            .map(|aabb| aabb.merge(&child_aabb))
            .or(Some(child_aabb));
        max_depth = max_depth.max(child_max_depth);
    }

    let child = Node::Branch {
        idx: own_idx as u32,
    };

    (child, aabb.unwrap(), max_depth + 1)
}

fn split8(objects: &mut [ObjectInfo]) -> [&mut [ObjectInfo]; 8] {
    let (s1_4, s5_8) = split(objects);

    let (s1_2, s3_4) = split(s1_4);
    let (s5_6, s7_8) = split(s5_8);

    let (s1, s2) = split(s1_2);
    let (s3, s4) = split(s3_4);
    let (s5, s6) = split(s5_6);
    let (s7, s8) = split(s7_8);

    [s1, s2, s3, s4, s5, s6, s7, s8]
}

fn split(objects: &mut [ObjectInfo]) -> (&mut [ObjectInfo], &mut [ObjectInfo]) {
    match objects.len() {
        0 => (&mut [], &mut []),
        1 => (objects, &mut []),
        2 => objects.split_at_mut(1),
        _ => split_sah(objects),
    }
}

fn split_sah(objects: &mut [ObjectInfo]) -> (&mut [ObjectInfo], &mut [ObjectInfo]) {
    let mut best_cost = f32::INFINITY;
    let mut best_split = None;

    for axis in [Dimension::X, Dimension::Y, Dimension::Z] {
        for object in objects.iter() {
            let pos = object.centroid[axis];

            let cost = calc_sah(objects, pos, axis);
            if cost < best_cost {
                best_cost = cost;
                best_split = Some((pos, axis));
            }
        }
    }

    let (pos, axis) = best_split.expect("objects.len() > 0");
    split_middle(objects, pos, axis)
}

fn calc_sah(objects: &[ObjectInfo], pos: f32, axis: Dimension) -> f32 {
    let mut left_count = 0;
    let mut right_count = 0;

    let mut left_aabb: Option<Aabb> = None;
    let mut right_aabb: Option<Aabb> = None;

    for obj in objects {
        let aabb = obj.bounds;
        let centroid = obj.centroid[axis];

        if centroid < pos {
            left_count += 1;
            left_aabb = left_aabb.map(|aabb| aabb.merge(&aabb)).or(Some(aabb));
        } else {
            right_count += 1;
            right_aabb = right_aabb.map(|aabb| aabb.merge(&aabb)).or(Some(aabb));
        }
    }

    let left_area = left_aabb.map_or(0.0, |aabb| aabb.surface_area());
    let right_area = right_aabb.map_or(0.0, |aabb| aabb.surface_area());

    left_area * left_count as f32 + right_area * right_count as f32
}

fn split_middle(
    objects: &mut [ObjectInfo],
    mid: f32,
    split_dim: Dimension,
) -> (&mut [ObjectInfo], &mut [ObjectInfo]) {
    let split_idx = partition(&mut objects[..], |obj| obj.centroid[split_dim] < mid);

    if split_idx == 0 || split_idx == objects.len() {
        split_equal_counts(objects, split_dim)
    } else {
        objects.split_at_mut(split_idx)
    }
}

fn split_equal_counts(
    objects: &mut [ObjectInfo],
    split_dim: Dimension,
) -> (&mut [ObjectInfo], &mut [ObjectInfo]) {
    if objects.is_empty() {
        return (&mut [], &mut []);
    }

    let mid = objects.len() / 2;

    objects.select_nth_unstable_by(mid, |a, b| {
        a.centroid[split_dim].total_cmp(&b.centroid[split_dim])
    });

    objects.split_at_mut(mid)
}

fn partition<T>(data: &mut [T], predicate: impl Fn(&T) -> bool) -> usize {
    let len = data.len();
    if len == 0 {
        return 0;
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return l;
        }
        data.swap(l, r);
    }
}
