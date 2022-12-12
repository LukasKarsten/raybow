use std::{ops::Range, sync::Arc};

pub use bvh::BvhNode;
pub use sphere::Sphere;
pub use world::World;

use crate::{material::Material, ray::Ray, vector::Vector};

mod bvh;
mod sphere;
mod world;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit>;
    fn bounding_box(&self) -> Aabb;
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

    pub fn hit(&self, ray: Ray, t_range: Range<f64>) -> bool {
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

pub struct Hit {
    pub point: Vector,
    pub normal: Vector,
    pub ray: Ray,
    pub front_face: bool,
    pub t: f64,
    pub material: Arc<dyn Material>,
}

impl Hit {
    pub fn new(
        point: Vector,
        normal: Vector,
        ray: Ray,
        t: f64,
        material: Arc<dyn Material>,
    ) -> Self {
        let (normal, front_face) = if ray.velocity.dot(normal) < 0.0 {
            (normal, true)
        } else {
            (-normal, false)
        };
        Self {
            point,
            normal,
            ray,
            front_face,
            t,
            material,
        }
    }
}
