use crate::{Color, geometry::Hit, raybow::WorkerState};

use super::{Material, MaterialHitResult};

pub struct DiffuseLight {
    pub emit: Color,
}

impl Material for DiffuseLight {
    fn hit(&self, _hit: &Hit, _state: &mut WorkerState) -> MaterialHitResult {
        MaterialHitResult::emitting(self.emit)
    }
}
