use crate::framebuffer::color;
use crate::framebuffer::color::Color;
use crate::framebuffer::coordinates::Coordinates;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};

const LETTER_FONT_WEIGHT: FontWeight = FontWeight::Regular;
const LETTER_FONT_HEIGHT: RasterHeight = RasterHeight::Size20;
pub const LETTER_WIDTH: usize = get_raster_width(LETTER_FONT_WEIGHT, LETTER_FONT_HEIGHT);

pub trait FrameBufferInterface {
    fn draw_rect_fill(&mut self, point: &Coordinates, width: u32, height: u32, color: Color) {
        let width = width as usize;
        let height = height as usize;
        for y in 0..height {
            for x in 0..width {
                self.use_pixel(point.x_usize() + x, point.y_usize() + y, color);
            }
        }
    }

    fn write_char(&mut self, c: char, coordinates: Coordinates, color: Color) {
        let char_raster =
            get_raster(c, LETTER_FONT_WEIGHT, LETTER_FONT_HEIGHT).expect("unsupported char");
        for (row_i, row) in char_raster.raster().iter().enumerate() {
            for (col_i, pixel) in row.iter().enumerate() {
                let actual_color = if pixel.count_zeros() == 8 {
                    color::BLUE_COLOR
                } else {
                    color
                };
                self.use_pixel(
                    coordinates.x_usize() + col_i,
                    coordinates.y_usize() + row_i,
                    actual_color,
                );
            }
        }
    }

    /// [x,y] the top left center
    fn draw_rect(&mut self, point: Coordinates, width: u32, height: u32, color: Color) {
        let width = width as usize;
        let height = height as usize;
        for y in 0..height {
            self.use_pixel(point.x_usize(), point.y_usize() + y, color);
            self.use_pixel(point.x_usize() + width, point.y_usize() + y, color);
        }
        for x in 0..width {
            self.use_pixel(point.x_usize() + x, point.y_usize(), color);
            self.use_pixel(point.x_usize() + x, point.y_usize() + height, color);
        }
    }

    fn raw_buffer(&mut self) -> &mut [u32];
    fn width(&self) -> usize {
        self.width_u32() as usize
    }
    fn width_u32(&self) -> u32 {
        SCREEN_WIDTH
    }
    fn height_u32(&self) -> u32 {
        SCREEN_HEIGHT
    }
    fn height(&self) -> usize {
        self.height_u32() as usize
    }

    fn use_pixel(&mut self, x_usize: usize, y_usize: usize, color: Color) {
        let width = self.width();
        self.raw_buffer()[width * y_usize + x_usize] = color.rgb();
    }

    fn display_image(&mut self, top_left: &Coordinates, image: &[u32], width: u32, height: u32) {
        let fb_width = self.width();
        let width = width as usize;
        for y in 0..height as usize {
            for x in 0..width {
                let pos: usize = y * width + x;
                let (x, y) = (x + top_left.x_usize(), y + top_left.y_usize());
                let index = fb_width * y + x;
                self.raw_buffer()[index] = image[pos];
            }
        }
    }
    fn clear_screen(&mut self) {
        for i in self.raw_buffer().iter_mut() {
            *i = 0;
        }
    }

    // draw the local buffer of the framebuffer to the screen
    fn update(&mut self);
}
