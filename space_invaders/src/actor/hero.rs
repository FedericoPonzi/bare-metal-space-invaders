use crate::actor::{Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use crate::HeroMovementDirection;
use std::mem;

const HERO: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const HERO_WIDTH: u32 = 46;
const HERO_HEIGHT: u32 = 29;

const HERO_SPAWN_X: u32 = 1000;
const HERO_SPAWN_Y: u32 = 1000;

const HERO_MOVEMENT_OFFSET: u32 = 10;

#[derive(Copy, Clone)]
pub struct Hero {
    pub(crate) structure: ActorStructure,
}

impl Hero {
    pub fn new() -> Hero {
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
    pub(crate) fn move_left(&mut self) {
        self.structure.coordinates.sub_x(HERO_MOVEMENT_OFFSET);
    }
    pub(crate) fn move_right(&mut self) {
        self.structure.coordinates.add_x(HERO_MOVEMENT_OFFSET);
    }
    pub fn handle_movement(&mut self, hero_movement_direction: HeroMovementDirection) {
        match hero_movement_direction {
            HeroMovementDirection::Left => {
                self.move_left();
            }
            HeroMovementDirection::Right => {
                self.move_right();
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
