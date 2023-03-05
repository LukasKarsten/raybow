use std::ops::Range;

use bumpalo::Bump;

use crate::{
    ray::Ray,
    vector::{Dimension, Vector, Vector3x8},
};

#[derive(Default)]
struct Aabbx8 {
    pub minimum: Vector3x8,
    pub maximum: Vector3x8,
}

#[derive(Default)]
pub struct AabbList {
    boxes: Vec<Aabbx8>,
    free_remaining: usize,
}

impl AabbList {
    pub fn new() -> Self {
        Self::default()
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
        let velocity_rcp = Vector3x8::from(ray.velocity.reciprocal());

        for (i, bounding_box) in self.boxes.iter().enumerate() {
            let t = &mut ts[(i * 8)..(i * 8 + 8)];

            let t0 = (bounding_box.minimum - origin) * velocity_rcp;
            let t1 = (bounding_box.maximum - origin) * velocity_rcp;

            let mut tmin: [f32; 8] = [0.0001; 8];
            let mut tmax: [f32; 8] = t.try_into().unwrap();

            for j in 0..8 {
                tmin[j] = tmin[j].max(t0.x()[j].min(t1.x()[j]));
                tmin[j] = tmin[j].max(t0.y()[j].min(t1.y()[j]));
                tmin[j] = tmin[j].max(t0.z()[j].min(t1.z()[j]));

                tmax[j] = tmax[j].min(t0.x()[j].max(t1.x()[j]));
                tmax[j] = tmax[j].min(t0.y()[j].max(t1.y()[j]));
                tmax[j] = tmax[j].min(t0.z()[j].max(t1.z()[j]));

                if tmin[j] <= tmax[j] {
                    t[j] = tmin[j];
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
        let vel_rcp = ray.velocity.reciprocal();

        let t0 = (self.minimum - ray.origin) * vel_rcp;
        let t1 = (self.maximum - ray.origin) * vel_rcp;

        let mut tmin = t_range.start;
        let mut tmax = t_range.end;

        let mut min = t0.min(t1);
        min.0[3] = tmin;

        let mut max = t0.max(t1);
        max.0[3] = tmax;

        tmin = min.max_elem();
        tmax = max.min_elem();

        tmin <= tmax
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }

    pub fn merge_vector(&self, vector: &Vector) -> Self {
        Self {
            minimum: self.minimum.min(*vector),
            maximum: self.maximum.max(*vector),
        }
    }

    pub fn intersection(&self, other: &Self) -> Vector {
        let min = self.minimum.max(other.minimum);
        let max = self.maximum.min(other.maximum);
        max - min
    }

    pub fn maximum_extent(&self) -> Option<Dimension> {
        let diff = self.maximum - self.minimum;

        if diff.is_almost_zero() {
            None
        } else {
            let axis = if diff.x() > diff.y() {
                if diff.x() > diff.z() {
                    Dimension::X
                } else {
                    Dimension::Z
                }
            } else if diff.y() > diff.z() {
                Dimension::Y
            } else {
                Dimension::Z
            };
            Some(axis)
        }
    }
}
