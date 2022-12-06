mod sphere;
mod world;

use std::{ops::Range, sync::Arc};

pub use sphere::Sphere;
pub use world::World;

use crate::{ray::Ray, material::Material, vector::Vector};

pub trait Geometry: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit>;
}

pub struct Hit {
    pub ray: Ray,
    pub material: Arc<dyn Material>,
    pub point: Vector,
    pub normal: Vector,
    pub t: f64,
}

pub enum FaceSide {
    Inside,
    Outside,
}

impl Hit {
    pub fn side(&self) -> FaceSide {
        if self.ray.velocity.dot(self.normal) < 0.0 {
            FaceSide::Outside
        } else {
            FaceSide::Inside
        }
    }

    pub fn normal_opposite_to_ray(&self) -> Vector {
        match self.side() {
            FaceSide::Outside => self.normal,
            FaceSide::Inside => -self.normal,
        }
    }
}
