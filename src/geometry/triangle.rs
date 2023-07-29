use std::{ops::Range, sync::Arc};

use bumpalo::Bump;

use crate::{material::Material, ray::Ray, vector::Vector};

use super::{aabb::Aabb, Hit, ObjectList};

pub struct TriangleMesh {
    triangles: Box<[Triangle]>,
    vertices: Box<[f32]>,
    material: Arc<dyn Material>,
}

impl TriangleMesh {
    pub fn new(vertices: Box<[f32]>, indices: Box<[u32]>, material: Arc<dyn Material>) -> Self {
        assert!(indices.len() % 3 == 0);

        let triangles = bytemuck::cast_slice_box(indices);

        Self {
            triangles,
            vertices,
            material,
        }
    }

    fn fetch_vertices(&self, triangle_index: usize) -> [Vector; 3] {
        let triangle = self.triangles[triangle_index];
        triangle.indices.map(|i| self.fetch_vertex(i))
    }

    fn fetch_vertex(&self, index: u32) -> Vector {
        let index = index as usize * 3;
        Vector::from_xyz(
            self.vertices[index],
            self.vertices[index + 1],
            self.vertices[index + 2],
        )
    }
}

impl ObjectList for TriangleMesh {
    type Object = Triangle;

    fn hit(&self, ray: Ray, t_range: Range<f32>, index: usize, _: &Bump) -> Option<Hit> {
        // https://jcgt.org/published/0002/01/05/paper.pdf

        let ray_dir = ray.direction;

        let kz = ray.direction.abs().largest_axis() as usize;
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;

        let sz = 1.0 / ray_dir[kz];
        let sx = -ray_dir[kx] * sz;
        let sy = -ray_dir[ky] * sz;

        let points = self.fetch_vertices(index);

        let [p1t, p2t, p3t] = points
            .map(|p| p - ray.origin)
            .map(|p| Vector::from_xyz(p[kx], p[ky], p[kz]))
            .map(|p| Vector::from_xyz(p.x() + sx * p.z(), p.y() + sy * p.z(), p.z() * sz));

        let mut e1 = p2t.x() * p3t.y() - p3t.x() * p2t.y();
        let mut e2 = p3t.x() * p1t.y() - p1t.x() * p3t.y();
        let mut e3 = p1t.x() * p2t.y() - p2t.x() * p1t.y();

        if e1 == 0.0 || e2 == 0.0 || e3 == 0.0 {
            e1 = (p2t.x() as f64 * p3t.y() as f64 - p3t.x() as f64 * p2t.y() as f64) as f32;
            e2 = (p3t.x() as f64 * p1t.y() as f64 - p1t.x() as f64 * p3t.y() as f64) as f32;
            e3 = (p1t.x() as f64 * p2t.y() as f64 - p2t.x() as f64 * p1t.y() as f64) as f32;
        }

        if (e1 < 0.0 || e2 < 0.0 || e3 < 0.0) && (e1 > 0.0 || e2 > 0.0 || e3 > 0.0) {
            return None;
        }

        let det = e1 + e2 + e3;
        if det == 0.0 {
            return None;
        }

        let t_scaled = e1 * p1t.z() + e2 * p2t.z() + e3 * p3t.z();
        if (det < 0.0 && (t_scaled >= 0.0 || t_scaled < t_range.end * det))
            || (det > 0.0 && (t_scaled <= 0.0 || t_scaled > t_range.end * det))
        {
            return None;
        }

        let inv_det = 1.0 / det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;
        let b3 = e3 * inv_det;

        let point = b1 * p1t + b2 * p2t + b3 * p3t;

        let t = t_scaled / det;

        if t < t_range.start || t > t_range.end {
            return None;
        }

        let normal = (p2t - p1t).cross3(p3t - p1t).normalize_unchecked();
        Some(Hit::new(point, normal, ray, t, self.material.as_ref()))
    }

    fn bounding_box(&self, index: usize) -> Aabb {
        let [p1, p2, p3] = self.fetch_vertices(index);

        let minimum = p1.min(p2).min(p3);
        let maximum = p1.max(p2).max(p3);

        Aabb { minimum, maximum }
    }

    fn objects_mut(&mut self) -> &mut [Self::Object] {
        &mut self.triangles
    }

    fn len(&self) -> usize {
        self.triangles.len()
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    indices: [u32; 3],
}
