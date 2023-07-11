use crate::actor::Shoot;
use crate::framebuffer::color::Color;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Pixel;
use crate::HeroMovementDirection;

use micromath::F32;

pub trait FrameBufferInterface {
    /// TODO: cleanup.
    /// Bresenham algorithm for draw line: https://gist.github.com/bert/1085538
    fn draw_line(&mut self, start: Coordinates, end: Coordinates, color: Color) {
        let x1 = F32::from(end.x as f32);
        let mut x0 = F32::from(start.x as f32);
        let mut y0 = F32::from(start.y as f32);
        let y1 = F32::from(end.y as f32);

        let dx = (x1 - x0).abs();
        let sx: F32 = if x0 < x1 { 1f32.into() } else { (-1f32).into() };
        let dy = -(y1 - y0).abs();
        let sy: F32 = if y0 < y1 { 1f32.into() } else { (-1f32).into() };
        let mut err = dx + dy;
        let mut e2: F32 = F32::from(0f32); /* error value e_xy */

        loop {
            self.use_pixel(Pixel::new(
                Coordinates::new(x0.0 as u32, y0.0 as u32),
                color.clone(),
            ));
            if x0 == x1 && y0 == y1 {
                break;
            }
            e2 = F32::from(2f32) * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            } /* e_xy+e_x > 0 */
            if e2 <= dx {
                err += dx;
                y0 += sy;
            } /* e_xy+e_y < 0 */
        }
    }
    fn draw_rect_fill(&mut self, point: Coordinates, width: u32, height: u32, color: Color) {
        for y in 0..height {
            for x in 0..width {
                self.use_pixel(Pixel::new(
                    Coordinates::new(point.x + x, point.y + y),
                    color,
                ));
            }
        }
    }

    /// [x,y] the top left center
    #[inline(always)]
    fn draw_rect(&mut self, point: Coordinates, width: u32, height: u32, color: Color) {
        for y in 0..height {
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x, point.y + y),
                color.clone(),
            ));
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x + width, point.y + y),
                color.clone(),
            ));
        }
        for x in 0..width {
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x + x, point.y),
                color.clone(),
            ));
            self.use_pixel(Pixel::new(
                Coordinates::new(point.x + x, point.y + height),
                color.clone(),
            ));
        }
    }

    fn raw_buffer(&mut self) -> &mut [u32];
    fn width(&self) -> usize;
    fn use_pixel(&mut self, pixel: Pixel) {
        let width = self.width();
        self.raw_buffer()[width * pixel.point.y as usize + pixel.point.x as usize] =
            pixel.color.as_rgb_u32();
    }

    fn display_image(&mut self, top_left: Coordinates, image: &[u32], width: u32) {
        let fb_width = self.width();
        for pos in 0..image.len() as u32 {
            let y = pos / width;
            let x = pos % width;
            let (x, y) = (x + top_left.x, y + top_left.y);
            self.raw_buffer()[fb_width * y as usize + x as usize] = image[pos as usize];
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
