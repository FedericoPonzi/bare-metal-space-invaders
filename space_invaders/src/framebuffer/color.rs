#[derive(Debug, Copy, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    brga: u32,
}

impl From<u32> for Color {
    #[inline(always)]
    fn from(brga: u32) -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            brga,
        }
    }
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self::new_alpha(red, green, blue, 0x00)
    }
    const fn new_alpha(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            brga: Self::brga_u32(alpha, red, green, blue),
        }
    }
    pub fn as_rgb_u32(&self) -> u32 {
        (255 << 28 | (self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
    }
    pub const fn brga_u32(alpha: u8, red: u8, green: u8, blue: u8) -> u32 {
        ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
    }

    pub fn as_brga_u32(&self) -> u32 {
        self.brga
    }
}

// pub const BLACK_COLOR: Color = Color::new(0, 0, 0);

pub const WHITE_COLOR: Color = Color::new(255, 255, 255);

pub const SHOT_COLOR: Color = Color::new(252, 186, 3);
