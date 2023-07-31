pub use color::Color;
pub use coordinates::Coordinates;

pub mod color;

pub mod coordinates;

pub mod fb_trait;
#[cfg(feature = "std")]
pub mod std_fb;
#[cfg(feature = "std")]
pub use std_fb::StdFrameBuffer;
