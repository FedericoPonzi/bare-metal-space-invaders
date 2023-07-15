use crate::actor::{Actor, ActorStructure, Sprite, HERO_HEIGHT};
use crate::framebuffer::color::{SHOT_COLOR, WHITE_COLOR};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::FrameBufferInterface;
use log::info;

const SHOOT_BOX_WIDTH: u32 = 3;
const SHOOT_BOX_HEIGHT: u32 = 7;
const SHOOT_BOX_COLOR: Color = SHOT_COLOR;

// pixels per millisecond.
const SHOOT_SPEED: f64 = 400.0 / 1000.0;

pub const SHOOT_SPAWN_OFFSET_Y: u32 = HERO_HEIGHT + 10;

// max shots available to render at a time
pub const SHOOT_MAX_ALLOC: usize = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShootOwner {
    Hero,
    Enemy,
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
            SHOOT_BOX_WIDTH,
            SHOOT_BOX_HEIGHT,
            SHOOT_BOX_COLOR,
        );
    }
}

impl Shoot {
    #[inline(always)]
    pub fn new(coordinates: Coordinates, owner: ShootOwner) -> Self {
        Shoot {
            structure: ActorStructure {
                sprite: None,
                width: SHOOT_BOX_WIDTH,
                height: SHOOT_BOX_HEIGHT,
                alive: true,
                coordinates,
            },
            owner,
        }
    }

    #[inline(always)]
    pub(crate) fn out_of_screen(&self) -> bool {
        let coordinates = self.structure.coordinates;
        (coordinates.y() as i32) - (self.structure.height as i32) <= 0
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

    #[inline(always)]
    pub fn is_hit<T: Actor>(&self, actor: &T) -> bool {
        let shoot_structure = self.structure;
        let shoot_coordinates = self.structure.coordinates;
        let actor_structure = actor.get_structure();
        let actor_coordinates = actor_structure.coordinates;

        let shoot_x = shoot_coordinates.x();
        let shoot_y = shoot_coordinates.y();
        let x = actor_coordinates.x();
        let y = actor_coordinates.y();

        let shoot_x_end = shoot_x + shoot_structure.width;
        let shoot_y_end = shoot_y + shoot_structure.height;
        let x_end = x + actor_structure.width;
        let y_end = y + actor_structure.height;

        x <= shoot_x_end && shoot_x <= x_end && y <= shoot_y_end && shoot_y <= y_end
            || x <= shoot_x_end && shoot_x <= x_end && y_end <= shoot_y_end && shoot_y <= y
            || x <= shoot_x_end && y <= shoot_y_end && shoot_y <= y && shoot_x <= x
            || x <= shoot_x_end && y_end <= shoot_y_end && shoot_y <= y && shoot_x <= x
            || y <= shoot_y_end && shoot_y <= y && x_end <= shoot_x_end && shoot_x <= x
            || y_end <= shoot_y_end && shoot_y <= y && x_end <= shoot_x_end && shoot_x <= x
    }
}

/*
#[cfg(test)]
mod test {
    #[macro_use]
    extern crate std;

    use crate::actor::{ActorStructure, Shoot, ShootOwner};

    #[test]
    pub fn test_hit() {
        let shoot = Shoot {
            structure: ActorStructure {
                sprite: &[0; 4],
                width: 1,
                height: 1,
                alive: true,
                coordinates: super::Coordinates { x: 0, y: 0 },
            },
            owner: ShootOwner::Hero,
        };
        let coordinates = super::Coordinates { x: 0, y: 0 };
        assert!(shoot.is_hit(&coordinates));
    }
}*/
