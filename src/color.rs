use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign};

use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
        }
    }

    pub fn random() -> Self {
        let [r, g, b]: [f64; 3] = rand::thread_rng().gen();
        Self { r, g, b }
    }

    pub fn to_rgb_bytes(self) -> [u8; 3] {
        let r: u8 = (self.r * 255.0).round() as u8;
        let g: u8 = (self.g * 255.0).round() as u8;
        let b: u8 = (self.b * 255.0).round() as u8;

        [r, g, b]
    }

    pub fn lerp(self, other: Color, t: f64) -> Color {
        fn lerp_f64(start: f64, end: f64, t: f64) -> f64 {
            start + t * (end - start)
        }

        Color {
            r: lerp_f64(self.r, other.r, t),
            g: lerp_f64(self.g, other.g, t),
            b: lerp_f64(self.b, other.b, t),
        }
    }
}

impl Add<Self> for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl AddAssign<Self> for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Mul<Color> for f64 {
    type Output = <Color as Mul<Self>>::Output;

    fn mul(self, rhs: Color) -> Self::Output {
        rhs * self
    }
}

impl Mul<Color> for Color {
    type Output = Self;

    fn mul(self, rhs: Color) -> Self::Output {
        Self::Output {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl MulAssign<Color> for Color {
    fn mul_assign(&mut self, rhs: Color) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
    }
}

impl Div<f64> for Color {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

impl DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}
