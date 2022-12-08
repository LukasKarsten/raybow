use crate::{color::Color, geometry::Hit, ray::Ray};

use super::{Material, random_unit_vector};

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)> {
        let mut scatter_dir = hit.normal + random_unit_vector();

        if scatter_dir.is_almost_zero() {
            scatter_dir = hit.normal;
        }

        let scattered = Ray::new(hit.point, scatter_dir);
        Some((scattered, self.albedo))
    }
}
