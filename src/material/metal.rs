use crate::{color::Color, geometry::Hit, ray::Ray, RayState, RngKey};

use super::{random_in_unit_sphere, reflect, Material};

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f32,
}

impl Material for Metal {
    fn scatter(&self, hit: &Hit, state: &RayState) -> Option<(Ray, Color)> {
        let fuzz_dir = random_in_unit_sphere(state, RngKey::MetalFuzzDirection);

        let reflected = reflect(hit.ray.velocity.normalize_unchecked(), hit.normal);
        let scattered = Ray::new(hit.point, reflected + self.fuzz * fuzz_dir);
        Some((scattered, self.albedo))
    }
}
