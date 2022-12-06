use crate::vector::Vector;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector,
    pub velocity: Vector,
}

impl Ray {
    pub fn new(origin: Vector, velocity: Vector) -> Self {
        Self { origin, velocity }
    }

    pub fn at(&self, t: f64) -> Vector {
        self.origin + self.velocity * t
    }
}
