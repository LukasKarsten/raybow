use std::{
    fmt::{self, Write},
    ops::{Add, Deref, DerefMut, Div, Index, Mul, Neg, Sub},
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

    pub fn dot(self, other: Self) -> f32 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z() + self.w() * other.w()
    }

    pub fn cross3(self, other: Self) -> Self {
        let x = self.y() * other.z() - self.z() * other.y();
        let y = self.z() * other.x() - self.x() * other.z();
        let z = self.x() * other.y() - self.y() * other.x();

        Self::from_xyzw(x, y, z, self.w())
    }

    pub fn is_almost_zero(self) -> bool {
        self.0.into_iter().all(|v| v.abs() < 1e-8)
    }

    pub fn min(self, other: Self) -> Self {
        let x = self.x().min(other.x());
        let y = self.y().min(other.y());
        let z = self.z().min(other.z());
        let w = self.w().min(other.w());
        Self::from_xyzw(x, y, z, w)
    }

    pub fn max(self, other: Self) -> Self {
        let x = self.x().max(other.x());
        let y = self.y().max(other.y());
        let z = self.z().max(other.z());
        let w = self.w().max(other.w());
        Self::from_xyzw(x, y, z, w)
    }

    pub fn abs(self) -> Self {
        Self(self.0.map(|v| v.abs()))
    }

    pub fn largest_axis(self) -> Dimension {
        let [x, y, z, _] = self.0;
        if x > y && x > z {
            Dimension::X
        } else if y > z {
            Dimension::Y
        } else {
            Dimension::Z
        }
    }
}

impl From<Vector> for [f32; 3] {
    fn from(vec: Vector) -> Self {
        let [x, y, z, _] = vec.0;
        [x, y, z]
    }
}

impl From<[f32; 3]> for Vector {
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self::from_xyz(x, y, z)
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
        let x = self.x() + rhs.x();
        let y = self.y() + rhs.y();
        let z = self.z() + rhs.z();
        let w = self.w() + rhs.w();
        Self::from_xyzw(x, y, z, w)
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
        let x = self.x() - rhs.x();
        let y = self.y() - rhs.y();
        let z = self.z() - rhs.z();
        let w = self.w() - rhs.w();
        Self::from_xyzw(x, y, z, w)
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        let z = self.z() * rhs;
        let w = self.w() * rhs;
        Self::from_xyzw(x, y, z, w)
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
        let x = self.x() * rhs.x();
        let y = self.y() * rhs.y();
        let z = self.z() * rhs.z();
        let w = self.w() * rhs.w();
        Self::from_xyzw(x, y, z, w)
    }
}

impl Div<Vector> for f32 {
    type Output = Vector;

    fn div(self, rhs: Vector) -> Self::Output {
        let x = self / rhs.x();
        let y = self / rhs.y();
        let z = self / rhs.z();
        let w = self / rhs.w();
        Vector::from_xyzw(x, y, z, w)
    }
}

impl Div<f32> for Vector {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let x = self.x() / rhs;
        let y = self.y() / rhs;
        let z = self.z() / rhs;
        let w = self.w() / rhs;
        Self::from_xyzw(x, y, z, w)
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let x = -self.x();
        let y = -self.y();
        let z = -self.z();
        let w = -self.w();
        Self::from_xyzw(x, y, z, w)
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

#[derive(Default, Clone, Copy, PartialEq)]
#[repr(C, align(64))]
struct F32x16([f32; 16]);

impl Deref for F32x16 {
    type Target = [f32; 16];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for F32x16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Vector3x16 {
    x: F32x16,
    y: F32x16,
    z: F32x16,
}

impl Vector3x16 {
    pub const ZERO: Self = Self {
        x: F32x16([0.0; 16]),
        y: F32x16([0.0; 16]),
        z: F32x16([0.0; 16]),
    };

    pub fn set_vec(&mut self, idx: usize, vec: [f32; 3]) {
        self.x[idx] = vec[0];
        self.y[idx] = vec[1];
        self.z[idx] = vec[2];
    }

    pub fn get_vec(&self, idx: usize) -> [f32; 3] {
        assert!(idx < 16);
        [self.x[idx], self.y[idx], self.z[idx]]
    }

    pub fn x(&self) -> &[f32; 16] {
        &self.x
    }

    pub fn y(&self) -> &[f32; 16] {
        &self.y
    }

    pub fn z(&self) -> &[f32; 16] {
        &self.z
    }
}

impl fmt::Debug for Vector3x16 {
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
