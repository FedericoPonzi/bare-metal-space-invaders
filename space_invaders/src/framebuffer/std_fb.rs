use crate::actor::{Shoot, ShootOwner, HERO_WIDTH};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::game_context::HeroMovementDirection;
use crate::{KeyPressedKeys, MemoryAllocator, UserInput, SCREEN_HEIGHT, SCREEN_WIDTH};
use core::alloc::Layout;
use log::info;
use minifb::{Key, Window, WindowOptions};
use std::vec;

pub struct StdFrameBuffer {
    pub(crate) window: Window,
    buffer: Vec<u32>,
}
impl MemoryAllocator for StdFrameBuffer {
    fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { std::alloc::alloc(layout) }
    }
}
impl UserInput for StdFrameBuffer {
    fn get_input(&self) -> impl Iterator<Item = KeyPressedKeys> {
        self.window
            .get_keys()
            .into_iter()
            .filter_map(|key| match key {
                Key::A | Key::Left => Some(KeyPressedKeys::Left),
                Key::D | Key::Right => Some(KeyPressedKeys::Right),
                Key::Space => Some(KeyPressedKeys::Shoot),
                _ => None,
            })
    }
}
impl StdFrameBuffer {
    pub fn new() -> Self {
        let mut window = Window::new(
            "Test - ESC to exit",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        let buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        window.set_title("BareMetal Space Invaders");

        StdFrameBuffer { window, buffer }
    }
}

impl FrameBufferInterface for StdFrameBuffer {
    fn raw_buffer(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn update(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}
