use std::{mem::MaybeUninit, ops::Range, sync::Arc};

use crate::{material::Material, ray::Ray, vector::Vector};

use super::{Geometry, Hit};

pub struct Sphere {
    center: Vector,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vector, radius: f64, material: Arc<dyn Material>) -> Self {
        assert!(radius > 0.0, "radius must be greater than 0");
        Self {
            center,
            radius,
            material,
        }
    }
}

#[repr(C, align(32))]
struct F64x4([f64; 4]);

#[repr(C, align(16))]
struct F64Aligned16(f64);

impl Geometry for Sphere {
    fn hit(&self, ray: Ray, t_range: Range<f64>) -> Option<Hit> {
        use std::arch::x86_64::*;

        let radius_squared = self.radius * self.radius;

        let a_c_halfb_halfb_vector = MaybeUninit::<F64x4>::uninit();
        let discriminant = MaybeUninit::<F64Aligned16>::uninit();

        unsafe {
            let origin = ray.origin.to_simd();
            let velocity = ray.velocity.to_simd();
            let center = self.center.to_simd();

            let oc = _mm256_sub_pd(origin, center);

            ///////////////////////////////////////////////////////////////////////////////////////////////////////////

            // dot(velocity, velocity)
            // dot(oc, velocity)
            // dot(oc, oc)
            let vel_vel = _mm256_mul_pd(velocity, velocity);
            let oc_oc = _mm256_mul_pd(oc, oc);
            let oc_vel = _mm256_mul_pd(oc, velocity);

            // lsum(vel*vel) | lsum(oc*oc) | hsum(vel*vel) | hsum(oc*oc)
            // ~~hsum(oc*oc) | hsum(vel*vel) | lsum(oc*oc) | lsum(vel*vel)~~
            let hadd_vel_vel_oc_oc = _mm256_hadd_pd(vel_vel, oc_oc);
            // lsum(oc*vel) | lsum(oc*vel) | hsum(oc*vel) | hsum(oc*vel)
            // ~~hsum(oc*vel) | hsum(oc*vel) | lsum(oc*vel) | lsum(oc*vel)~~
            let hadd_oc_vel = _mm256_hadd_pd(oc_vel, oc_vel);

            // hsum(oc*vel) | hsum(oc*vel) | lsum(vel*vel) | lsum(oc*oc)
            // ~~lsum(oc*vel) | lsum(oc*vel) | hsum(oc*oc) | hsum(vel*vel)~~
            let swapped = _mm256_permute2f128_pd::<0x21>(hadd_vel_vel_oc_oc, hadd_oc_vel);
            // lsum(oc*vel) | lsum(oc*vel) | hsum(vel*vel) | hsum(oc*oc)
            // ~~hsum(oc*vel) | hsum(oc*vel) | lsum(oc*oc) | lsum(vel*vel)~~
            let blended = _mm256_blend_pd::<0b1100>(hadd_vel_vel_oc_oc, hadd_oc_vel);

            // oc . vel | oc . vel | oc . oc | vel . vel
            // oc . oc | vel . vel | oc . vel | oc . vel
            let sum = _mm256_add_pd(swapped, blended);

            ///////////////////////////////////////////////////////////////////////////////////////////////////////////

            let radius = _mm256_set_pd(0.0, 0.0, radius_squared, 0.0);

            let a_c_halfb_halfb = _mm256_sub_pd(sum, radius);

            _mm256_store_pd(a_c_halfb_halfb_vector.as_ptr() as _, a_c_halfb_halfb);

            let c_a_halfb_halfb = _mm256_permute_pd(a_c_halfb_halfb, 0b1001);

            // ac | ac | halfb^2 | halfb^2
            let tmp1 = _mm256_mul_pd(a_c_halfb_halfb, c_a_halfb_halfb);
            // halfb^2 | halfb^2
            let tmp2 = _mm256_extractf128_pd::<1>(tmp1);
            // halfb^2 - ac | halfb^2 - ac
            let result = _mm_sub_pd(tmp2, _mm256_castpd256_pd128(tmp1));

            _mm_store_pd1(discriminant.as_ptr() as _, result);
        }

        let [a, _, half_b, _] = unsafe { a_c_halfb_halfb_vector.assume_init() }.0;
        let discriminant = unsafe { discriminant.assume_init().0 };

        /*
        {
            let _oc = ray.origin - self.center;
            let _a = ray.velocity.magnitude_squared();
            let _half_b = _oc.dot(&ray.velocity);
            let _c = _oc.magnitude_squared() - self.radius * self.radius;
            let _discriminant = _half_b * _half_b - _a * _c;

            assert_eq!(half_b, _half_b);
            assert_eq!(a, _a);
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

        Some(Hit {
            material: Arc::clone(&self.material),
            ray,
            point,
            normal: (point - self.center) / self.radius,
            t: root,
        })
    }
}
