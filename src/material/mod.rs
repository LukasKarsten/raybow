use std::f64::consts::TAU;

use rand::Rng;

use crate::{color::Color, geometry::Hit, ray::Ray, Vector};

pub use dialectric::Dialectric;
pub use lambertian::Lambertian;
pub use metal::Metal;

mod dialectric;
mod lambertian;
mod metal;

pub trait Material: Send + Sync {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)>;
}

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

fn reflect(v: Vector, n: Vector) -> Vector {
    v - 2.0 * v.dot(n) * n
}

fn refract(uv: Vector, n: Vector, cos_theta: f64, etai_over_etat: f64) -> Vector {
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = (-(1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}
