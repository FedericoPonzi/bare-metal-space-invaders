use crate::actor::{Actor, ActorStructure};
use crate::framebuffer::coordinates::Coordinates;
use crate::{SCREEN_MARGIN, SCREEN_WIDTH};
use core::mem;
use log::info;

const SPRITE_SIZE: usize = 5120;
const ENEMY: &[u8; SPRITE_SIZE] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien.data");
const ENEMY_WIDTH: u32 = 40;
const ENEMY_HEIGHT: u32 = 32;

const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 15;
const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL: u32 = 15;

const ALIEN_ROWS: u32 = 4;
pub const ALIEN_COLS: u32 = ((SCREEN_WIDTH - SCREEN_MARGIN * 2) as u32
    / (ENEMY_WIDTH + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW))
    - 5;

/// by how many pixel should the enemy go down
pub const ENEMY_STEP_DOWN: usize = 10; // TODO: should be calculate based on rows, screen size and alien size

const ENEMY_SPEED_PER_MS: i32 = 20; // pixels per second

pub const TOTAL_ENEMIES: usize = (ALIEN_ROWS * ALIEN_COLS) as usize;

#[derive(Copy, Clone)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
    // every loop iteration, might cause a sub-pixel movement.
    // virtual x has the `real` value that is used to set the new x.
    virtual_x: f64,
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
            virtual_x: 0f64,
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
        let offset_x =
            SCREEN_MARGIN as u32 + ENEMY_WIDTH * x + (BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW * x);
        for y in 0..ALIEN_ROWS {
            let offset_y =
                (ENEMY_HEIGHT + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL) * y + SCREEN_MARGIN as u32;
            enemies[(y * ALIEN_COLS + x) as usize].structure.coordinates =
                Coordinates::new(offset_x, offset_y);
            enemies[(y * ALIEN_COLS + x) as usize].virtual_x = offset_x.into();
        }
    }
    info!("enemy 0: {:?}", enemies[0].structure.coordinates);
    enemies
}
#[derive(Eq, PartialEq, Debug)]
pub enum EnemiesDirection {
    Right,
    Left,
}
impl EnemiesDirection {
    fn invert_direction(&self) -> Self {
        use EnemiesDirection::{Left, Right};
        match self {
            Right => Left,
            Left => Right,
        }
    }
    fn to_offset(&self, delta_ms: u64, speedup: i32) -> i32 {
        use EnemiesDirection::{Left, Right};
        let delta_ms = delta_ms as i32;
        (match self {
            Right => (ENEMY_SPEED_PER_MS + speedup) * delta_ms,
            Left => (-ENEMY_SPEED_PER_MS - speedup) * delta_ms,
        } / 1000)
            + speedup
    }
}
/// largest_x is the largest x coordinate of still alive enemy
/// lowest_x is the lowest x coordinate of still alive enemy
pub fn move_enemies(
    offset_y: &mut usize,
    enemy: &mut [Enemy; TOTAL_ENEMIES],
    direction: EnemiesDirection,
    delta_ms: u64,
) -> EnemiesDirection {
    let mut lowest_col: Option<(u32, u32)> = None;
    let mut largest_col: Option<(u32, u32)> = None;
    let mut enemies_dead = 0;
    // determine the direction.
    for x in 0..ALIEN_COLS {
        for y in 0..ALIEN_ROWS {
            let enemy = enemy[(y * ALIEN_COLS + x) as usize];
            if !enemy.structure.alive {
                enemies_dead += 1;
                continue;
            }
            if largest_col.is_none() || core::cmp::max(largest_col.unwrap().0, x) == x {
                largest_col = Some((x, y));
            }
            if lowest_col.is_none() || core::cmp::min(lowest_col.unwrap().0, x) == x {
                lowest_col = Some((x, y));
            }
        }
    }
    // speed up per dead enemy
    let speedup = (ENEMY_SPEED_PER_MS as f32 * (enemies_dead as f32 / TOTAL_ENEMIES as f32)) as i32;
    let speedup = if direction == EnemiesDirection::Right {
        speedup
    } else {
        -speedup
    };
    let lowest_col = lowest_col.unwrap();
    let lowest_enemy = enemy[(lowest_col.1 * ALIEN_COLS + lowest_col.0) as usize];
    let largest_col = largest_col.unwrap();
    let largest_enemy = enemy[(largest_col.1 * ALIEN_COLS + largest_col.0) as usize];
    let right_limit = direction == EnemiesDirection::Right
        && largest_enemy.structure.coordinates.x + ENEMY_WIDTH
            >= (SCREEN_WIDTH - SCREEN_MARGIN) as u32;

    let left_limit = direction == EnemiesDirection::Left
        && lowest_enemy.structure.coordinates.x <= SCREEN_MARGIN as u32;
    if left_limit || right_limit {
        // move down one row, invert direction
        *offset_y += ENEMY_STEP_DOWN;
        for x in 0..ALIEN_COLS {
            for y in 0..ALIEN_ROWS {
                let index = (y * ALIEN_COLS + x) as usize;
                let new_y = enemy[index].structure.coordinates.y + *offset_y as u32;
                if enemy[index].structure.alive {
                    enemy[index].move_to(Coordinates::new(
                        enemy[index].structure.coordinates.x,
                        new_y,
                    ));
                }
            }
        }
        return direction.invert_direction();
    }

    for x in 0..ALIEN_COLS {
        let offset_x = direction.to_offset(delta_ms, speedup);
        for y in 0..ALIEN_ROWS {
            let index = (y * ALIEN_COLS + x) as usize;
            if enemy[index].structure.alive {
                enemy[index].virtual_x += offset_x as f64;
                enemy[index].move_to(Coordinates::new(
                    enemy[index].virtual_x.round() as u32,
                    enemy[index].structure.coordinates.y,
                ));
            }
        }
    }
    direction
}
