mod enemy;
mod hero;
mod shoot;

pub use enemy::*;
pub use hero::*;
pub use shoot::*;

use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite {
    sprite: &'static [u32],
}
impl Sprite {
    pub fn new(sprite: &'static [u8], fb: &impl FrameBufferInterface) -> Self {
        let bytes: &'static [u8] = sprite;
        let len = bytes.len() / core::mem::size_of::<u32>();

        let alignment = 16; // Specify the desired alignment (e.g., 16 bytes)

        let data_size = len * core::mem::size_of::<u32>();
        let new_size = data_size + alignment - 1;

        let original_data_ptr = bytes.as_ptr();
        let new_data_ptr = {
            let layout = core::alloc::Layout::from_size_align(new_size, alignment)
                .expect("Failed to create layout");
            let new_data_ptr = unsafe { fb.alloc(layout) } as *mut u8;
            if new_data_ptr.is_null() {
                panic!("Failed to allocate memory with the desired alignment");
            }
            new_data_ptr
        };

        // Copy the original data to the newly allocated memory
        unsafe {
            core::ptr::copy_nonoverlapping(original_data_ptr, new_data_ptr, data_size);
        }

        // Create a slice from the new aligned memory
        let data: &'static [u32] =
            unsafe { core::slice::from_raw_parts(new_data_ptr as *const u32, len) };
        Self { sprite: data }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorStructure {
    // TODO: can probably use RC instead
    pub sprite: Option<Sprite>,
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
        fb.display_image(
            structure.coordinates,
            structure.sprite.unwrap().sprite,
            structure.width,
        );
    }
}
