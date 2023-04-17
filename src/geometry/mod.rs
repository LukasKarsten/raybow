use std::{ops::Range, sync::Arc};

use aabb::Aabb;
pub use sphere::Sphere;

use crate::{material::Material, ray::Ray, vector::Vector};

mod aabb;
pub mod bvh;
mod sphere;

pub trait Object: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit>;

    fn bounding_box(&self) -> Aabb;

    fn centroid(&self) -> Vector {
        let bounds = self.bounding_box();
        (bounds.minimum + bounds.maximum) * 0.5
    }
}

impl Object for Box<dyn Object> {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit> {
        self.as_ref().hit(ray, t_range)
    }

    fn bounding_box(&self) -> Aabb {
        self.as_ref().bounding_box()
    }
}

impl Object for Arc<dyn Object> {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit> {
        self.as_ref().hit(ray, t_range)
    }

    fn bounding_box(&self) -> Aabb {
        self.as_ref().bounding_box()
    }
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
