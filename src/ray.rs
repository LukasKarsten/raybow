use crate::vector::Vector;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector,
    pub direction: Vector,
}

impl Ray {
    pub fn new(origin: Vector, velocity: Vector) -> Self {
        Self {
            origin,
            direction: velocity.normalize_unchecked(),
        }
    }

    pub fn at(&self, t: f32) -> Vector {
        self.origin + self.direction * t
    }
}
