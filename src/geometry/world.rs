use bumpalo::Bump;

use crate::ray::Ray;

use super::{AabbList, Hit, Hittable};

pub struct World {
    bounding_boxes: AabbList,
    objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            bounding_boxes: AabbList::new(),
            objects: Vec::new(),
        }
    }

    pub fn push(&mut self, object: Box<dyn Hittable>) {
        self.bounding_boxes.push(object.bounding_box());
        self.objects.push(object);
    }
}

impl World {
    pub fn hit(&self, ray: Ray, arena: &mut Bump) -> Option<Hit> {
        let ts = self.bounding_boxes.intersections(ray, arena);
        assert_eq!(ts.len(), self.objects.len());

        let mut nearest_hit = None;
        let mut nearest_t = f32::INFINITY;

        for object in ts
            .into_iter()
            .zip(self.objects.iter())
            .filter_map(|(t, obj)| (*t < f32::INFINITY).then_some(obj))
        {
            if let Some(hit) = object.hit(ray, 0.0001..nearest_t) {
                nearest_t = hit.t;
                nearest_hit = Some(hit);
            }
        }

        nearest_hit
    }
}
