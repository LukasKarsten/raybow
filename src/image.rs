use crate::Color;

pub struct Image {
    width: u32,
    height: u32,
    pub pixels: Box<[Color]>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
        let num_pixels = usize::try_from(width).expect("width too large")
            * usize::try_from(height).expect("height too large");

        Self {
            width,
            height,
            pixels: vec![Color::BLACK; num_pixels].into_boxed_slice(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<Color> {
        let x = usize::try_from(x).ok()?;
        let y = usize::try_from(y).ok()?;
        let width = usize::try_from(self.width).ok()?;

        let idx = x.checked_add(y.checked_mul(width)?)?;
        self.pixels.get(idx).copied()
    }

    pub fn into_srgb_8bit(self) -> Box<[u8]> {
        self.pixels
            .iter()
            .flat_map(|color| color.apply_gamma().to_rgb_bytes_8bit())
            .collect()
    }

    pub fn into_srgb_16bit(self) -> Box<[u8]> {
        self.pixels
            .iter()
            .flat_map(|color| color.apply_gamma().to_rgb_bytes_16bit())
            .collect()
    }
}
