#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Color {
    pub(crate) rgb: u32,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            rgb: Self::rgb_u32(red, green, blue),
        }
    }

    const fn rgb_u32(red: u8, green: u8, blue: u8) -> u32 {
        (255 << 28 | (red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
    }
    // inlined to increase performance by 5~ ms per loop
    #[inline(always)]
    pub fn rgb(&self) -> u32 {
        self.rgb
    }
}

// pub const BLACK_COLOR: Color = Color::new(0, 0, 0);

pub const WHITE_COLOR: Color = Color::new(255, 255, 255);
pub const BLUE_COLOR: Color = Color::new(0, 0, 255);
pub const SHOT_COLOR: Color = Color::new(252, 186, 3);
