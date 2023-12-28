use crate::{color::Color, geometry::Hit, ray::Ray, raybow::WorkerState};

use super::{random_in_unit_sphere, reflect, Material, MaterialHitResult};

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f32,
}

impl Material for Metal {
    fn hit(&self, hit: &Hit, state: &mut WorkerState) -> MaterialHitResult {
        let fuzz_dir = random_in_unit_sphere(state);

        let reflected = reflect(hit.ray.direction.normalize_unchecked(), hit.normal);
        let scattered = Ray::new(hit.point, reflected + self.fuzz * fuzz_dir);
        MaterialHitResult::reflecting(scattered, self.albedo)
    }
}
