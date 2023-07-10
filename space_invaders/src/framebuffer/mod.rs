pub use color::Color;
pub use coordinates::Coordinates;

pub mod color;

pub mod coordinates;

pub mod fb_trait;
#[cfg(feature = "std")]
pub mod std_fb;
#[cfg(feature = "std")]
pub use std_fb::StdFrameBuffer;

#[derive(Debug, Clone)]
pub struct Pixel {
    pub point: Coordinates,
    pub color: Color,
}

impl Pixel {
    pub fn new(point: Coordinates, color: Color) -> Self {
        Self { color, point }
    }
}
