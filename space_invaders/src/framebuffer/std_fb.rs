use crate::actor::{Shoot, ShootOwner, HERO_WIDTH};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::{HeroMovementDirection, SCREEN_HEIGHT, SCREEN_WIDTH};
use minifb::{Key, Window, WindowOptions};
use std::vec;

pub struct StdFrameBuffer {
    pub(crate) window: Window,
    buffer: Vec<u32>,
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
    fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe { std::alloc::alloc(layout) }
    }

    //fn write_char(&mut self, c: char, coordinates: Coordinates, color: Color) {}

    fn random(&self) -> u32 {
        // generate a random u32
        rand::random()
    }

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

    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>) {
        let mut hero_movement_direction = HeroMovementDirection::Still;
        let mut shoot = None;
        for key in self.window.get_keys() {
            match key {
                Key::A | Key::Left => {
                    hero_movement_direction = HeroMovementDirection::Left;
                }
                Key::D | Key::Right => {
                    hero_movement_direction = HeroMovementDirection::Right;
                }
                Key::Space => {
                    let new_shoot = Shoot::new(
                        Coordinates::new(
                            hero_coordinates.x() + HERO_WIDTH / 2,
                            hero_coordinates.y() - 10,
                        ),
                        ShootOwner::Hero,
                    );
                    //info!("pew!");
                    shoot = Some(new_shoot);
                }
                _ => {
                    hero_movement_direction = HeroMovementDirection::Still;
                }
            }
        }
        (hero_movement_direction, shoot)
    }
}
