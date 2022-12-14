use std::{
    arch::x86_64::*,
    fmt::{self, Write},
    mem::MaybeUninit,
    ops::{Add, Deref, DerefMut, Div, Index, Mul, Neg, Sub},
};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C, align(32))]
pub struct Vector(pub [f32; 4]);

#[derive(Default)]
#[repr(C, align(32))]
struct Align32<T>(T);

impl<T> Deref for Align32<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Align32<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

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

    pub fn dot(self, other: Self) -> f32 {
        unsafe {
            let dp = _mm_dp_ps::<0xF1>(self.to_simd(), other.to_simd());
            f32::from_bits(_mm_extract_ps::<0>(dp) as u32)
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

            let epsilon = _mm_set1_ps(epsilon);

            let negative1 = _mm_set1_epi32(-1);
            let mask = _mm_castsi128_ps(_mm_srli_epi32::<1>(negative1));

            let abs = _mm_and_ps(this, mask);

            let result = _mm_movemask_ps(_mm_cmp_ps::<_CMP_LT_OQ>(abs, epsilon));

            result == 0b1111
        }
    }

    pub fn min(self, other: Self) -> Self {
        unsafe { Self::from_simd(_mm_min_ps(self.to_simd(), other.to_simd())) }
    }

    pub fn max(self, other: Self) -> Self {
        unsafe { Self::from_simd(_mm_max_ps(self.to_simd(), other.to_simd())) }
    }

    pub fn sum(self) -> f32 {
        self.0.into_iter().sum()
    }

    pub fn product3(self) -> f32 {
        let [x, y, z, _] = self.0;
        x * y * z
    }

    pub fn reciprocal(self) -> Self {
        unsafe { Self::from_simd(_mm_rcp_ps(self.to_simd())) }
    }

    pub fn to_simd(&self) -> __m128 {
        unsafe { _mm_load_ps(&self.0 as _) }
    }

    pub fn from_simd(vec: __m128) -> Self {
        unsafe {
            let data = MaybeUninit::uninit();
            _mm_store_ps(data.as_ptr() as _, vec);
            data.assume_init()
        }
    }
}

impl Into<[f32; 3]> for Vector {
    fn into(self) -> [f32; 3] {
        [self[0], self[1], self[2]]
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

impl Sub<Vector> for Vector3x8 {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self {
            x: F32x8(self.x.map(|a| a / rhs.x())),
            y: F32x8(self.y.map(|a| a / rhs.y())),
            z: F32x8(self.z.map(|a| a / rhs.z())),
        }
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

#[derive(Default, Clone, Copy, PartialEq)]
#[repr(C, align(32))]
struct F32x8([f32; 8]);

impl Deref for F32x8 {
    type Target = [f32; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for F32x8 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Vector3x8 {
    x: F32x8,
    y: F32x8,
    z: F32x8,
}

impl Vector3x8 {
    pub fn new(x: [f32; 8], y: [f32; 8], z: [f32; 8]) -> Self {
        Self {
            x: F32x8(x),
            y: F32x8(y),
            z: F32x8(z),
        }
    }

    pub fn from_vecs(vecs: [[f32; 3]; 8]) -> Self {
        let mut x = [0.0; 8];
        let mut y = [0.0; 8];
        let mut z = [0.0; 8];
        for i in 0..8 {
            x[i] = vecs[i][0];
            y[i] = vecs[i][1];
            z[i] = vecs[i][2];
        }
        Self::new(x, y, z)
    }

    pub fn set_vec(&mut self, idx: usize, vec: [f32; 3]) {
        self.x[idx] = vec[0];
        self.y[idx] = vec[1];
        self.z[idx] = vec[2];
    }

    pub fn cross(&self, other: &Self) -> Self {
        let mut cross = Self::default();

        for i in 0..8 {
            cross.x[i] = self.y[i] * other.z[i] - self.z[i] * other.y[i];
            cross.y[i] = self.z[i] * other.x[i] - self.x[i] * other.z[i];
            cross.z[i] = self.x[i] * other.y[i] - self.y[i] * other.x[i];
        }

        cross
    }

    pub fn dot(&self, other: &Self) -> [f32; 8] {
        let x = self.x.iter().zip(other.x.iter()).map(|(a, b)| a * b);
        let y = self.y.iter().zip(other.y.iter()).map(|(a, b)| a * b);
        let z = self.z.iter().zip(other.z.iter()).map(|(a, b)| a * b);

        let mut dot = [0.0; 8];
        for (i, (x, (y, z))) in x.zip(y.zip(z)).enumerate() {
            dot[i] = x + y + z;
        }

        dot
    }

    pub fn magnitude_squared(&self) -> [f32; 8] {
        self.dot(self)
    }

    pub fn magnitude(&self) -> [f32; 8] {
        self.magnitude_squared().map(f32::sqrt)
    }

    pub fn normalize_unchecked(&self) -> Self {
        let mag = self.magnitude();

        let mut result = Self::default();

        for i in 0..8 {
            result.x[i] = self.x[i] / mag[i];
            result.y[i] = self.y[i] / mag[i];
            result.z[i] = self.z[i] / mag[i];
        }

        result
    }

    fn map(&self, f: impl FnMut(f32) -> f32 + Copy) -> Self {
        Self {
            x: F32x8(self.x.map(f)),
            y: F32x8(self.y.map(f)),
            z: F32x8(self.z.map(f)),
        }
    }

    fn zip_map(&self, other: &Self, f: impl FnMut(f32, f32) -> f32 + Copy) -> Self {
        fn zip_map(a: &F32x8, b: &F32x8, mut f: impl FnMut(f32, f32) -> f32 + Copy) -> F32x8 {
            let mut result = F32x8::default();
            for i in 0..8 {
                result[i] = f(a[i], b[i]);
            }
            result
        }
        Self {
            x: zip_map(&self.x, &other.x, f),
            y: zip_map(&self.y, &other.y, f),
            z: zip_map(&self.z, &other.z, f),
        }
    }

    pub fn reciprocal(&self) -> Self {
        self.map(|a| 1.0 / a)
    }

    pub fn min(&self, other: &Self) -> Self {
        self.zip_map(other, f32::min)
    }

    pub fn max(&self, other: &Self) -> Self {
        self.zip_map(other, f32::max)
    }

    pub fn x(&self) -> &[f32; 8] {
        &self.x
    }

    pub fn y(&self) -> &[f32; 8] {
        &self.y
    }

    pub fn z(&self) -> &[f32; 8] {
        &self.z
    }
}

impl Add<Self> for Vector3x8 {
    type Output = Vector3x8;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip_map(&rhs, f32::add)
    }
}

impl Sub<Self> for Vector3x8 {
    type Output = Vector3x8;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip_map(&rhs, f32::sub)
    }
}

impl Mul<Self> for Vector3x8 {
    type Output = Vector3x8;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip_map(&rhs, f32::mul)
    }
}

impl Mul<f32> for Vector3x8 {
    type Output = Vector3x8;

    fn mul(self, rhs: f32) -> Self::Output {
        self.map(|a| a * rhs)
    }
}

impl Mul<Vector3x8> for f32 {
    type Output = <Vector3x8 as Mul<Self>>::Output;

    fn mul(self, rhs: Vector3x8) -> Self::Output {
        rhs * self
    }
}

impl Div<f32> for Vector3x8 {
    type Output = Vector3x8;

    fn div(self, rhs: f32) -> Self::Output {
        self.map(|a| a / rhs)
    }
}

impl Neg for Vector3x8 {
    type Output = Vector3x8;

    fn neg(self) -> Self::Output {
        self.map(f32::neg)
    }
}

impl From<Vector> for Vector3x8 {
    fn from(vec: Vector) -> Self {
        Self::new([vec.x(); 8], [vec.y(); 8], [vec.z(); 8])
    }
}

impl fmt::Debug for Vector3x8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Vector3x8(")?;

        for i in 0..8 {
            if f.alternate() {
                f.write_fmt(format_args!("\n    {}: [", i))?;
            } else {
                if i != 0 {
                    f.write_str(", ")?;
                }
                f.write_fmt(format_args!("{}:[", i))?;
            }
            self.x[i].fmt(f)?;
            f.write_str(", ")?;
            self.y[i].fmt(f)?;
            f.write_str(", ")?;
            self.z[i].fmt(f)?;
            f.write_char(']')?;
        }

        if f.alternate() {
            f.write_char('\n')?;
        }
        f.write_char(')')
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
