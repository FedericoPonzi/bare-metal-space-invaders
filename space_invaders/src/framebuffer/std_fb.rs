use crate::actor::{Shoot, ShootOwner};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::HeroMovementDirection;
use alloc::vec;
use alloc::vec::Vec;
use log::info;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const MARGIN: usize = 30;

pub struct StdFrameBuffer {
    pub(crate) window: Window,
    buffer: Vec<u32>,
}

impl StdFrameBuffer {
    pub fn new() -> Self {
        let mut window = Window::new(
            "Test - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        let buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        StdFrameBuffer { window, buffer }
    }
}

impl FrameBufferInterface for StdFrameBuffer {
    fn raw_buffer(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    fn width(&self) -> usize {
        WIDTH
    }

    fn update(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
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
                        Coordinates::new(hero_coordinates.x, hero_coordinates.y - 20),
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
