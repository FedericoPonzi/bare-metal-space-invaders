mod enemy;
mod hero;
mod shoot;

pub use enemy::*;
pub use hero::*;
pub use shoot::*;

use crate::framebuffer::color::WHITE_COLOR;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use alloc::vec;
use alloc::vec::Vec;

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

fn scale_down_image(image: &[u32], width: usize, height: usize, scaling_factor: usize) -> Vec<u32> {
    let scaled_width = width / scaling_factor;
    let scaled_height = height / scaling_factor;
    let mut scaled_image = vec![0u32; scaled_width * scaled_height];

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let mut sum_r = 0;
            let mut sum_g = 0;
            let mut sum_b = 0;

            for i in 0..scaling_factor {
                for j in 0..scaling_factor {
                    let px = x * scaling_factor + j;
                    let py = y * scaling_factor + i;
                    let index = py * width + px;
                    let pixel = image[index];

                    sum_r += (pixel >> 16) & 0xFF;
                    sum_g += (pixel >> 8) & 0xFF;
                    sum_b += pixel & 0xFF;
                }
            }

            let num_pixels = (scaling_factor * scaling_factor) as u32;
            let avg_r = sum_r / num_pixels;
            let avg_g = sum_g / num_pixels;
            let avg_b = sum_b / num_pixels;
            let scaled_pixel = (avg_r << 16) | (avg_g << 8) | avg_b;
            scaled_image[y * scaled_width + x] = scaled_pixel;
        }
    }

    scaled_image
}
