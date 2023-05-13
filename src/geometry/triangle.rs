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
        let [v0, v1, v2] = self.fetch_vertices(index);

        // translate so that the ray origin is at the origin
        let v0t = v0 - ray.origin;
        let v1t = v1 - ray.origin;
        let v2t = v2 - ray.origin;

        // find the largest axis of the ray direction
        let kz = ray.velocity.abs().largest_axis() as usize;
        let kx = (kz + 1) % 3;
        let ky = (kx + 1) % 3;

        // rotate so that the ray direction's largest axis is the z axis
        let d = Vector::from_xyz(ray.velocity[kx], ray.velocity[ky], ray.velocity[kz]);
        let v0t = Vector::from_xyz(v0t[kx], v0t[ky], v0t[kz]);
        let v1t = Vector::from_xyz(v1t[kx], v1t[ky], v1t[kz]);
        let v2t = Vector::from_xyz(v2t[kx], v2t[ky], v2t[kz]);

        // shear so that the ray direction is aligned with the z axis (we ignore z for now)
        let sz = 1.0 / d.z();
        let sx = -d.x() * sz;
        let sy = -d.y() * sz;

        let v0t = Vector::from_xyz(v0t.x() + sx * v0t.z(), v0t.y() + sy * v0t.z(), v0t.z());
        let v1t = Vector::from_xyz(v1t.x() + sx * v1t.z(), v1t.y() + sy * v1t.z(), v1t.z());
        let v2t = Vector::from_xyz(v2t.x() + sx * v2t.z(), v2t.y() + sy * v2t.z(), v2t.z());

        // compute edge functions
        let e0 = v1t.x() * v2t.y() - v2t.x() * v1t.y();
        let e1 = v2t.x() * v0t.y() - v0t.x() * v2t.y();
        let e2 = v0t.x() * v1t.y() - v1t.x() * v0t.y();

        if (e0 < 0.0 || e1 < 0.0 || e2 < 0.0) && (e0 > 0.0 || e1 > 0.0 || e2 > 0.0) {
            return None;
        }
        let det = e0 + e1 + e2;
        if det == 0.0 {
            return None;
        }

        let z0 = v0t.z() * sz;
        let z1 = v1t.z() * sz;
        let z2 = v2t.z() * sz;
        let t_scaled = e0 * z0 + e1 * z1 + e2 * z2;
        if (det < 0.0 && (t_scaled >= 0.0 || t_scaled < t_range.end * det))
            || (det > 0.0 && (t_scaled <= 0.0 || t_scaled > t_range.end * det))
        {
            return None;
        }

        let inv_det = 1.0 / det;
        let b0 = e0 * inv_det;
        let b1 = e1 * inv_det;
        let b2 = e2 * inv_det;

        let point = b0 * v0 + b1 * v1 + b2 * v2;

        let t = (point - ray.origin).length();

        if t < t_range.start || t > t_range.end {
            return None;
        }

        let normal = (v1 - v0).cross3(v2 - v0).normalize_unchecked();
        Some(Hit::new(point, normal, ray, t, self.material.as_ref()))
    }

    fn bounding_box(&self, index: usize) -> Aabb {
        let [v0, v1, v2] = self.fetch_vertices(index);

        let minimum = v0.min(v1).min(v2);
        let maximum = v0.max(v1).max(v2);

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
