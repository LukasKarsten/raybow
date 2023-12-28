use crate::{color::Color, geometry::Hit, ray::Ray, raybow::WorkerState};

use super::{random_unit_vector, Material, MaterialHitResult};

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn hit(&self, hit: &Hit, state: &mut WorkerState) -> MaterialHitResult {
        let mut scatter_dir = hit.normal + random_unit_vector(state);

        if scatter_dir.is_almost_zero() {
            scatter_dir = hit.normal;
        }

        let scattered = Ray::new(hit.point, scatter_dir);
        MaterialHitResult::reflecting(scattered, self.albedo)
    }
}
