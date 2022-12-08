use crate::{color::Color, geometry::Hit, ray::Ray};

use super::{random_in_unit_sphere, reflect, Material};

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)> {
        let reflected = reflect(hit.ray.velocity.normalize_unchecked(), hit.normal);
        let scattered = Ray::new(hit.point, reflected + self.fuzz * random_in_unit_sphere());
        Some((scattered, self.albedo))
    }
}
