use crate::actor::{Actor, ActorStructure, Enemy, Sprite, HERO_HEIGHT};
use crate::framebuffer::color::{SHOT_COLOR, WHITE_COLOR};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::FrameBufferInterface;
use log::info;

pub const SHOOT_BOX_WIDTH: u32 = 3;
pub const SHOOT_BOX_HEIGHT: u32 = 7;
const SHOOT_BOX_COLOR: Color = SHOT_COLOR;

// pixels per millisecond.
const SHOOT_SPEED: f64 = 400.0 / 1000.0;

pub const SHOOT_SPAWN_OFFSET_Y: u32 = HERO_HEIGHT + 10;

pub const SHOOT_ENEMY_MAX: usize = 3;
pub const SHOOT_HERO_MAX: usize = 4;

// max shots available to render at a time
pub const SHOOT_MAX_ALLOC: usize = SHOOT_ENEMY_MAX + SHOOT_HERO_MAX;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShootOwner {
    Hero,
    Enemy,
}

impl From<&Enemy> for Shoot {
    fn from(enemy: &Enemy) -> Self {
        let enemy_coordinates = enemy.structure.coordinates;
        Self {
            owner: ShootOwner::Enemy,
            structure: Shoot::structure(Coordinates::new(
                enemy_coordinates.x(),
                enemy_coordinates.y() + enemy.structure.height,
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shoot {
    pub(crate) structure: ActorStructure,
    pub(crate) owner: ShootOwner,
}

impl Actor for Shoot {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        fb.draw_rect_fill(
            self.structure.coordinates,
            self.structure.width,
            self.structure.height,
            SHOOT_BOX_COLOR,
        );
    }
}

impl Shoot {
    #[inline(always)]
    pub fn new(coordinates: Coordinates, owner: ShootOwner) -> Self {
        Shoot {
            structure: Self::structure(coordinates),
            owner,
        }
    }

    const fn structure(coordinates: Coordinates) -> ActorStructure {
        ActorStructure {
            sprite: None,
            width: SHOOT_BOX_WIDTH,
            height: SHOOT_BOX_HEIGHT,
            alive: true,
            coordinates,
        }
    }

    #[inline(always)]
    pub(crate) fn out_of_screen(&self, screen_height: u32) -> bool {
        let coordinates = self.structure.coordinates;
        (coordinates.y() as i32) - (self.structure.height as i32) <= 0
            || (coordinates.y() + self.structure.height) >= (screen_height)
    }

    #[inline(always)]
    pub(crate) fn move_forward(&mut self, delta: u64) {
        match &self.owner {
            ShootOwner::Hero => {
                self.structure.coordinates.sub_virtual_y(SHOOT_SPEED, delta);
            }
            ShootOwner::Enemy => {
                self.structure.coordinates.add_virtual_y(SHOOT_SPEED, delta);
            }
        }
    }
}
