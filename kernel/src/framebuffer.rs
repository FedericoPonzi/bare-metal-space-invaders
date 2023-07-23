use crate::mmio::PL011_UART_START;
use crate::uart_pl011::PL011Uart;
use alloc::vec;
use core::alloc::GlobalAlloc;
use space_invaders::{FrameBufferInterface, KeyPressedKeys, MemoryAllocator, UserInput};

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
impl MemoryAllocator for FrameBuffer {
    #[inline(always)]
    fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { crate::allocator::ALLOCATOR.alloc(layout) }
    }
}
impl UserInput for FrameBuffer {
    #[inline(always)]
    fn get_input(&self) -> impl Iterator<Item = KeyPressedKeys> {
        // TODO: no need to init it again.
        UARTIterator::new(unsafe { PL011Uart::new(PL011_UART_START) })
            .filter_map(|ch| match ch {
                'a' | 'A' => Some(KeyPressedKeys::Left),
                'd' | 'D' => Some(KeyPressedKeys::Right),
                //'r' | 'R' => Some(KeyPressedKeys::RestartGame),
                ' ' => Some(KeyPressedKeys::Shoot),
                _ => None,
            })
            .into_iter()
    }
}

impl FrameBufferInterface for FrameBuffer {
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

// Define a custom iterator that reads characters from UART until None is encountered.
struct UARTIterator {
    uart: PL011Uart,
    max_input: usize,
}

impl UARTIterator {
    fn new(uart: PL011Uart) -> Self {
        UARTIterator {
            uart,
            max_input: 10,
        }
    }
}

impl Iterator for UARTIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.max_input == 0 {
            return None;
        }
        match self.uart.read_char_unblocking() {
            Some(ch) => {
                self.max_input -= 1;
                Some(ch)
            }
            None => {
                self.max_input = 0;
                None
            }
        }
    }
}
