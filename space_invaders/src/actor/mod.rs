mod barricade;
mod enemy;
mod hero;
pub(crate) mod shoot;

pub use barricade::*;
pub use enemy::*;
pub use hero::*;
pub use shoot::*;

use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::MemoryAllocator;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite {
    sprite: &'static [u32],
}
impl Sprite {
    pub fn new<A>(sprite: &'static [u8], fb: &A) -> Self
    where
        A: MemoryAllocator,
    {
        let bytes: &'static [u8] = sprite;
        let len = bytes.len() / core::mem::size_of::<u32>();

        let alignment = 16; // Specify the desired alignment (e.g., 16 bytes)

        let data_size = len * core::mem::size_of::<u32>();
        let new_size = data_size + alignment - 1;

        let original_data_ptr = bytes.as_ptr();
        let new_data_ptr = {
            let layout = core::alloc::Layout::from_size_align(new_size, alignment)
                .expect("Failed to create layout");
            let new_data_ptr = fb.alloc(layout);
            assert!(
                !new_data_ptr.is_null(),
                "Failed to allocate memory with the desired alignment"
            );
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
        if !structure.alive {
            return;
        }
        fb.display_image(
            structure.coordinates,
            structure.sprite.unwrap().sprite,
            structure.width,
        );
    }

    #[inline(always)]
    fn is_hit(&self, actor_structure: &ActorStructure) -> bool {
        let self_structure = self.get_structure();
        let self_coordinates = self_structure.coordinates;
        let actor_structure = actor_structure;
        let actor_coordinates = actor_structure.coordinates;

        let self_x = self_coordinates.x();
        let self_y = self_coordinates.y();
        let x = actor_coordinates.x();
        let y = actor_coordinates.y();

        let self_x_end = self_x + self_structure.width;
        let self_y_end = self_y + self_structure.height;
        let x_end = x + actor_structure.width;
        let y_end = y + actor_structure.height;

        x <= self_x_end && self_x <= x_end && y <= self_y_end && self_y <= y_end
            || x <= self_x_end && self_x <= x_end && y_end <= self_y_end && self_y <= y
            || x <= self_x_end && y <= self_y_end && self_y <= y && self_x <= x
            || x <= self_x_end && y_end <= self_y_end && self_y <= y && self_x <= x
            || y <= self_y_end && self_y <= y && x_end <= self_x_end && self_x <= x
            || y_end <= self_y_end && self_y <= y && x_end <= self_x_end && self_x <= x
    }
}

/*
#[cfg(test)]
mod test {
    #[macro_use]
    extern crate std;

    use crate::actor::{ActorStructure, Shoot, ShootOwner};

    #[test]
    pub fn test_hit() {
        let shoot = Shoot {
            structure: ActorStructure {
                sprite: &[0; 4],
                width: 1,
                height: 1,
                alive: true,
                coordinates: super::Coordinates { x: 0, y: 0 },
            },
            owner: ShootOwner::Hero,
        };
        let coordinates = super::Coordinates { x: 0, y: 0 };
        assert!(shoot.is_hit(&coordinates));
    }
}*/
