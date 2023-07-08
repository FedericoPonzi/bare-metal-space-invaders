use crate::framebuffer::color::{BLACK_COLOR, WHITE_COLOR};
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::{HeroMovementDirection, StdFrameBuffer, HERO};
use std::mem;

const HERO_MOVEMENT_OFFSET: u32 = 10;
const SHOOT_MOVEMENT_OFFSET: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorStructure {
    // TODO: can probably use RC instead
    pub sprite: &'static [u32],
    pub width: u32,
    pub height: u32,
    pub alive: bool,
    // Top left offset
    pub coordinates: Coordinates,
}

pub trait Actor {
    fn get_structure(&self) -> &ActorStructure;
    fn set_coordinates(&mut self, coordinates: Coordinates);

    #[inline(always)]
    fn move_to(&mut self, top_left_offset: Coordinates) {
        self.set_coordinates(top_left_offset);
    }

    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        let structure = self.get_structure();
        fb.draw_rect_fill(
            structure.coordinates,
            structure.width,
            structure.height,
            WHITE_COLOR,
        );
        fb.display_image(structure.coordinates, structure.sprite, structure.width);
    }
}

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

#[derive(Copy, Clone)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
}
impl Actor for Enemy {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
}

#[derive(Copy, Clone)]
pub struct Hero {
    pub(crate) structure: ActorStructure,
}

impl Hero {
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

#[cfg(test)]
mod test {
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
}
