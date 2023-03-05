use std::ops::Range;

pub use aabb::{Aabb, AabbList};
pub use sphere::Sphere;

use crate::{material::Material, ray::Ray, vector::Vector};

mod aabb;
pub mod bvh;
mod sphere;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit>;
    fn bounding_box(&self) -> Aabb;
}

pub struct Hit<'m> {
    pub point: Vector,
    pub normal: Vector,
    pub ray: Ray,
    pub front_face: bool,
    pub t: f32,
    pub material: &'m dyn Material,
}

impl<'m> Hit<'m> {
    pub fn new(
        point: Vector,
        normal: Vector,
        ray: Ray,
        t: f32,
        material: &'m dyn Material,
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
