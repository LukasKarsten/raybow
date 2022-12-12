use std::ops::Range;

use crate::ray::Ray;

use super::{Hit, Hittable};

pub struct World {
    geometries: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            geometries: Vec::new(),
        }
    }

    pub fn add_geometry(&mut self, geometry: Box<dyn Hittable>) {
        self.geometries.push(geometry);
    }
}

impl Hittable for World {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit> {
        let mut nearest_hit = None;
        let mut nearest_t = t_range.end;

        for geometry in &self.geometries {
            if let Some(hit) = geometry.hit(ray, t_range.start..nearest_t) {
                nearest_t = hit.t;
                nearest_hit = Some(hit);
            }
        }

        nearest_hit
    }

    fn bounding_box(&self) -> super::Aabb {
        unimplemented!()
    }
}
