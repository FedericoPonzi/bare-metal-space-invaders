use crate::actor::{Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use crate::{HeroMovementDirection, SCREEN_HEIGHT, SCREEN_MARGIN, SCREEN_WIDTH};
use core::mem;

const HERO: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const HERO_WIDTH: u32 = 46;
const HERO_HEIGHT: u32 = 29;

const HERO_SPAWN_X: u32 = (SCREEN_WIDTH / 2) as u32 - HERO_WIDTH;
const HERO_SPAWN_Y: u32 = (SCREEN_HEIGHT - SCREEN_MARGIN - HERO_HEIGHT as usize) as u32;

const HERO_SPEED_MS: u64 = 200; // pixels per millisecond

#[derive(Copy, Clone)]
pub struct Hero {
    pub(crate) structure: ActorStructure,
}
impl Default for Hero {
    fn default() -> Self {
        let hero_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(HERO) };

        Hero {
            structure: ActorStructure {
                sprite: hero_sprite,
                width: HERO_WIDTH,
                height: HERO_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(HERO_SPAWN_X, HERO_SPAWN_Y),
            },
        }
    }
}
impl Hero {
    pub fn new() -> Hero {
        Self::default()
    }
    fn move_left(&mut self, delta: u64) {
        self.structure
            .coordinates
            .sub_x((delta * HERO_SPEED_MS / 1000) as u32);
        self.structure.coordinates.x =
            core::cmp::max(SCREEN_MARGIN as u32, self.structure.coordinates.x);
    }

    fn move_right(&mut self, delta: u64) {
        self.structure
            .coordinates
            .add_x((delta * HERO_SPEED_MS / 1000) as u32);
        if self.structure.coordinates.x + self.structure.width
            >= (SCREEN_WIDTH - SCREEN_MARGIN) as u32
        {
            self.structure.coordinates.x =
                SCREEN_WIDTH as u32 - self.structure.width - SCREEN_MARGIN as u32;
        }
    }

    pub fn handle_movement(&mut self, hero_movement_direction: HeroMovementDirection, delta: u64) {
        match hero_movement_direction {
            HeroMovementDirection::Left => {
                self.move_left(delta);
            }
            HeroMovementDirection::Right => {
                self.move_right(delta);
            }
            _ => {
                // hero hasn't move
            }
        }
    }
}

impl Actor for Hero {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }
    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
}
