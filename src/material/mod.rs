use std::f64::consts::TAU;

use rand::Rng;

use crate::{color::Color, geometry::Hit, ray::Ray, Vector};

fn random_unit_vector() -> Vector {
    // https://math.stackexchange.com/a/44691
    let mut rng = rand::thread_rng();
    let theta = rng.gen_range(0.0..TAU);
    let z = rng.gen_range(-1.0..1.0);

    let tmp = (1.0f64 - z * z).sqrt();

    let x = theta.cos() * tmp;
    let y = theta.sin() * tmp;

    Vector::from_xyz(x, y, z)
}

fn random_in_unit_sphere() -> Vector {
    random_unit_vector() * rand::thread_rng().gen::<f64>()
}

fn refract(uv: Vector, n: Vector, cos_theta: f64, etai_over_etat: f64) -> Vector {
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = (-(1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}

pub trait Material: Send + Sync {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)>;
}

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

fn reflect(v: Vector, n: Vector) -> Vector {
    v - 2.0 * v.dot(n) * n
}

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
