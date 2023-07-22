use crate::actor::{Actor, ActorStructure, Sprite};
use crate::framebuffer::Coordinates;
use crate::game_context::HeroMovementDirection;
use crate::{MemoryAllocator, SCREEN_HEIGHT, SCREEN_MARGIN, SCREEN_WIDTH, SCREEN_WIDTH_NO_MARGIN};

const HERO: &[u8] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/hero.data");
pub const HERO_WIDTH: u32 = 60;
pub(crate) const HERO_HEIGHT: u32 = 29;

pub const HERO_SPAWN_X: u32 = (SCREEN_WIDTH / 2) as u32 - HERO_WIDTH;
pub const HERO_SPAWN_Y: u32 = (SCREEN_HEIGHT - SCREEN_MARGIN - HERO_HEIGHT as usize) as u32;

const HERO_SPEED_MS: f64 = 200.0 / 1000.0; // pixels per millisecond

#[derive(Copy, Clone)]
pub struct Hero {
    pub(crate) structure: ActorStructure,
}

impl Hero {
    #[inline(always)]
    pub fn new<A>(fb: &A) -> Hero
    where
        A: MemoryAllocator,
    {
        Hero {
            structure: ActorStructure {
                sprite: Some(Sprite::new(HERO, fb)),
                width: HERO_WIDTH,
                height: HERO_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(HERO_SPAWN_X, HERO_SPAWN_Y),
            },
        }
    }
    #[inline(always)]
    fn move_left(&mut self, delta: u64) {
        self.structure
            .coordinates
            .sub_virtual_x(delta as f64 * HERO_SPEED_MS);
        self.structure
            .coordinates
            .set_virtual_x(
                core::cmp::max(SCREEN_MARGIN as u32, self.structure.coordinates.x()) as f64,
            );
    }

    #[inline(always)]
    fn move_right(&mut self, delta: u64) {
        self.structure
            .coordinates
            .add_virtual_x(delta as f64 * HERO_SPEED_MS);
        if self.structure.coordinates.x() + self.structure.width >= (SCREEN_WIDTH_NO_MARGIN) as u32
        {
            self.structure.coordinates.set_virtual_x(
                (SCREEN_WIDTH as u32 - self.structure.width - SCREEN_MARGIN as u32) as f64,
            );
        }
    }

    #[inline(always)]
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
