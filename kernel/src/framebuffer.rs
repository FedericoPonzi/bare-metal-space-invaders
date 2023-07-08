use crate::println;
use micromath::F32;

pub struct FrameBuffer {
    // this could be an array.
    pub(crate) lfb_ptr: &'static u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pitch: u32,
    pub(crate) is_rgb: bool,
    pub(crate) is_brg: bool,
    /// Bits used by each pixel
    pub depth_bits: u32,
}
impl FrameBuffer {
    fn write_pixel(&self, p: Pixel) {}
}

#[derive(Debug, Clone)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}
#[derive(Debug, Clone)]
pub struct Pixel {
    point: Point,
    color: Color,
}

#[derive(Debug, Clone)]
pub struct Point {
    x: u32,
    y: u32,
}
impl Point {
    pub(crate) fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Pixel {
    pub fn new(point: Point, color: Color) -> Self {
        Self { color, point }
    }
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self::new_alpha(red, green, blue, 0x00)
    }
    fn new_alpha(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
    fn as_rgb_u32(&self) -> u32 {
        (255 << 28 | (self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
    }
    fn as_brg_u32(&self) -> u32 {
        ((self.alpha as u32) << 24)
            | ((self.red as u32) << 16)
            | ((self.green as u32) << 8)
            | (self.blue as u32)
    }
}

const WHITE_COLOR: Color = Color {
    red: 255,
    green: 255,
    blue: 255,
    alpha: 255,
};

impl FrameBuffer {
    fn max_screen_size(&self) -> u32 {
        (self.depth_bits) * self.width * self.height
    }
    pub fn use_pixel(&self, pixel: Pixel) {
        unsafe {
            let ptr = self.lfb_ptr as *const u32 as *mut u32;
            let x = pixel.point.x;
            let y = pixel.point.y;
            let to_write = pixel.color.as_brg_u32();
            // why fb_virtualwidth? because it's the "actual" screen size.
            // No need to use depth - It's a pointer to u32 and depth is 32 bits => 4 bytes.
            // No need to use pitch, because I think it also refers to physical.
            let offset = (crate::mailbox::FB_VIRTUAL_WIDTH * y) + x;
            if offset < self.max_screen_size() {
                // ptr is a pointer to u32. The add works by doing offset * size_of::<u32> = offset*4.
                let ptr = ptr.add((offset) as usize);
                core::ptr::write_volatile(ptr, to_write);
            } else {
                println!(
                    "Request to write pixel: {:?}, but max screensize is :{}",
                    pixel,
                    self.max_screen_size()
                );
            }
        }
    }
    /// TODO: cleanup.
    /// Bresenham algorithm for draw line: https://gist.github.com/bert/1085538
    fn draw_line(&mut self, start: Point, end: Point, color: Color) {
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
                Point::new(x0.0 as u32, y0.0 as u32),
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

    /// [x,y] the top left center
    fn draw_rect(&mut self, point: Point, width: u32, height: u32, color: Color, fill: bool) {
        if fill {
            for y in 0..height {
                for x in 0..width {
                    self.use_pixel(Pixel::new(
                        Point::new(point.x + x, point.y + y),
                        color.clone(),
                    ));
                }
            }
        } else {
            for y in 0..height {
                self.use_pixel(Pixel::new(Point::new(point.x, point.y + y), color.clone()));
                self.use_pixel(Pixel::new(
                    Point::new(point.x + width, point.y + y),
                    color.clone(),
                ));
            }
            for x in 0..width {
                self.use_pixel(Pixel::new(Point::new(point.x + x, point.y), color.clone()));
                self.use_pixel(Pixel::new(
                    Point::new(point.x + x, point.y + height),
                    color.clone(),
                ));
            }
        }
    }
}

pub fn lfb_showpicture(fb: &mut FrameBuffer) {
    const CRAB_WIDTH: u32 = 250;
    const CRAB_HEIGHT: u32 = 167;
    const BYTES_PER_PIXEL: u32 = 3;
    unsafe {
        const CRAB_BYTES: u32 = CRAB_HEIGHT * CRAB_WIDTH * BYTES_PER_PIXEL;
        let crab: &[u8; (CRAB_HEIGHT * CRAB_WIDTH * BYTES_PER_PIXEL) as usize] =
            include_bytes!("/home/isaacisback/dev/rust/iris-os/crab.rgb");

        for pos in (0..CRAB_BYTES).step_by(BYTES_PER_PIXEL as usize) {
            let y = pos / BYTES_PER_PIXEL / CRAB_WIDTH;
            let x = pos / BYTES_PER_PIXEL - y * CRAB_WIDTH;
            let pos = pos as usize;
            let pixel = Color::new(crab[pos], crab[pos + 1], crab[pos + 2]);
            let pixel = Pixel::new(Point::new(x + 5 as u32, y + 5 as u32), pixel);
            fb.use_pixel(pixel);
        }
        fb.draw_rect(
            Point::new(5, 5),
            CRAB_WIDTH,
            CRAB_HEIGHT,
            Color::new(0, 0xff, 0),
            false,
        );
        fb.draw_line(Point::new(0, 0), Point::new(150, 132), WHITE_COLOR);
    }
}
