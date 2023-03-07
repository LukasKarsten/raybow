use std::{
    arch::x86_64::*,
    mem::MaybeUninit,
    ops::{Add, Div, Index, Mul, Neg, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Dimension {
    X = 0,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(16))]
pub struct Vector(pub [f32; 4]);

impl Vector {
    pub const ZERO: Self = Self([0.0; 4]);

    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_xyzw(x, y, z, 0.0)
    }

    pub const fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }

    pub fn x(self) -> f32 {
        self.0[0]
    }

    pub fn y(self) -> f32 {
        self.0[1]
    }

    pub fn z(self) -> f32 {
        self.0[2]
    }

    pub fn w(self) -> f32 {
        self.0[3]
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalize_unchecked(self) -> Self {
        self / self.length()
    }

    #[cfg(feature = "simd")]
    pub fn dot(self, other: Self) -> f32 {
        unsafe {
            let dp = _mm_dp_ps::<0xF1>(self.to_simd(), other.to_simd());
            f32::from_bits(_mm_extract_ps::<0>(dp) as u32)
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn dot(self, other: Self) -> f32 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z() + self.w() * other.w()
    }

    pub fn cross3(self, other: Self) -> Self {
        let x = self.y() * other.z() - self.z() * other.y();
        let y = self.z() * other.x() - self.x() * other.z();
        let z = self.x() * other.y() - self.y() * other.x();

        Self::from_xyzw(x, y, z, self.w())
    }

    #[cfg(feature = "simd")]
    pub fn is_almost_zero(self) -> bool {
        unsafe {
            let this = self.to_simd();

            let epsilon = _mm_set1_ps(1e-8);
            let mask = _mm_set1_ps(-0.0);

            let abs = _mm_andnot_ps(mask, this);
            let result = _mm_movemask_ps(_mm_cmp_ps::<_CMP_LT_OQ>(abs, epsilon));

            result == 0b1111
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn is_almost_zero(self) -> bool {
        self.0.into_iter().all(|v| v.abs() < 1e-8)
    }

    #[cfg(feature = "simd")]
    pub fn min(self, other: Self) -> Self {
        unsafe { Self::from_simd(_mm_min_ps(self.to_simd(), other.to_simd())) }
    }

    #[cfg(not(feature = "simd"))]
    pub fn min(self, other: Self) -> Self {
        let x = self.x().min(other.x());
        let y = self.y().min(other.y());
        let z = self.z().min(other.z());
        let w = self.w().min(other.w());
        Self::from_xyzw(x, y, z, w)
    }

    #[cfg(feature = "simd")]
    pub fn min_elem(self) -> f32 {
        unsafe {
            let wzyx = self.to_simd();
            let yxwz = _mm_permute_ps::<0b01_00_11_10>(wzyx);

            let wy_zx_wy_zx = _mm_min_ps(wzyx, yxwz);

            let zx_wy_zx_wy = _mm_permute_ps::<0b10_11_00_01>(wy_zx_wy_zx);

            let wzyx_min = _mm_min_ps(wy_zx_wy_zx, zx_wy_zx_wy);

            f32::from_bits(_mm_extract_ps::<0>(wzyx_min) as u32)
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn min_elem(self) -> f32 {
        self.x().min(self.y()).min(self.z()).min(self.w())
    }

    #[cfg(feature = "simd")]
    pub fn max(self, other: Self) -> Self {
        unsafe { Self::from_simd(_mm_max_ps(self.to_simd(), other.to_simd())) }
    }

    #[cfg(not(feature = "simd"))]
    pub fn max(self, other: Self) -> Self {
        let x = self.x().max(other.x());
        let y = self.y().max(other.y());
        let z = self.z().max(other.z());
        let w = self.w().max(other.w());
        Self::from_xyzw(x, y, z, w)
    }

    #[cfg(feature = "simd")]
    pub fn max_elem(self) -> f32 {
        unsafe {
            let wzyx = self.to_simd();
            let yxwz = _mm_permute_ps::<0b01_00_11_10>(wzyx);

            let wy_zx_wy_zx = _mm_max_ps(wzyx, yxwz);

            let zx_wy_zx_wy = _mm_permute_ps::<0b10_11_00_01>(wy_zx_wy_zx);

            let wzyx_min = _mm_max_ps(wy_zx_wy_zx, zx_wy_zx_wy);

            f32::from_bits(_mm_extract_ps::<0>(wzyx_min) as u32)
        }
    }

    #[cfg(not(feature = "simd"))]
    pub fn max_elem(self) -> f32 {
        self.x().max(self.y()).max(self.z()).max(self.w())
    }

    pub fn sum(self) -> f32 {
        self.0.into_iter().sum()
    }

    pub fn product3(self) -> f32 {
        let [x, y, z, _] = self.0;
        x * y * z
    }

    #[cfg(feature = "simd")]
    pub fn reciprocal(self) -> Self {
        unsafe { Self::from_simd(_mm_rcp_ps(self.to_simd())) }
    }

    #[cfg(not(feature = "simd"))]
    pub fn reciprocal(self) -> Self {
        Self(self.0.map(|v| 1.0 / v))
    }

    pub fn to_simd(&self) -> __m128 {
        unsafe { _mm_load_ps(&self.0 as _) }
    }

    pub fn from_simd(vec: __m128) -> Self {
        unsafe {
            let mut data = MaybeUninit::uninit();
            _mm_store_ps(data.as_mut_ptr() as _, vec);
            data.assume_init()
        }
    }
}

impl From<Vector> for [f32; 3] {
    fn from(vec: Vector) -> Self {
        let [x, y, z, _] = vec.0;
        [x, y, z]
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm_add_ps(self.to_simd(), rhs.to_simd())) }
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm_sub_ps(self.to_simd(), rhs.to_simd())) }
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        unsafe { Self::from_simd(_mm_mul_ps(self.to_simd(), _mm_set1_ps(rhs))) }
    }
}

impl Mul<Vector> for f32 {
    type Output = <Vector as Mul<Self>>::Output;

    fn mul(self, rhs: Vector) -> Self::Output {
        rhs * self
    }
}

impl Mul<Vector> for Vector {
    type Output = Self;

    fn mul(self, rhs: Vector) -> Self::Output {
        unsafe { Self::from_simd(_mm_mul_ps(self.to_simd(), rhs.to_simd())) }
    }
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        unsafe { Self::from_simd(_mm_div_ps(self.to_simd(), _mm_set1_ps(rhs))) }
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        unsafe {
            let this = self.to_simd();
            let negated = _mm_xor_ps(this, _mm_set1_ps(-0.0));
            Self::from_simd(negated)
        }
    }
}

impl Index<usize> for Vector {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Index<Dimension> for Vector {
    type Output = f32;

    fn index(&self, index: Dimension) -> &Self::Output {
        &self.0[usize::from(index as u8)]
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
