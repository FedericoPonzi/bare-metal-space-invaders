use crate::actor::{Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use crate::{SCREEN_MARGIN, SCREEN_WIDTH};
use core::mem;

const SPRITE_SIZE: usize = 5120;
const ENEMY: &[u8; SPRITE_SIZE] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien.data");
const ENEMY_WIDTH: u32 = 40;
const ENEMY_HEIGHT: u32 = 32;

const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 10;

const ALIEN_ROWS: u32 = 3;
const ALIEN_COLS: u32 = ((SCREEN_WIDTH - SCREEN_MARGIN * 2) as u32
    / (ENEMY_WIDTH + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW));

const ENEMY_SPEED_PER_MS: u32 = 10; // 10 pixels per millisecond

pub const TOTAL_ENEMIES: usize = (ALIEN_ROWS * ALIEN_COLS) as usize;

#[derive(Copy, Clone)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
}
impl Default for Enemy {
    fn default() -> Self {
        let enemy_sprite: &[u32; SPRITE_SIZE / 4] = unsafe { mem::transmute(ENEMY) };

        Enemy {
            structure: ActorStructure {
                sprite: enemy_sprite,
                width: ENEMY_WIDTH,
                height: ENEMY_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(0, 0),
            },
        }
    }
}
impl Enemy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Actor for Enemy {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
}

pub fn init_enemies() -> [Enemy; TOTAL_ENEMIES] {
    let mut enemies = [Enemy::new(); TOTAL_ENEMIES];
    for x in 0..ALIEN_COLS {
        let offset_x = ENEMY_WIDTH * x + (BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW * x);
        for y in 0..ALIEN_ROWS {
            let offset_y = ENEMY_HEIGHT * y + SCREEN_MARGIN as u32;
            enemies[(y * ALIEN_COLS + x) as usize].structure.coordinates =
                Coordinates::new(offset_x, offset_y);
        }
    }
    enemies
}

pub fn move_enemies(offset: u32, offset_y: u32, aliens: &mut [Enemy; TOTAL_ENEMIES]) {
    for x in 0..ALIEN_COLS {
        let offset_x = ENEMY_WIDTH * x + (offset + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW * x);
        for y in 0..ALIEN_ROWS {
            let offset_y = ENEMY_HEIGHT * y + offset_y;
            let index = (y * ALIEN_COLS + x) as usize;
            if aliens[index].structure.alive {
                aliens[index].move_to(Coordinates::new(offset_x, offset_y));
            }
        }
    }
}
