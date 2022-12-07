mod sphere;
mod world;

use std::{ops::Range, sync::Arc};

pub use sphere::Sphere;
pub use world::World;

use crate::{material::Material, ray::Ray, vector::Vector};

pub trait Geometry: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit>;
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
