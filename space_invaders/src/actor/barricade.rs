use crate::actor::ActorStructure;
use crate::framebuffer::color::SHOT_COLOR;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::{Actor, FrameBufferInterface};
use log::info;

const BARRICADE_BOX_WIDTH: u32 = 3;
const BARRICADE_BOX_HEIGHT: u32 = 7;
const BARRICADE_BOX_COLOR: Color = SHOT_COLOR;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Barricade {
    pub(crate) structure: ActorStructure,
}

impl Actor for Barricade {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        fb.draw_rect_fill(
            self.structure.coordinates,
            BARRICADE_BOX_WIDTH,
            BARRICADE_BOX_HEIGHT,
            BARRICADE_BOX_COLOR,
        );
    }
}

impl Barricade {
    #[inline(always)]
    pub fn new(coordinates: Coordinates) -> Self {
        Barricade {
            structure: ActorStructure {
                sprite: None,
                width: BARRICADE_BOX_WIDTH,
                height: BARRICADE_BOX_HEIGHT,
                alive: true,
                coordinates,
            },
        }
    }

    const fn structure(coordinates: Coordinates) -> ActorStructure {
        ActorStructure {
            sprite: None,
            width: BARRICADE_BOX_WIDTH,
            height: BARRICADE_BOX_HEIGHT,
            alive: true,
            coordinates,
        }
    }
}
