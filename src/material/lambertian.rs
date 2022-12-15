use crate::{color::Color, geometry::Hit, ray::Ray, RayState, RngKey};

use super::{random_unit_vector, Material};

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, hit: &Hit, state: &RayState) -> Option<(Ray, Color)> {
        let mut scatter_dir = hit.normal + random_unit_vector(state, RngKey::ScatterDirection);

        if scatter_dir.is_almost_zero() {
            scatter_dir = hit.normal;
        }

        let scattered = Ray::new(hit.point, scatter_dir);
        Some((scattered, self.albedo))
    }
}
