use crate::actor::{Actor, ActorStructure, HERO_HEIGHT, HERO_SPRITE_U32, HERO_WIDTH};
use crate::{Coordinates, FrameBufferInterface, SCREEN_MARGIN};

const UI_LIFES_X: u32 = SCREEN_MARGIN / 2;
const UI_LIFES_Y: u32 = SCREEN_MARGIN / 2;
const UI_LIFES_X_OFFSET_BETWEEN_LIFES: u32 = 20;

pub struct LivesCount {
    pub(crate) count: u8,
    structure: ActorStructure,
}

impl LivesCount {
    pub fn new(lives: u8) -> Self {
        Self {
            count: lives,
            structure: ActorStructure::new(Coordinates::new(0, 0)),
        }
    }
    pub fn is_out_of_lives(&self) -> bool {
        self.count == 0
    }
    pub fn decrease(&mut self) {
        self.count -= 1;
    }
}

impl Actor for LivesCount {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }

    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        const COORDINATES: [Coordinates; 3] = [
            Coordinates::new(UI_LIFES_X, UI_LIFES_Y),
            Coordinates::new(
                UI_LIFES_X + (HERO_WIDTH + UI_LIFES_X_OFFSET_BETWEEN_LIFES),
                UI_LIFES_Y,
            ),
            Coordinates::new(
                UI_LIFES_X + 2 * (HERO_WIDTH + UI_LIFES_X_OFFSET_BETWEEN_LIFES),
                UI_LIFES_Y,
            ),
        ];
        for coordinate in COORDINATES.iter().take(self.count as usize) {
            fb.display_image(coordinate, HERO_SPRITE_U32, HERO_WIDTH, HERO_HEIGHT);
        }
    }
}
