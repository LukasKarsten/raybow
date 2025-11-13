use std::{ops::Range, sync::Arc};

use aabb::Aabb;
use bumpalo::Bump;
pub use sphere::Sphere;
pub use triangle::TriangleMesh;

use crate::{material::Material, ray::Ray, vector::Vector};

mod aabb;
pub mod bvh;
mod sphere;
mod triangle;

pub trait Object: Send + Sync {
    fn hit(&self, ray: Ray, t_range: Range<f32>, arena: &Bump) -> Option<Hit<'_>>;

    fn bounding_box(&self) -> Aabb;

    fn centroid(&self) -> Vector {
        let bounds = self.bounding_box();
        (bounds.minimum + bounds.maximum) * 0.5
    }
}

impl Object for Box<dyn Object> {
    fn hit(&self, ray: Ray, t_range: Range<f32>, arena: &Bump) -> Option<Hit<'_>> {
        self.as_ref().hit(ray, t_range, arena)
    }

    fn bounding_box(&self) -> Aabb {
        self.as_ref().bounding_box()
    }
}

impl Object for Arc<dyn Object> {
    fn hit(&self, ray: Ray, t_range: Range<f32>, arena: &Bump) -> Option<Hit<'_>> {
        self.as_ref().hit(ray, t_range, arena)
    }

    fn bounding_box(&self) -> Aabb {
        self.as_ref().bounding_box()
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait ObjectList {
    type Object;

    fn hit(&self, ray: Ray, t_range: Range<f32>, index: usize, arena: &Bump) -> Option<Hit<'_>>;

    fn bounding_box(&self, index: usize) -> Aabb;

    fn centroid(&self, index: usize) -> Vector {
        let bounds = self.bounding_box(index);
        (bounds.minimum + bounds.maximum) * 0.5
    }

    fn objects_mut(&mut self) -> &mut [Self::Object];

    fn len(&self) -> usize;
}

impl<O: Object> ObjectList for Vec<O> {
    type Object = O;

    fn hit(&self, ray: Ray, t_range: Range<f32>, index: usize, arena: &Bump) -> Option<Hit<'_>> {
        self[index].hit(ray, t_range, arena)
    }

    fn bounding_box(&self, index: usize) -> Aabb {
        self[index].bounding_box()
    }

    fn centroid(&self, index: usize) -> Vector {
        self[index].centroid()
    }

    fn objects_mut(&mut self) -> &mut [Self::Object] {
        self
    }

    fn len(&self) -> usize {
        self.len()
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
        let (normal, front_face) = if ray.direction.dot(normal) < 0.0 {
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
