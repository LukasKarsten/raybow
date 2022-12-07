use std::{
    arch::x86_64::*,
    mem::MaybeUninit,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(32))]
pub struct Vector(pub [f64; 4]);

#[repr(C, align(16))]
struct Align16<T>(T);

impl Vector {
    pub fn new_zero() -> Self {
        Self([0.0; 4])
    }

    pub const fn from_xyz(x: f64, y: f64, z: f64) -> Self {
        Self::from_xyzw(x, y, z, 0.0)
    }

    pub const fn from_xyzw(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self([x, y, z, w])
    }

    pub fn from_distribution(
        dist: &impl rand::distributions::Distribution<f64>,
        rng: &mut impl rand::Rng,
    ) -> Self {
        Self([
            dist.sample(rng),
            dist.sample(rng),
            dist.sample(rng),
            dist.sample(rng),
        ])
    }

    pub fn x(self) -> f64 {
        self.0[0]
    }

    pub fn y(self) -> f64 {
        self.0[1]
    }

    pub fn z(self) -> f64 {
        self.0[2]
    }

    pub fn w(self) -> f64 {
        self.0[3]
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalize_unchecked(self) -> Self {
        unsafe {
            let this = self.to_simd();

            let squared = _mm256_mul_pd(this, this);

            // x*x + y*y | x*x + y*y | z*z + w*w | z*z + w*w
            let sums = _mm256_hadd_pd(squared, squared);
            // z*z + w*w | z*z + w*w | x*x + y*y | x*x + y*y
            let sums_swapped = _mm256_permute2f128_pd::<0x01>(sums, sums);

            let length = _mm256_sqrt_pd(_mm256_add_pd(sums, sums_swapped));

            Self::from_simd(_mm256_div_pd(this, length))
        }
    }

    pub fn dot(self, other: Self) -> f64 {
        unsafe {
            // x*x | y*y | z*z | w*w
            let squared = _mm256_mul_pd(self.to_simd(), other.to_simd());
            // x*x + y*y | x*x + y*y | z*z + w*w | z*z + w*w
            let sums = _mm256_hadd_pd(squared, squared);
            // z*z + w*w | z*z + w*w
            let hi_sums = _mm256_extractf128_pd::<1>(sums);
            // x*x + y*y + z*z + w*w | x*x + y*y + z*z + w*w
            let dot_product = _mm_add_pd(_mm256_castpd256_pd128(sums), hi_sums);

            let mut result = Align16(0.0);
            _mm_store_pd1(&mut result.0, dot_product);
            result.0
        }
    }

    pub fn cross3(self, other: Self) -> Self {
        let x = self.y() * other.z() - self.z() * other.y();
        let y = self.z() * other.x() - self.x() * other.z();
        let z = self.x() * other.y() - self.y() * other.x();

        Self::from_xyzw(x, y, z, self.w())
    }

    pub fn is_almost_zero(self) -> bool {
        let epsilon = 1e-8;
        unsafe {
            let this = self.to_simd();

            let epsilon = _mm256_set1_pd(epsilon);

            let negative1 = _mm256_set1_epi64x(-1);
            let mask = _mm256_castsi256_pd(_mm256_srli_epi64::<1>(negative1));

            let abs = _mm256_and_pd(this, mask);

            // requires unstable feature "stdsimd"
            // let result = _mm256_cmp_pd_mask::<_CMP_LT_OQ>(abs, epsilon);
            let result = _mm256_movemask_pd(_mm256_cmp_pd::<_CMP_LT_OQ>(abs, epsilon));

            result == 0b1111
        }
    }

    pub fn to_simd(&self) -> __m256d {
        unsafe { _mm256_load_pd(&self.0 as _) }
    }

    pub fn from_simd(vec: __m256d) -> Self {
        unsafe {
            let data = MaybeUninit::uninit();
            _mm256_store_pd(data.as_ptr() as _, vec);
            data.assume_init()
        }
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm256_add_pd(self.to_simd(), rhs.to_simd())) }
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm256_sub_pd(self.to_simd(), rhs.to_simd())) }
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        unsafe { Self::from_simd(_mm256_mul_pd(self.to_simd(), _mm256_set1_pd(rhs))) }
    }
}

impl Mul<Vector> for f64 {
    type Output = <Vector as Mul<Self>>::Output;

    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vector> for Vector {
    type Output = Self;

    fn mul(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm256_mul_pd(self.to_simd(), rhs.to_simd())) }
    }
}

impl Div<f64> for Vector {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        unsafe { Self::from_simd(_mm256_div_pd(self.to_simd(), _mm256_set1_pd(rhs))) }
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        unsafe {
            let this = self.to_simd();
            let negated = _mm256_xor_pd(this, _mm256_set1_pd(-0.0));
            Self::from_simd(negated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Vector;

    #[test]
    fn test_is_almost_zero() {
        let v = Vector::from_xyzw(1.0, 0.0, -1.0, 0.0);
        assert!(!v.is_almost_zero());

        let v = Vector::from_xyzw(1e-9, 1e-9, 1e-9, 1e-9);
        assert!(v.is_almost_zero());
    }

    #[test]
    fn negate_vector() {
        let v = Vector::from_xyzw(1.0, 2.0, 3.0, 4.0);
        let v_neg = Vector::from_xyzw(-1.0, -2.0, -3.0, -4.0);
        assert_eq!(-v, v_neg);
    }
}
