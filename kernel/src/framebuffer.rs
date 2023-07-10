use crate::uart_pl011::PL011Uart;
use alloc::vec;
use log::info;
use space_invaders::actor::{Shoot, ShootOwner};
use space_invaders::{Coordinates, HeroMovementDirection, Pixel};

/// RPI 3 framebuffer
pub struct FrameBuffer {
    // this could be an array.
    pub lfb_ptr: &'static u32,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub is_rgb: bool,
    pub is_brg: bool,
    /// crate::mailbox::FB_VIRTUAL_WIDTH
    pub fb_virtual_width: u32,
    /// Bits used by each pixel
    pub depth_bits: u32,
    pub buffer: vec::Vec<u32>, //[u32; 3024],
    pub uart: PL011Uart,
}

impl space_invaders::FrameBufferInterface for FrameBuffer {
    fn raw_buffer(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn width(&self) -> usize {
        self.width as usize
    }

    fn update(&mut self) {
        //let cnt = self.width as usize * self.height as usize;
        unsafe {
            let dst_buffer = self.lfb_ptr as *const u32 as *mut u32;
            let src_buffer = self.buffer.as_ptr();
            core::ptr::copy(src_buffer, dst_buffer, self.buffer.len());
        }
    }

    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>) {
        let mut max = 10;
        let mut hero = HeroMovementDirection::Still;
        let mut shoot = None;
        loop {
            max -= 1;
            match self.uart.read_char_unblocking() {
                Some(ch) => match ch {
                    'a' | 'A' => {
                        hero = HeroMovementDirection::Left;
                    }
                    'd' | 'D' => {
                        hero = HeroMovementDirection::Right;
                    }
                    ' ' => {
                        let new_shoot = Shoot::new(
                            Coordinates::new(hero_coordinates.x, hero_coordinates.y - 20),
                            ShootOwner::Hero,
                        );
                        info!("pew!");
                        shoot = Some(new_shoot);
                    }
                    received => {
                        info!("Received key {}, not doing anything.", received);
                    }
                },
                None => break, // input empty
            }
            if max == 0 {
                break;
            }
        }

        info!("Hero direction: {:?},  shoot: {:?} ", hero, shoot.is_some());
        (hero, shoot)
    }
}

impl FrameBuffer {
    pub fn max_screen_size(&self) -> u32 {
        (self.depth_bits) * self.width * self.height
    }
    pub fn clear_screen(&mut self) {
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
                /*println!(
                    "Request to write pixel: {:?}, but max screensize is :{}",
                    pixel,
                    self.max_screen_size()
                );*/
            }
            if offset >= self.max_screen_size() {
                //info!("Offset is >= max_screen_size, skipping.");
            }
        }
    }
}
