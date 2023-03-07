use std::ops::Range;

use crate::{
    ray::Ray,
    vector::{Dimension, Vector},
};

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
