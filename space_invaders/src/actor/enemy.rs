use crate::actor::{scale_down_image, Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use core::mem;

const ENEMY: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const ENEMY_WIDTH: u32 = 46;
const ENEMY_HEIGHT: u32 = 29;

const ALIEN_ROWS: u32 = 4;
const ALIEN_COLS: u32 = 14;

const ENEMY_SPEED_PER_MS: u32 = 10; // 10 pixels per millisecond

pub const TOTAL_ENEMIES: usize = (ALIEN_ROWS * ALIEN_COLS) as usize;

const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 10;

#[derive(Copy, Clone)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
}

impl Enemy {
    pub fn new() -> Self {
        let enemy_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(ENEMY) };
        //let enemy_sprite =
        //    scale_down_image(enemy_sprite, ENEMY_WIDTH as usize, ENEMY_HEIGHT as usize, 2);

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
        let offset_x = ENEMY_WIDTH * x + (10 * x);
        for y in 0..ALIEN_ROWS {
            let offset_y = ENEMY_HEIGHT * y;
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
