use std::{ops::Range, sync::Arc};

use crate::{material::Material, ray::Ray, vector::Vector};

use super::{Aabb, Hit, Hittable};

pub struct Sphere {
    center: Vector,
    radius: f32,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vector, radius: f32, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit> {
        let oc = self.center - ray.origin;
        let tca = oc.dot(ray.velocity);
        if tca < 0.0 {
            return None;
        }

        let d2 = oc.length_squared() - tca * tca;

        let r2 = self.radius * self.radius;
        if d2 > r2 {
            return None;
        }

        let thc = (r2 - d2).sqrt();

        let mut t = tca - thc;

        if t_range.start > t || t_range.end < t {
            t = tca + thc;
            if t_range.start > t || t_range.end < t {
                return None;
            }
        }

        let point = ray.at(t);

        Some(Hit::new(
            point,
            (point - self.center) / self.radius,
            ray,
            t,
            Arc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Aabb {
        Aabb {
            minimum: self.center - Vector::from_xyz(self.radius, self.radius, self.radius),
            maximum: self.center + Vector::from_xyz(self.radius, self.radius, self.radius),
        }
    }
}
