use std::ops::Range;

use bumpalo::Bump;

use crate::{
    ray::Ray,
    vector::{Vector, Vector3x8},
};

#[derive(Default)]
struct Aabbx8 {
    pub minimum: Vector3x8,
    pub maximum: Vector3x8,
}

pub struct AabbList {
    boxes: Vec<Aabbx8>,
    free_remaining: usize,
}

impl AabbList {
    pub fn new() -> Self {
        Self {
            boxes: Vec::new(),
            free_remaining: 0,
        }
    }

    pub fn push(&mut self, aabb: Aabb) {
        if self.free_remaining == 0 {
            self.boxes.push(Aabbx8::default());
            self.free_remaining = 8;
        }

        self.free_remaining -= 1;
        let last = self.boxes.last_mut().unwrap();
        let idx = 7 - self.free_remaining;
        last.minimum.set_vec(idx, aabb.minimum.into());
        last.maximum.set_vec(idx, aabb.maximum.into());
    }

    pub fn intersections<'a>(&self, ray: Ray, arena: &'a mut Bump) -> &'a [f32] {
        let len = self.boxes.len();

        let ts = arena.alloc_slice_fill_copy(len * 8, f32::INFINITY);

        let origin = Vector3x8::from(ray.origin);
        let velocity_rcp = Vector3x8::from(ray.velocity).reciprocal();

        for i in 0..len {
            let bounding_box = &self.boxes[i];
            let t = &mut ts[(i * 8)..(i * 8 + 8)];

            let t0 = (bounding_box.minimum - origin) * velocity_rcp;
            let t1 = (bounding_box.maximum - origin) * velocity_rcp;

            /*
            unsafe {
                use std::arch::x86_64::*;

                let t0x = _mm256_load_ps(t0.x().as_ptr());
                let t0y = _mm256_load_ps(t0.y().as_ptr());
                let t0z = _mm256_load_ps(t0.z().as_ptr());

                let t0min = _mm256_min_ps(_mm256_min_ps(t0x, t0y), t0z);
                let t0max = _mm256_max_ps(_mm256_max_ps(t0x, t0y), t0z);

                let t1x = _mm256_load_ps(t1.x().as_ptr());
                let t1y = _mm256_load_ps(t1.y().as_ptr());
                let t1z = _mm256_load_ps(t1.z().as_ptr());

                let t1min = _mm256_min_ps(_mm256_min_ps(t1x, t1y), t1z);
                let t1max = _mm256_max_ps(_mm256_max_ps(t1x, t1y), t1z);

                let min = _mm256_min_ps(t0min, t1min);
                let max = _mm256_max_ps(t0max, t1max);

                let tmin = _mm256_set1_ps(0.0);
                let tmax = _mm256_loadu_ps(t.as_ptr());

                let min = _mm256_max_ps(tmin, min);
                let max = _mm256_min_ps(tmax, max);

                let cmp = _mm256_castps_si256(_mm256_cmp_ps::<_CMP_LE_OQ>(min, max));
                _mm256_maskstore_ps(t.as_mut_ptr(), cmp, min);
            }
            */

            let mut tmin: [f32; 8] = [0.0001; 8];
            let mut tmax: [f32; 8] = t.try_into().unwrap();

            for i in 0..8 {
                tmin[i] = tmin[i].max(t0.x()[i].min(t1.x()[i]));
                tmin[i] = tmin[i].max(t0.y()[i].min(t1.y()[i]));
                tmin[i] = tmin[i].max(t0.z()[i].min(t1.z()[i]));

                tmax[i] = tmax[i].min(t0.x()[i].max(t1.x()[i]));
                tmax[i] = tmax[i].min(t0.y()[i].max(t1.y()[i]));
                tmax[i] = tmax[i].min(t0.z()[i].max(t1.z()[i]));

                if tmin[i] <= tmax[i] {
                    t[i] = tmin[i];
                }
            }
        }

        &ts[..(len * 8 - self.free_remaining)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub minimum: Vector,
    pub maximum: Vector,
}

impl Aabb {
    pub const ZERO: Self = Self {
        minimum: Vector::ZERO,
        maximum: Vector::ZERO,
    };

    pub fn hit(&self, ray: Ray, t_range: Range<f32>) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.velocity[a];
            let mut t0 = (self.minimum[a] - ray.origin[a]) * inv_d;
            let mut t1 = (self.maximum[a] - ray.origin[a]) * inv_d;
            if inv_d < 0.0 {
                std::mem::swap(&mut t0, &mut t1);
            }
            let t_min = t0.max(t_range.start);
            let t_max = t1.min(t_range.end);
            if t_max <= t_min {
                return false;
            }
        }

        true
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }

    pub fn intersection(&self, other: &Self) -> Vector {
        let min = self.minimum.max(other.minimum);
        let max = self.maximum.min(other.maximum);
        max - min
    }
}
