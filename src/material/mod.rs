use rand::distributions::Uniform;

use crate::{color::Color, geometry::Hit, ray::Ray, Vector};

pub trait Material: Send + Sync {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)> {
        let mut scatter_dir = hit.normal_opposite_to_ray() + random_unit_vector();

        if scatter_dir.is_almost_zero() {
            scatter_dir = hit.normal_opposite_to_ray();
        }

        let scattered = Ray::new(hit.point, scatter_dir);
        Some((scattered, self.albedo))
    }
}

fn random_unit_vector() -> Vector {
    random_in_unit_sphere().normalize()
}

fn random_in_unit_sphere() -> Vector {
    loop {
        let mut rng = rand::thread_rng();
        let dist = Uniform::new_inclusive(-1.0, 1.0);
        let v = Vector::from_distribution(&dist, &mut rng);
        if v.length_squared() < 1.0 {
            return v;
        }
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, hit: &Hit) -> Option<(Ray, Color)> {
        let reflected = reflect(hit.ray.velocity.normalize(), hit.normal_opposite_to_ray());
        let scattered = Ray::new(hit.point, reflected + self.fuzz * random_in_unit_sphere());
        Some((scattered, self.albedo))
    }
}

fn reflect(v: Vector, n: Vector) -> Vector {
    v - 2.0 * v.dot(n) * n
}
