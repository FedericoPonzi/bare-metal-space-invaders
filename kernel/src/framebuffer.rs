use crate::mailbox::{set_virtual_framebuffer_offset, TOTAL_FB_BUFFER_LEN};
use crate::mmio::PL011_UART_START;
use crate::uart_pl011::PL011Uart;
use core::alloc::GlobalAlloc;
use log::info;
use space_invaders::{FrameBufferInterface, KeyPressedKeys, MemoryAllocator, UserInput};

/// RPI 3 framebuffer
pub struct FrameBuffer {
    // this could be an array.
    pub framebuff: &'static mut [u32],
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
    pub uart: PL011Uart,
    pub current_index: u8,
}
impl MemoryAllocator for FrameBuffer {
    fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { crate::allocator::ALLOCATOR.alloc(layout) }
    }
}
impl UserInput for FrameBuffer {
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
    fn raw_buffer(&mut self) -> &mut [u32] {
        let start = self.width() * self.current_height_offset();
        let end_of_buffer = start + self.single_screen_len();
        &mut self.framebuff[start..end_of_buffer]
    }

    fn width(&self) -> usize {
        self.width as usize
    }

    fn update(&mut self) {
        set_virtual_framebuffer_offset(self.current_index as u32 * self.height);
        self.current_index = Self::inverse(self.current_index);
    }

    fn clear_screen(&mut self) {
        let mut slice_ptr = (&mut self.raw_buffer()).as_mut_ptr();
        //info!("clearing screen, index: {}", self.current_index);

        for i in 0..self.single_screen_len() {
            unsafe {
                // volatile is 10ms slower than non volatile :/
                // but using non-volatile makes the sprite flicker
                core::ptr::write_volatile(slice_ptr.add(i), 0);
            }
        }
    }
}

impl FrameBuffer {
    fn single_screen_len(&self) -> usize {
        (self.height * self.width) as usize
    }
    fn current_height_offset(&self) -> usize {
        self.height as usize * self.current_index as usize
    }
    fn inverse(index: u8) -> u8 {
        if index == 1 {
            0
        } else {
            1
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
