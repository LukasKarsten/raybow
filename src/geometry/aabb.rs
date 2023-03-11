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

        let min = t0.min(t1);
        let max = t0.max(t1);

        unsafe {
            use std::arch::x86_64::*;

            let tmin = _mm_set1_ps(t_range.start);
            let tmax = _mm_set1_ps(t_range.end);

            let min_0zyx = min.to_simd();
            let max_0zyx = max.to_simd();

            let min_wwyy = _mm_shuffle_ps::<0b00_00_01_01>(min_0zyx, tmin);
            let max_wwyy = _mm_shuffle_ps::<0b00_00_01_01>(max_0zyx, tmax);

            let min_0_zw_0_xy = _mm_max_ps(min_0zyx, min_wwyy);
            let max_0_zw_0_xy = _mm_min_ps(max_0zyx, max_wwyy);

            let min_0_0_0_zw = _mm_permute_ps::<0b10_10_10_10>(min_0_zw_0_xy);
            let max_0_0_0_zw = _mm_permute_ps::<0b10_10_10_10>(max_0_zw_0_xy);

            let tmin = _mm_max_ps(min_0_0_0_zw, min_0_zw_0_xy);
            let tmax = _mm_min_ps(max_0_0_0_zw, max_0_zw_0_xy);

            let tmin = f32::from_bits(_mm_extract_ps::<0>(tmin) as u32);
            let tmax = f32::from_bits(_mm_extract_ps::<0>(tmax) as u32);

            tmin <= tmax
        }
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
