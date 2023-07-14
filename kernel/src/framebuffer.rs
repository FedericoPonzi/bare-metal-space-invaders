use crate::uart_pl011::PL011Uart;
use alloc::vec;
use core::alloc::GlobalAlloc;
use log::info;
use space_invaders::actor::{Shoot, ShootOwner};
use space_invaders::{Coordinates, FrameBufferInterface, HeroMovementDirection, Pixel};

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
    //todo: using the array instead of vec breaks the system init.
    pub buffer: vec::Vec<u32>, //[u32; FB_BUFFER_LEN],
    pub uart: PL011Uart,
}

impl space_invaders::FrameBufferInterface for FrameBuffer {
    #[inline(always)]
    fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { crate::allocator::ALLOCATOR.alloc(layout) }
    }
    #[inline(always)]
    fn raw_buffer(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    #[inline(always)]
    fn width(&self) -> usize {
        self.width as usize
    }

    #[inline(always)]
    fn update(&mut self) {
        unsafe {
            let dst_buffer = self.lfb_ptr as *const u32 as *mut u32;
            let src_buffer = self.buffer.as_ptr();
            core::ptr::copy_nonoverlapping(src_buffer, dst_buffer, self.buffer.len());
        }
    }

    #[inline(always)]
    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
        fb: &impl FrameBufferInterface,
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
                    'r' | 'R' => {
                        hero = HeroMovementDirection::RestartGame;
                    }
                    ' ' => {
                        let new_shoot = Shoot::new(
                            Coordinates::new(hero_coordinates.x(), hero_coordinates.y() - 20),
                            ShootOwner::Hero,
                            fb,
                        );
                        //info!("pew!");
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

        //info!("Hero direction: {:?},  shoot: {:?} ", hero, shoot.is_some());
        (hero, shoot)
    }
}

impl FrameBuffer {
    pub fn max_screen_size(&self) -> u32 {
        (self.depth_bits) * self.width * self.height
    }
    #[inline(always)]
    pub fn clear_screen(&mut self) {
        unsafe {
            core::ptr::write_bytes(
                self.lfb_ptr as *const u32 as *mut u32,
                0,
                self.width as usize * self.height as usize,
            )
        }
    }
}
