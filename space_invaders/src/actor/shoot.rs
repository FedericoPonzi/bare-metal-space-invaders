use crate::actor::{Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use core::mem;

const SHOOT: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");

const SHOOT_WIDTH: u32 = 46;
const SHOOT_HEIGHT: u32 = 29;

const SHOOT_MOVEMENT_OFFSET: u32 = 20;

// max shots available to render at a time
pub const SHOOT_MAX_ALLOC: usize = 30;

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
}

impl Shoot {
    pub fn new(coordinates: Coordinates, owner: ShootOwner) -> Self {
        let shoot_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(SHOOT) };
        Shoot {
            structure: ActorStructure {
                sprite: shoot_sprite,
                width: SHOOT_WIDTH,
                height: SHOOT_HEIGHT,
                alive: true,
                coordinates,
            },
            owner,
        }
    }

    pub(crate) fn move_forward(&mut self) {
        if self.owner == ShootOwner::Hero {
            self.structure.coordinates.sub_y(SHOOT_MOVEMENT_OFFSET);
        } else {
            self.structure.coordinates.add_y(SHOOT_MOVEMENT_OFFSET);
        }
    }

    pub fn is_hit(&self, coordinates: &Coordinates) -> bool {
        let shoot_structure = self.structure;
        let shoot_coordinates = shoot_structure.coordinates;
        let shoot_width = shoot_structure.width;
        let shoot_height = shoot_structure.height;
        let shoot_x = shoot_coordinates.x;
        let shoot_y = shoot_coordinates.y;
        let x = coordinates.x;
        let y = coordinates.y;
        let shoot_x_end = shoot_x + shoot_width;
        let shoot_y_end = shoot_y + shoot_height;
        let x_end = x + shoot_width;
        let y_end = y + shoot_height;
        // TODO: verify :D
        x >= shoot_x && x <= shoot_x_end && y >= shoot_y && y <= shoot_y_end
            || x_end >= shoot_x && x_end <= shoot_x_end && y >= shoot_y && y <= shoot_y_end
            || x >= shoot_x && x <= shoot_x_end && y_end >= shoot_y && y_end <= shoot_y_end
            || x_end >= shoot_x && x_end <= shoot_x_end && y_end >= shoot_y && y_end <= shoot_y_end
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