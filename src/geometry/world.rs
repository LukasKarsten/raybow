use std::ops::Range;

use crate::ray::Ray;

use super::{Aabb, Hit, Hittable};

pub struct World {
    bounding_boxes: Vec<Aabb>,
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            bounding_boxes: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn push(&mut self, object: Box<dyn Hittable>) {
        self.bounding_boxes.push(object.bounding_box());
        self.objects.push(object);
    }
}

impl Hittable for World {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit> {
        let mut ts: Vec<f64> = std::iter::repeat(f64::INFINITY)
            .take(self.bounding_boxes.len())
            .collect();
        Aabb::intersections(ray, &self.bounding_boxes, &mut ts);

        let mut nearest_hit = None;
        let mut nearest_t = t_range.end;

        for (i, t) in ts.into_iter().enumerate() {
            if t < f64::INFINITY {
                let object = unsafe { self.objects.get_unchecked(i) };
                if let Some(hit) = object.hit(ray, t_range.start..nearest_t) {
                    nearest_t = hit.t;
                    nearest_hit = Some(hit);
                }
            }
        }

        nearest_hit
    }

    fn bounding_box(&self) -> super::Aabb {
        unimplemented!()
    }
}
