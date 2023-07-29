use std::f32::consts::TAU;

use crate::{
    ray::Ray,
    raybow::{RayState, RngKey},
    vector::Vector,
};

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
        mut lookat: Vector,
        mut vup: Vector,
        vfov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();

        let vp_height = 2.0 * h;
        let vp_width = aspect_ratio * vp_height;

        if (lookat - lookfrom).is_almost_zero() {
            lookat = lookat + Vector::from_xyz(0.0, 0.0, -1.0);
        }
        if vup.is_almost_zero() {
            vup = Vector::from_xyz(0.0, 1.0, 0.0);
        }

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

    pub fn get_ray(&self, s: f32, t: f32, state: &RayState) -> Ray {
        let rd = self.lens_radius * random_lens_position(state);
        let offset = self.u * rd.x() + self.v * rd.y();

        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}

fn random_lens_position(state: &RayState) -> Vector {
    let [angle, len, ..] = state.gen_random_floats(RngKey::CameraLensPosition);

    let theta = angle * TAU;

    Vector::from_xyz(theta.sin(), theta.cos(), 0.0) * len
}
