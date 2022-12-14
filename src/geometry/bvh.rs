use std::ops::Range;

use crate::ray::Ray;

use super::{Aabb, Hit, Hittable};

pub enum BvhNode {
    Branch {
        bounding_box: Aabb,
        left: Box<BvhNode>,
        right: Box<BvhNode>,
    },
    Leaf(Box<dyn Hittable>),
    Empty,
}

impl BvhNode {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn push(&mut self, new_object: Box<dyn Hittable>) {
        let mut this = std::mem::replace(self, Self::Empty);

        *self = match this {
            Self::Empty => Self::Leaf(new_object),
            Self::Leaf(obj) => Self::Branch {
                bounding_box: obj.bounding_box().merge(&new_object.bounding_box()),
                left: Box::new(BvhNode::Leaf(obj)),
                right: Box::new(BvhNode::Leaf(new_object)),
            },
            Self::Branch {
                ref mut bounding_box,
                ref mut left,
                ref mut right,
                ..
            } => {
                let new_aabb = new_object.bounding_box();
                let left_aabb = left.bounding_box();
                let right_aabb = right.bounding_box();

                let left_new_aabb = left_aabb.merge(&new_aabb);
                let right_new_aabb = right_aabb.merge(&new_aabb);

                let left_intersection = left_new_aabb.intersection(&right_aabb);
                let right_intersection = right_new_aabb.intersection(&left_aabb);
                let self_intersection = bounding_box.intersection(&new_aabb);

                if self_intersection.product3() <= 0.0 {
                    Self::Branch {
                        bounding_box: bounding_box.merge(&new_aabb),
                        left: Box::new(this),
                        right: Box::new(Self::Leaf(new_object)),
                    }
                } else {
                    // XXX: There is a bias here when both intersections are the same
                    if left_intersection.product3() < right_intersection.product3() {
                        left.push(new_object);
                    } else {
                        right.push(new_object);
                    }
                    *bounding_box = bounding_box.merge(&new_aabb);
                    this
                }
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, ray: Ray, t_range: Range<f32>) -> Option<Hit> {
        match self {
            Self::Empty => None,
            Self::Leaf(obj) => obj.hit(ray, t_range),
            Self::Branch {
                bounding_box,
                left,
                right,
            } => {
                if !bounding_box.hit(ray, t_range.clone()) {
                    return None;
                }

                let hit_left = left.hit(ray, t_range.clone());
                match hit_left {
                    Some(hit_left) => right.hit(ray, t_range.start..hit_left.t).or(Some(hit_left)),
                    None => right.hit(ray, t_range),
                }
            }
        }
    }

    fn bounding_box(&self) -> Aabb {
        match self {
            Self::Empty => Aabb::ZERO,
            Self::Leaf(obj) => obj.bounding_box(),
            Self::Branch { bounding_box, .. } => *bounding_box,
        }
    }
}
