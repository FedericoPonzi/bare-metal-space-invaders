mod enemy;
mod hero;
mod shoot;

pub use enemy::*;
pub use hero::*;
pub use shoot::*;

use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorStructure {
    // TODO: can probably use RC instead
    pub sprite: &'static [u32],
    pub width: u32,
    pub height: u32,
    pub alive: bool,
    // Top left offset
    pub coordinates: Coordinates,
}

pub trait Actor {
    fn get_structure(&self) -> &ActorStructure;
    fn set_coordinates(&mut self, coordinates: Coordinates);

    #[inline(always)]
    fn move_to(&mut self, top_left_offset: Coordinates) {
        self.set_coordinates(top_left_offset);
    }

    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        let structure = self.get_structure();
        /*fb.draw_rect_fill(
            structure.coordinates,
            structure.width,
            structure.height,
            WHITE_COLOR,
        );*/

        fb.display_image(structure.coordinates, structure.sprite, structure.width);
    }
}
