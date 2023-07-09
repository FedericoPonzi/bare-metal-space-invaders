use crate::actor::{Shoot, ShootOwner};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::framebuffer::Pixel;
use crate::HeroMovementDirection;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
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
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        StdFrameBuffer { window, buffer }
    }
}

impl FrameBufferInterface for StdFrameBuffer {
    fn use_pixel(&mut self, pixel: Pixel) {
        self.buffer[WIDTH * pixel.point.y as usize + pixel.point.x as usize] =
            pixel.color.as_rgb_u32();
    }

    fn display_image(&mut self, top_left: Coordinates, image: &[u32], width: u32) {
        for pos in 0..image.len() as u32 {
            let y = pos / width;
            let x = pos % width;
            let (x, y) = (x + top_left.x, y + top_left.y);
            self.buffer[WIDTH * y as usize + x as usize] = image[pos as usize];
        }
    }

    fn update(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
    fn clear_screen(&mut self) {
        for i in self.buffer.iter_mut() {
            *i = 0;
        }
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
                    println!("pew!");
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
