use crate::actor::Shoot;
use crate::framebuffer::color::Color;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Pixel;
use crate::{HeroMovementDirection, SCREEN_HEIGHT, SCREEN_WIDTH};
use core::alloc;

pub trait FrameBufferInterface {
    fn alloc(&self, layout: alloc::Layout) -> *mut u8;

    fn draw_rect_fill(&mut self, point: Coordinates, width: u32, height: u32, color: Color) {
        for y in 0..height {
            for x in 0..width {
                self.use_pixel(Pixel::new(
                    Coordinates::new(point.x() + x, point.y() + y),
                    color,
                ));
            }
        }
    }

    //TODO: fn write_char(&mut self, c: char, coordinates: Coordinates, color: Color);

    fn write_ui(&mut self, coordinates: Coordinates, text: &str, color: Color) {
        let mut x = coordinates.x();
        let mut y = coordinates.y();
        for c in text.chars() {
            if c == '\n' {
                y += 1;
                x = coordinates.x();
                continue;
            }
            self.use_pixel(Pixel::new(Coordinates::new(x, y), color));
            x += 1;
            if x >= SCREEN_WIDTH as u32 {
                x = coordinates.x();
                y += 1;
                if y >= SCREEN_HEIGHT as u32 {
                    break;
                }
            }
            //self.write_char(c, Coordinates::new(x, y), color);
        }
    }
    fn random(&self) -> u32;

    /// [x,y] the top left center
    #[inline(always)]
    fn draw_rect(&mut self, point: Coordinates, width: u32, height: u32, color: Color) {
        for y in 0..height {
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x(), point.y() + y),
                color.clone(),
            ));
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x() + width, point.y() + y),
                color.clone(),
            ));
        }
        for x in 0..width {
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x() + x, point.y()),
                color.clone(),
            ));
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x() + x, point.y() + height),
                color.clone(),
            ));
        }
    }

    fn raw_buffer(&mut self) -> &mut [u32];
    fn width(&self) -> usize {
        SCREEN_WIDTH
    }
    fn height(&self) -> usize {
        SCREEN_HEIGHT
    }
    fn use_pixel(&mut self, pixel: Pixel) {
        let width = self.width();
        self.raw_buffer()[width * pixel.point.y() as usize + pixel.point.x() as usize] =
            pixel.color.as_rgb_u32();
    }

    fn display_image(&mut self, top_left: Coordinates, image: &[u32], width: u32) {
        let fb_width = self.width();

        for pos in 0..image.len() as u32 {
            let y = pos / width;
            let x = pos % width;
            let (x, y) = (x + top_left.x(), y + top_left.y());
            let index = fb_width * y as usize + x as usize;
            self.raw_buffer()[index] = image[pos as usize];
        }
    }
    fn clear_screen(&mut self) {
        for i in self.raw_buffer().iter_mut() {
            *i = 0;
        }
    }

    // draw the local buffer of the framebuffer to the screen
    fn update(&mut self);

    // get input from keyboard
    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>);
}
