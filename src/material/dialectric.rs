use rand::Rng;

use crate::{color::Color, geometry::Hit, ray::Ray};

use super::{reflect, refract, Material};

pub struct Dialectric {
    pub index: f64,
}

impl Dialectric {
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dialectric {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)> {
        let attenuation = Color::from_rgb(1.0, 1.0, 1.0);
        let refraction_ratio = if hit.front_face {
            1.0 / self.index
        } else {
            self.index
        };

        let unit_vel = hit.ray.velocity.normalize_unchecked();

        let cos_theta = (-unit_vel).dot(hit.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let can_refract = refraction_ratio * sin_theta > 1.0
            || Self::reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen();

        let dir = if can_refract {
            reflect(unit_vel, hit.normal)
        } else {
            refract(unit_vel, hit.normal, cos_theta, refraction_ratio)
        };

        Some((Ray::new(hit.point, dir), attenuation))
    }
}
