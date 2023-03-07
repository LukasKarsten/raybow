use std::f32::consts::TAU;

use crate::{color::Color, geometry::Hit, ray::Ray, vector::Vector, RayState, RngKey};

pub use dialectric::Dialectric;
pub use diffuse_light::DiffuseLight;
pub use lambertian::Lambertian;
pub use metal::Metal;

mod dialectric;
mod diffuse_light;
mod lambertian;
mod metal;

pub trait Material: Send + Sync {
    fn hit(&self, hit: &Hit, state: &RayState) -> MaterialHitResult;
}

pub struct Reflection {
    pub ray: Ray,
    pub attenuation: Color,
}

pub struct MaterialHitResult {
    pub reflection: Option<Reflection>,
    pub emission: Color,
}

impl MaterialHitResult {
    fn reflecting(ray: Ray, attenuation: Color) -> Self {
        Self {
            reflection: Some(Reflection { ray, attenuation }),
            emission: Color::BLACK,
        }
    }

    fn emitting(emission: Color) -> Self {
        Self {
            reflection: None,
            emission,
        }
    }
}

// https://math.stackexchange.com/a/44691
fn unit_vector_from_cylinder(angle: f32, z: f32) -> Vector {
    let theta = angle * TAU;
    let tmp = (1.0f32 - z * z).sqrt();

    let x = theta.cos() * tmp;
    let y = theta.sin() * tmp;

    Vector::from_xyz(x, y, z)
}

fn random_unit_vector(state: &RayState, rng_key: RngKey) -> Vector {
    let [angle, z, ..] = state.gen_random_floats(rng_key);
    unit_vector_from_cylinder(angle, -1.0 + z * 2.0)
}

fn random_in_unit_sphere(state: &RayState, rng_key: RngKey) -> Vector {
    let [angle, z, len, ..] = state.gen_random_floats(rng_key);
    unit_vector_from_cylinder(angle, -1.0 + z * 2.0) * len
}

fn reflect(v: Vector, n: Vector) -> Vector {
    v - 2.0 * v.dot(n) * n
}

fn refract(uv: Vector, n: Vector, cos_theta: f32, etai_over_etat: f32) -> Vector {
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = (-(1.0 - r_out_perp.length_squared()).abs().sqrt()) * n;
    r_out_perp + r_out_parallel
}
