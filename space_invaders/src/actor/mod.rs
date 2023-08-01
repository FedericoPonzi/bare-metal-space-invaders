mod barricade;
mod enemy;
mod hero;
mod lifes;
mod score_count;
pub(crate) mod shoot;

pub use barricade::*;
pub use enemy::*;
pub use hero::*;
pub use score_count::ScoreCount;
pub use shoot::*;

use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::fb_trait::FrameBufferInterface;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sprite {
    sprite: &'static [u32],
}
impl Sprite {
    pub fn new(sprite: &'static [u32]) -> Self {
        Self { sprite }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorStructure {
    pub sprite: Option<Sprite>,
    pub width: u32,
    pub height: u32,
    pub alive: bool,
    // Top left offset
    pub coordinates: Coordinates,
}

pub trait Actor {
    fn get_structure(&self) -> &ActorStructure;
    fn set_coordinates(&mut self, coordinates: Coordinates);
    fn is_alive(&self) -> bool {
        self.get_structure().alive
    }
    fn get_coordinates(&self) -> &Coordinates {
        &self.get_structure().coordinates
    }
    fn move_to(&mut self, top_left_offset: Coordinates) {
        self.set_coordinates(top_left_offset);
    }
    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        let structure = self.get_structure();
        fb.display_image(
            &structure.coordinates,
            structure.sprite.unwrap().sprite,
            structure.width,
            structure.height,
        );
    }

    fn is_hit(&self, actor_structure: &ActorStructure) -> bool {
        let self_structure = self.get_structure();
        let actor_structure = actor_structure;
        let actor_coordinates = &actor_structure.coordinates;

        let self_x = self.get_coordinates().x();
        let self_y = self.get_coordinates().y();
        let x = actor_coordinates.x();
        let y = actor_coordinates.y();

        let self_x_end = self_x + self_structure.width;
        let self_y_end = self_y + self_structure.height;
        let x_end = x + actor_structure.width;
        let y_end = y + actor_structure.height;

        x <= self_x_end && self_x <= x_end && y <= self_y_end && self_y <= y_end
            || x <= self_x_end && self_x <= x_end && y_end <= self_y_end && self_y <= y
            || x <= self_x_end && y <= self_y_end && self_y <= y && self_x <= x
            || x <= self_x_end && y_end <= self_y_end && self_y <= y && self_x <= x
            || y <= self_y_end && self_y <= y && x_end <= self_x_end && self_x <= x
            || y_end <= self_y_end && self_y <= y && x_end <= self_x_end && self_x <= x
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
