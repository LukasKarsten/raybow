use crate::{ray::Ray, Vector};

pub struct Camera {
    origin: Vector,
    vp_width: f64,
    vp_height: f64,
    focal_length: f64,
}

impl Camera {
    pub fn new(
        origin: Vector,
        viewport_width: f64,
        viewport_height: f64,
        focal_length: f64,
    ) -> Self {
        Self {
            origin,
            vp_width: viewport_width,
            vp_height: viewport_height,
            focal_length,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let x = (u - 0.5) * self.vp_width;
        let y = (v - 0.5) * self.vp_height;
        let z = -self.focal_length;
        Ray::new(self.origin, Vector::from_xyz(x, y, z))
    }
}

/*
pub struct Camera {
    origin: Point,
    lower_left_corner: Point,
    horizontal: Vector,
    vertical: Vector,
}

impl Camera {
    pub fn new(
        origin: Point,
        viewport_width: f64,
        viewport_height: f64,
        focal_length: f64,
    ) -> Self {
        let horizontal = Vector::new(viewport_width, 0.0, 0.0);
        let vertical = Vector::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - horizontal * 0.5 - vertical * 0.5 - Vector::new(0.0, 0.0, focal_length);

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
*/
