use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::{KeyPressedKeys, UserInput, SCREEN_HEIGHT, SCREEN_WIDTH};
use log::info;
use minifb::{Key, Window, WindowOptions};

pub struct StdFrameBuffer {
    pub(crate) window: Window,
    buffer: Vec<u32>,
}
impl UserInput for StdFrameBuffer {
    fn get_input(&mut self) -> impl Iterator<Item = KeyPressedKeys> {
        info!(
            "self.window.get_key_pressed() : {:?} ",
            self.window.get_keys()
        );
        let keys = self.window.get_keys();
        self.window.update();
        keys.into_iter().filter_map(|key| {
            info!("key: {key:?}");
            match key {
                Key::A | Key::Left => Some(KeyPressedKeys::Left),
                Key::D | Key::Right => Some(KeyPressedKeys::Right),
                Key::R => Some(KeyPressedKeys::Restart),
                Key::Space => Some(KeyPressedKeys::Shoot),
                _ => None,
            }
        })
    }
}
impl Default for StdFrameBuffer {
    fn default() -> Self {
        let mut window = Window::new(
            "Test - ESC to exit",
            SCREEN_WIDTH as usize,
            SCREEN_HEIGHT as usize,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        let buffer: Vec<u32> = vec![0; (SCREEN_WIDTH * SCREEN_HEIGHT) as usize];

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        window.set_title("BareMetal Space Invaders");

        StdFrameBuffer { window, buffer }
    }
}
impl StdFrameBuffer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FrameBufferInterface for StdFrameBuffer {
    fn raw_buffer(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn update(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize)
            .unwrap();
    }
}
