/// RPI 3 framebuffer
pub struct FrameBuffer {
    // this could be an array.
    pub(crate) lfb_ptr: &'static u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pitch: u32,
    pub(crate) is_rgb: bool,
    pub(crate) is_brg: bool,
    /// crate::mailbox::FB_VIRTUAL_WIDTH
    pub(crate) fb_virtual_width: u32,
    /// Bits used by each pixel
    pub depth_bits: u32,
    pub(crate) buffer: Vec<u32>,
}

impl FrameBuffer {
    pub fn max_screen_size(&self) -> u32 {
        (self.depth_bits) * self.width * self.height
    }
    pub(crate) fn clear_screen(&mut self) {
        unsafe {
            core::ptr::write_bytes(
                self.lfb_ptr as *const u32 as *mut u32,
                0,
                self.width as usize * self.height as usize,
            )
        }
    }
    pub fn use_pixel(&mut self, pixel: Pixel) {
        unsafe {
            let ptr = self.lfb_ptr as *const u32 as *mut u32;
            let x = pixel.point.x;
            let y = pixel.point.y;
            let to_write = pixel.color.as_brga_u32();
            // why fb_virtualwidth? because it's the "actual" screen size.
            // No need to use depth - It's a pointer to u32 and depth is 32 bits => 4 bytes.
            // No need to use pitch, because I think it also refers to physical.
            let offset = (self.fb_virtual_width * y) + x;
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
            if offset >= self.max_screen_size() {
                println!("Offset is >= max_screen_size, skipping.");
            }
        }
    }
}
