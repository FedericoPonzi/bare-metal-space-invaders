use crate::actor::{
    ActorStructure, HERO_SPAWN_X, HERO_SPAWN_Y, SHOOT_BOX_HEIGHT, SHOOT_BOX_WIDTH,
    SHOOT_SPAWN_OFFSET_Y,
};
use crate::framebuffer::color::SHOT_COLOR;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::{Actor, FrameBufferInterface, SCREEN_MARGIN, SCREEN_WIDTH};
use log::info;

const BARRICADE_BOX_WIDTH: u32 = SHOOT_BOX_WIDTH + 10;
const BARRICADE_BOX_HEIGHT: u32 = SHOOT_BOX_HEIGHT + 10;
const BARRICADE_BOX_COLOR: Color = SHOT_COLOR;

// how much above the hero should the barricade spawn
// account for shoot spawn, in this way it's able to hit it.
const BARRICADE_OFFSET_FROM_HERO_Y: f64 = SHOOT_SPAWN_OFFSET_Y as f64 + 10.0;
// these are derived from the pattern in create_barricade.
const TOTAL_BLOCKS_PER_BARRICADE: usize = 14;
const BARRICADE_ROWS: f64 = 3.0;
const BARRICADE_COLS: u32 = 6;

const BARRICADE_OFFSET_Y: f64 = HERO_SPAWN_Y as f64
    - BARRICADE_OFFSET_FROM_HERO_Y
    - BARRICADE_ROWS * BARRICADE_BOX_HEIGHT as f64;

const TOTAL_BARRICADES: usize = 4;
// screen margin * 4 to add additional margin to the screen.
// it's divided by TOTAL_BARRICATES + 1 because it's the count of space inbetween barricade.
const BARRICADE_OFFSET_X: f64 =
    (SCREEN_WIDTH as u32 - SCREEN_MARGIN as u32 * 4 - BARRICADE_BOX_WIDTH * BARRICADE_COLS) as f64
        / (TOTAL_BARRICADES + 1) as f64;

// This is really a barricade block.
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
        if !self.structure.alive {
            return;
        }
        fb.draw_rect_fill(
            self.structure.coordinates,
            self.structure.width,
            self.structure.height,
            BARRICADE_BOX_COLOR,
        );
    }
}

impl Barricade {
    #[inline(always)]
    pub fn new(coordinates: Coordinates) -> Self {
        Barricade {
            structure: Self::structure(coordinates),
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

    // todo: probably these could be const functions.
    pub fn create_barricades() -> [Barricade; TOTAL_BLOCKS_PER_BARRICADE * TOTAL_BARRICADES] {
        let mut barricades =
            [Barricade::new(Coordinates::new(0, 0)); TOTAL_BLOCKS_PER_BARRICADE * TOTAL_BARRICADES];
        for i in 1..=TOTAL_BARRICADES {
            let x = SCREEN_MARGIN as u32 + BARRICADE_OFFSET_X as u32 * i as u32;
            let coordinate = Coordinates::new(x, BARRICADE_OFFSET_Y as u32);
            let new_b = Self::create_barricade(coordinate);
            barricades[(i - 1) * TOTAL_BLOCKS_PER_BARRICADE
                ..(i - 1) * TOTAL_BLOCKS_PER_BARRICADE + TOTAL_BLOCKS_PER_BARRICADE]
                .copy_from_slice(&new_b);
        }
        barricades
    }

    fn create_barricade(coordinates: Coordinates) -> [Barricade; TOTAL_BLOCKS_PER_BARRICADE] {
        let mut barricades = [Barricade::new(coordinates); TOTAL_BLOCKS_PER_BARRICADE];
        /*
        shape:
            xxxx
           xxxxxx
           xx  xx
          keep in sync with BARRICADE_ROWS const.
        */
        #[rustfmt::skip]
        const OFFSETS: [(f64, f64); TOTAL_BLOCKS_PER_BARRICADE] = [
                                                    (1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (4.0, 0.0),
                                        (0.0, 1.0), (1.0, 1.0), (2.0, 1.0), (3.0, 1.0), (4.0, 1.0), (5.0, 1.0),
                                        (0.0, 2.0), (1.0, 2.0),                         (4.0, 2.0), (5.0, 2.0),
        ];

        for (index, offset) in OFFSETS.iter().enumerate() {
            barricades[index]
                .structure
                .coordinates
                .add_virtual_x(offset.0 * BARRICADE_BOX_WIDTH as f64);
            barricades[index]
                .structure
                .coordinates
                .add_virtual_y(offset.1 * BARRICADE_BOX_HEIGHT as f64, 1);
        }

        barricades
    }
}
