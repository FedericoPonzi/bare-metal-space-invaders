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

    /// display a pixel
    fn use_pixel(&mut self, pixel: Pixel);

    /// draw the image to the local buffer. It needs a call to update in order to show it on screen.
    fn display_image(&mut self, top_left: Coordinates, image: &[u32], width: u32);

    // draw the local buffer of the framebuffer to the screen
    fn update(&mut self);

    // clear the screen (i.e. clear the framebuffer)
    fn clear_screen(&mut self);

    // get input from keyboard
    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>);
}
