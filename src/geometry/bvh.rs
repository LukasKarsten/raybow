use std::{num::NonZeroU16, sync::Arc};

use bumpalo::Bump;

use crate::{
    ray::Ray,
    vector::{Dimension, Vector},
};

use super::{Aabb, Hit, Hittable};

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum SplitMethod {
    Middle,
    EqualCounts,
    //Sah,
}

struct ObjectInfo {
    aabb: Aabb,
    centroid: Vector,
    object: Arc<dyn Hittable>,
}

impl ObjectInfo {
    fn new(object: Arc<dyn Hittable>) -> Self {
        let aabb = object.bounding_box();
        Self {
            aabb,
            centroid: (aabb.minimum + aabb.maximum) * 0.5,
            object,
        }
    }
}

struct BuildNode<'a> {
    aabb: Aabb,
    variant: BuildNodeVariant<'a>,
}

enum BuildNodeVariant<'a> {
    Leaf {
        objects_offset: u32,
        num_objects: NonZeroU16,
    },
    Branch {
        split_dim: Dimension,
        children: [&'a BuildNode<'a>; 2],
    },
}

impl<'a> BuildNode<'a> {
    fn build_recursive(
        split_method: SplitMethod,
        arena: &'a Bump,
        objects: &'a mut [ObjectInfo],
        total_nodes: &mut usize,
        ordered_objects: &mut Vec<Arc<dyn Hittable>>,
    ) -> &'a BuildNode<'a> {
        *total_nodes += 1;

        let aabb = objects
            .iter()
            .map(|obj| obj.aabb)
            .reduce(|a, b| a.merge(&b))
            .expect("objects is not empty");

        let centroid_bounds = centroid_bounds(objects);

        match centroid_bounds.maximum_extent() {
            None => {
                let object = &objects[0];
                let objects_offset = ordered_objects.len();
                ordered_objects.push(Arc::clone(&object.object));
                arena.alloc(Self {
                    aabb,
                    variant: BuildNodeVariant::Leaf {
                        objects_offset: objects_offset
                            .try_into()
                            .expect("ordered_objects.len() fits into 32 bits"),
                        num_objects: NonZeroU16::new(
                            objects.len().try_into().unwrap(), // XXX: Split somehow if too large
                        )
                        .expect("objects is not empty"),
                    },
                })
            }
            Some(split_dim) => {
                let (left, right) = match split_method {
                    SplitMethod::Middle => split_middle(objects, &centroid_bounds, split_dim),
                    SplitMethod::EqualCounts => split_equal_counts(objects, split_dim),
                };

                arena.alloc(BuildNode {
                    aabb,
                    variant: BuildNodeVariant::Branch {
                        split_dim,
                        children: [
                            Self::build_recursive(
                                split_method,
                                arena,
                                left,
                                total_nodes,
                                ordered_objects,
                            ),
                            Self::build_recursive(
                                split_method,
                                arena,
                                right,
                                total_nodes,
                                ordered_objects,
                            ),
                        ],
                    },
                })
            }
        }
    }

    fn into_linear(&self, nodes: &mut Vec<LinearNode>) {
        match self.variant {
            BuildNodeVariant::Leaf {
                objects_offset,
                num_objects,
            } => {
                nodes.push(LinearNode {
                    aabb: self.aabb,
                    variant: LinearNodeVariant::Leaf {
                        objects_offset,
                        num_objects,
                    },
                });
            }
            BuildNodeVariant::Branch {
                split_dim,
                children,
            } => {
                let idx = nodes.len();

                nodes.push(LinearNode {
                    aabb: self.aabb,
                    variant: LinearNodeVariant::Branch {
                        second_child_offset: 0,
                        split_dim,
                    },
                });
                children[0].into_linear(nodes);
                let second_child_offset_val = nodes
                    .len()
                    .try_into()
                    .expect("nodes.len() fits into 32 bits");
                let LinearNodeVariant::Branch { second_child_offset, .. } = &mut nodes[idx].variant else {
                    panic!("this function is such a clusterfuck");
                };
                *second_child_offset = second_child_offset_val;
                children[1].into_linear(nodes);
            }
        }
    }
}

fn centroid_bounds(objects: &[ObjectInfo]) -> Aabb {
    let mut iter = objects.iter();

    let first_obj = iter.next().expect("objects is not empty");

    let mut aabb = Aabb {
        minimum: first_obj.centroid,
        maximum: first_obj.centroid,
    };

    for obj in iter {
        aabb = aabb.merge_vector(&obj.centroid);
    }

    aabb
}

fn split_middle<'o>(
    objects: &'o mut [ObjectInfo],
    centroid_bounds: &Aabb,
    split_dim: Dimension,
) -> (&'o mut [ObjectInfo], &'o mut [ObjectInfo]) {
    let mid = (centroid_bounds.maximum[split_dim] - centroid_bounds.minimum[split_dim]) / 2.0;
    let split_idx = partition(objects, |obj| obj.centroid[split_dim] < mid);
    if split_idx == 0 || split_idx == objects.len() {
        split_equal_counts(objects, split_dim)
    } else {
        objects.split_at_mut(split_idx)
    }
}

fn split_equal_counts<'o>(
    objects: &'o mut [ObjectInfo],
    split_dim: Dimension,
) -> (&'o mut [ObjectInfo], &'o mut [ObjectInfo]) {
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

#[repr(align(32))] // Align to cache lines boundaries
struct LinearNode {
    aabb: Aabb,
    variant: LinearNodeVariant,
}

enum LinearNodeVariant {
    Leaf {
        objects_offset: u32,
        num_objects: NonZeroU16,
    },
    Branch {
        second_child_offset: u32,
        split_dim: Dimension,
    },
}

pub struct LinearTree {
    nodes: Vec<LinearNode>,
    ordered_objects: Vec<Arc<dyn Hittable>>,
}

impl LinearTree {
    pub fn new(objects: Vec<Arc<dyn Hittable>>) -> Self {
        let arena = Bump::new();

        let mut object_infos: Vec<ObjectInfo> = objects.into_iter().map(ObjectInfo::new).collect();

        let mut ordered_objects = Vec::new();

        let mut total_nodes = 0;
        let build_node = BuildNode::build_recursive(
            SplitMethod::Middle,
            &arena,
            &mut object_infos,
            &mut total_nodes,
            &mut ordered_objects,
        );

        let mut nodes = Vec::new();
        build_node.into_linear(&mut nodes);

        Self {
            nodes,
            ordered_objects,
        }
    }

    pub fn hit(&self, ray: Ray, _arena: &Bump) -> Option<Hit> {
        let inv_vel = ray.velocity.reciprocal();
        let vel_is_neg = [inv_vel.x() < 0.0, inv_vel.y() < 0.0, inv_vel.z() < 0.0];

        let mut nodes_to_visit = Vec::with_capacity(64);
        let mut curr_node_idx: u32 = 0;

        let mut nearest_hit = None;
        let mut nearest_t = f32::INFINITY;

        loop {
            let node = &self.nodes[curr_node_idx as usize];
            if node.aabb.hit(ray, 0.0001..nearest_t) {
                match node.variant {
                    LinearNodeVariant::Leaf {
                        objects_offset,
                        num_objects,
                    } => {
                        let start: usize = objects_offset.try_into().unwrap();
                        let end = start + usize::from(num_objects.get());
                        for i in start..end {
                            let object = &self.ordered_objects[i];
                            if let Some(hit) = object.hit(ray, 0.0001..nearest_t) {
                                nearest_t = hit.t;
                                nearest_hit = Some(hit);
                            }
                        }
                        match nodes_to_visit.pop() {
                            Some(node) => curr_node_idx = node,
                            None => break,
                        }
                    }
                    LinearNodeVariant::Branch {
                        second_child_offset,
                        split_dim,
                    } => {
                        if vel_is_neg[usize::from(split_dim as u8)] {
                            nodes_to_visit.push(curr_node_idx + 1);
                            curr_node_idx = second_child_offset;
                        } else {
                            nodes_to_visit.push(second_child_offset);
                            curr_node_idx += 1;
                        }
                    }
                }
            } else {
                match nodes_to_visit.pop() {
                    Some(node) => curr_node_idx = node,
                    None => break,
                }
            }
        }

        nearest_hit
    }
}
