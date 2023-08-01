use crate::actor::{Actor, ActorStructure, Sprite};
use crate::framebuffer::Coordinates;
use crate::{SCREEN_HEIGHT_NO_MARGIN, SCREEN_MARGIN, SCREEN_WIDTH, SCREEN_WIDTH_NO_MARGIN};

pub static HERO_SPRITE_U32: &[u32] = crate::include_bytes_align_as!(
    u32,
    "/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/hero.data"
);

pub const HERO_WIDTH: u32 = 60;
pub(crate) const HERO_HEIGHT: u32 = 29;

pub const HERO_SPAWN_X: u32 = (SCREEN_WIDTH / 2) - HERO_WIDTH;
pub const HERO_SPAWN_Y: u32 = SCREEN_HEIGHT_NO_MARGIN - HERO_HEIGHT;

const HERO_SPEED_MS: f64 = 200.0 / 1000.0; // pixels per millisecond

#[derive(Clone, Copy, Debug)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
    RestartGame,
}

#[derive(Copy, Clone)]
pub struct Hero {
    pub(crate) structure: ActorStructure,
}

impl Hero {
    pub fn new() -> Hero {
        Hero {
            structure: ActorStructure {
                sprite: Some(Sprite::new(HERO_SPRITE_U32)),
                width: HERO_WIDTH,
                height: HERO_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(HERO_SPAWN_X, HERO_SPAWN_Y),
            },
        }
    }
    fn move_left(&mut self, delta: u64) {
        self.structure
            .coordinates
            .sub_virtual_x(delta as f64 * HERO_SPEED_MS);
        self.structure
            .coordinates
            .set_virtual_x(core::cmp::max(SCREEN_MARGIN, self.get_coordinates().x()) as f64);
    }

    fn move_right(&mut self, delta: u64) {
        self.structure
            .coordinates
            .add_virtual_x(delta as f64 * HERO_SPEED_MS);
        if self.get_coordinates().x() + self.structure.width >= SCREEN_WIDTH_NO_MARGIN {
            self.structure
                .coordinates
                .set_virtual_x((SCREEN_WIDTH - self.structure.width - SCREEN_MARGIN) as f64);
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
