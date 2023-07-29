use crate::vector::Vector;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub minimum: Vector,
    pub maximum: Vector,
}

impl Aabb {
    pub const ZERO: Self = Self {
        minimum: Vector::ZERO,
        maximum: Vector::ZERO,
    };

    pub fn merge(&self, other: &Self) -> Self {
        Self {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }

    pub fn surface_area(&self) -> f32 {
        let [x, y, z, _] = (self.maximum - self.minimum).0;
        2.0 * (x * y + x * z + y * z)
    }
}
