use std::f32::consts::TAU;

use crate::{ray::Ray, vector::Vector};

pub struct Camera {
    origin: Vector,
    lower_left_corner: Vector,
    horizontal: Vector,
    vertical: Vector,
    u: Vector,
    v: Vector,
    lens_radius: f32,
}

impl Camera {
    pub fn new(
        lookfrom: Vector,
        lookat: Vector,
        vup: Vector,
        vfov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();

        let vp_height = 2.0 * h;
        let vp_width = aspect_ratio * vp_height;

        let w = (lookfrom - lookat).normalize_unchecked();
        let u = vup.cross3(w).normalize_unchecked();
        let v = w.cross3(u);

        let horizontal = focus_dist * vp_width * u;
        let vertical = focus_dist * vp_height * v;

        Self {
            origin: lookfrom,
            horizontal,
            vertical,
            lower_left_corner: lookfrom - horizontal / 2.0 - vertical / 2.0 - focus_dist * w,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk();
        let offset = self.u * rd.x() + self.v * rd.y();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

fn random_in_unit_disk() -> Vector {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let theta = rng.gen_range(0.0..TAU);

    Vector::from_xyz(theta.sin(), theta.cos(), 0.0) * rng.gen::<f32>()
}
