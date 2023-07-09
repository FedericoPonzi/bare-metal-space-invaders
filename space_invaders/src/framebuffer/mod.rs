use color::Color;
use coordinates::Coordinates;

pub mod color;
pub mod coordinates;
pub mod fb_trait;
#[cfg(feature = "std")]
pub mod std_fb;
#[cfg(feature = "std")]
pub use std_fb::StdFrameBuffer;

#[cfg(feature = "no_std")]
pub mod raw_fb;
#[cfg(feature = "no_std")]
pub use raw_fb::FrameBuffer;

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
