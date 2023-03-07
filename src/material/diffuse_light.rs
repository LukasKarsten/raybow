use crate::{geometry::Hit, Color, RayState};

use super::{Material, MaterialHitResult};

pub struct DiffuseLight {
    pub emit: Color,
}

impl Material for DiffuseLight {
    fn hit(&self, _hit: &Hit, _state: &RayState) -> MaterialHitResult {
        MaterialHitResult::emitting(self.emit)
    }
}