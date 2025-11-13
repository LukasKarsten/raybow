use crate::{color::Color, geometry::Hit, ray::Ray, raybow::WorkerState};

use super::{Material, MaterialHitResult, reflect, refract};

pub struct Dialectric {
    pub index: f32,
}

impl Material for Dialectric {
    fn hit(&self, hit: &Hit, state: &mut WorkerState) -> MaterialHitResult {
        let attenuation = Color::WHITE;
        let refraction_ratio = if hit.front_face {
            1.0 / self.index
        } else {
            self.index
        };

        let unit_vel = hit.ray.direction.normalize_unchecked();

        let cos_theta = (-unit_vel).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let [reflect_threshold, ..] = state.gen_random_floats();

        let cannot_refract = refraction_ratio * sin_theta > 1.0
            || reflectance(cos_theta, refraction_ratio) > reflect_threshold;

        let dir = if cannot_refract {
            reflect(unit_vel, hit.normal)
        } else {
            refract(unit_vel, hit.normal, cos_theta, refraction_ratio)
        };

        MaterialHitResult::reflecting(Ray::new(hit.point, dir), attenuation)
    }
}

fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    let r0 = r0 * r0;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
