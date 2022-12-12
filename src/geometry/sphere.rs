use std::{ops::Range, sync::Arc};

use crate::{material::Material, ray::Ray, vector::Vector};

use super::{Aabb, Hit, Hittable};

pub struct Sphere {
    center: Vector,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vector, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit> {
        use std::arch::x86_64::*;

        #[repr(C, align(16))]
        struct Align16<T>(T);

        let halfb_halfb_c_a_vector;
        let mut discriminant = Align16(0.0);

        unsafe {
            let origin = ray.origin.to_simd();
            let velocity = ray.velocity.to_simd();
            let center = self.center.to_simd();

            let oc = _mm256_sub_pd(origin, center);

            let vel_vel = _mm256_mul_pd(velocity, velocity);
            let oc_oc = _mm256_mul_pd(oc, oc);
            let oc_vel = _mm256_mul_pd(oc, velocity);

            // vel_x^2 + vel_y^2 | oc_x^2 + oc_y^2 | vel_z^2 + vel_w^2 | oc_z^2 + oc_w^2
            // lsum(vel^2) | lsum(oc^2) | hsum(vel^2) | hsum(oc^2)
            let hadd_vel_vel_oc_oc = _mm256_hadd_pd(vel_vel, oc_oc);
            // oc_x * vel_x + oc-y * vel_y | oc_x * vel_x + oc-y * vel_y | oc_z * vel_z + oc_w * vel_w | oc_z * vel_z + oc_w * vel_w
            // lsum(oc*vel) | lsum(oc*vel) | hsum(oc*vel) | hsum(oc*vel)
            let hadd_oc_vel = _mm256_hadd_pd(oc_vel, oc_vel);

            // lsum(oc*vel) | lsum(oc*vel) | hsum(oc^2) | hsum(vel^2)
            let swapped = _mm256_permute2f128_pd::<0x21>(hadd_vel_vel_oc_oc, hadd_oc_vel);
            // hsum(oc*vel) | hsum(oc*vel) | lsum(oc^2) | lsum(vel^2)
            let blended = _mm256_blend_pd::<0b1100>(hadd_vel_vel_oc_oc, hadd_oc_vel);

            // halfb | halfb | |oc|^2 | a
            let dots = _mm256_add_pd(swapped, blended);

            let radius = _mm256_set_pd(0.0, 0.0, self.radius, 0.0);
            let halfb_halfb_c_a = _mm256_fnmadd_pd(radius, radius, dots);
            let halfb_halfb = _mm256_extractf128_pd::<1>(halfb_halfb_c_a);

            halfb_halfb_c_a_vector = Vector::from_simd(halfb_halfb_c_a);

            let halfb_halfb_a_c = _mm256_permute_pd(halfb_halfb_c_a, 0b1001);

            let halfbsqr_halfbsqr_ac_ac = _mm256_mul_pd(halfb_halfb_c_a, halfb_halfb_a_c);
            let halfbsqr_halfbsqr = _mm_mul_pd(halfb_halfb, halfb_halfb);

            let discriminants = _mm_sub_pd(
                halfbsqr_halfbsqr,
                _mm256_castpd256_pd128(halfbsqr_halfbsqr_ac_ac),
            );
            _mm_store_pd1(&mut discriminant.0, discriminants);
        }

        let [a, _, half_b, _] = halfb_halfb_c_a_vector.0;
        let discriminant = discriminant.0;

        /*
        {
            let _oc = ray.origin - self.center;
            let _a = ray.velocity.length_squared();
            let _half_b = _oc.dot(ray.velocity);
            let _c = _oc.length_squared() - self.radius * self.radius;
            let _discriminant = _half_b * _half_b - _a * _c;

            assert_eq!(a, _a);
            assert_eq!(half_b, _half_b);
            assert_eq!(c, _c);
            assert_eq!(discriminant, _discriminant);
        }
        */

        if discriminant < 0.0 {
            return None;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if t_range.start > root || t_range.end < root {
            root = (-half_b + sqrtd) / a;
            if t_range.start > root || t_range.end < root {
                return None;
            }
        }

        let point = ray.at(root);

        Some(Hit::new(
            point,
            (point - self.center) / self.radius,
            ray,
            root,
            Arc::clone(&self.material),
        ))
    }

    fn bounding_box(&self) -> Aabb {
        Aabb {
            minimum: self.center - Vector::from_xyz(self.radius, self.radius, self.radius),
            maximum: self.center + Vector::from_xyz(self.radius, self.radius, self.radius),
        }
    }
}
