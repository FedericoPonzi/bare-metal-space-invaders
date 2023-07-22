use crate::actor::{Actor, ActorStructure, Sprite, SHOOT_ENEMY_MAX};
use crate::framebuffer::coordinates::Coordinates;
use crate::{FrameBufferInterface, Shoot, SCREEN_HEIGHT, SCREEN_MARGIN, SCREEN_WIDTH};

pub const ENEMY: &[u8] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien.data");
const ENEMY_WIDTH: u32 = 40;
const ENEMY_HEIGHT: u32 = 32;

const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 15;
const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL: u32 = 15;

const ENEMY_ROWS: u32 = 4;
pub const ENEMY_COLS: u32 = ((SCREEN_WIDTH - SCREEN_MARGIN * 2) as u32
    / (ENEMY_WIDTH + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW))
    - 10;

/// by how many pixel should the enemy go down
pub const ENEMY_STEP_DOWN: usize = (SCREEN_HEIGHT - SCREEN_MARGIN) / ENEMY_HEIGHT as usize;

const ENEMY_SPEED_PER_MS: i32 = 25; // pixels per second

pub const TOTAL_ENEMIES: usize = (ENEMY_ROWS * ENEMY_COLS) as usize;

#[derive(Copy, Clone)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
    // every loop iteration, might cause a sub-pixel movement.
}

impl Enemy {
    pub fn new(fb: &impl FrameBufferInterface) -> Self {
        Enemy {
            structure: ActorStructure {
                sprite: Some(Sprite::new(ENEMY, fb)),
                width: ENEMY_WIDTH,
                height: ENEMY_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(0, 0),
            },
        }
    }

    pub fn set_green_alien(&mut self, fb: &impl FrameBufferInterface) {
        const ENEMY_GREEN: &[u8] =
            include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/green.data");
        const ENEMY_GREEN_WIDTH: u32 = 40;
        const ENEMY_GREEN_HEIGHT: u32 = 32;
        self.structure.width = ENEMY_GREEN_WIDTH;
        self.structure.height = ENEMY_GREEN_HEIGHT;

        self.structure.sprite = Some(Sprite::new(ENEMY_GREEN, fb));
    }

    pub fn set_red_alien(&mut self, fb: &impl FrameBufferInterface) {
        const ENEMY_RED: &[u8] =
            include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/red.data");
        const ENEMY_RED_WIDTH: u32 = 39;
        const ENEMY_RED_HEIGHT: u32 = 31;
        self.structure.width = ENEMY_RED_WIDTH;
        self.structure.height = ENEMY_RED_HEIGHT;

        self.structure.sprite = Some(Sprite::new(ENEMY_RED, fb));
    }
}

impl Actor for Enemy {
    #[inline(always)]
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }
    #[inline(always)]
    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
}

pub struct Enemies {
    pub(crate) enemies: [Enemy; TOTAL_ENEMIES],
    // used for speedup calculation and high score
    pub(crate) enemies_dead: usize,
    lowest_col: (u32, u32),
    largest_col: (u32, u32),
    pub(crate) enemy_shoots: usize,
    direction: EnemiesDirection,
}
impl Enemies {
    pub(crate) fn new(fb: &impl FrameBufferInterface) -> Self {
        Self {
            enemies: Self::init_enemies(fb),
            lowest_col: (0, 0),
            largest_col: (0, 0),
            enemies_dead: 0,
            enemy_shoots: 0,
            direction: EnemiesDirection::Right,
        }
    }

    #[inline(always)]
    pub fn init_enemies(fb: &impl FrameBufferInterface) -> [Enemy; TOTAL_ENEMIES] {
        let mut enemies = [Enemy::new(fb); TOTAL_ENEMIES];

        for x in 0..ENEMY_COLS {
            let offset_x =
                SCREEN_MARGIN as u32 + ENEMY_WIDTH * x + (BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW * x);

            for y in 0..ENEMY_ROWS {
                let offset_y = (ENEMY_HEIGHT + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL) * y
                    + SCREEN_MARGIN as u32;

                enemies[(y * ENEMY_COLS + x) as usize].structure.coordinates =
                    Coordinates::new(offset_x, offset_y);

                if y == 1 {
                    enemies[(y * ENEMY_COLS + x) as usize].set_green_alien(fb);
                }
                if y >= 2 {
                    enemies[(y * ENEMY_COLS + x) as usize].set_red_alien(fb);
                }
            }
        }
        enemies
    }
    pub(crate) fn handle_enemies_shoot(&mut self, rnd: u32, shoots: &mut [Option<Shoot>]) {
        if self.enemy_shoots < SHOOT_ENEMY_MAX {
            let enemy_shooting = rnd as usize % (TOTAL_ENEMIES - self.enemies_dead);
            for (id, enemy) in self
                .enemies
                .iter()
                .filter(|e| e.structure.alive)
                .enumerate()
            {
                if enemy_shooting == id {
                    for sh in shoots.iter_mut() {
                        if sh.is_none() {
                            sh.replace(Shoot::from(enemy));
                            self.enemy_shoots += 1;
                            break;
                        }
                    }
                }
            }
        }
    }

    /// largest_x is the largest x coordinate of still alive enemy
    /// lowest_x is the lowest x coordinate of still alive enemy
    #[inline(always)]
    pub(crate) fn move_enemies(&mut self, delta_ms: u64) {
        // determine the direction.

        // speed up per dead enemy
        let speedup =
            (ENEMY_SPEED_PER_MS as f32 * (self.enemies_dead as f32 / TOTAL_ENEMIES as f32)) as i32;
        let speedup = if self.direction == EnemiesDirection::Right {
            speedup
        } else {
            -speedup
        };

        let lowest_enemy =
            self.enemies[(self.lowest_col.1 * ENEMY_COLS + self.lowest_col.0) as usize];
        let largest_enemy =
            self.enemies[(self.largest_col.1 * ENEMY_COLS + self.largest_col.0) as usize];
        let right_limit = self.direction == EnemiesDirection::Right
            && largest_enemy.structure.coordinates.x() + ENEMY_WIDTH
                >= (SCREEN_WIDTH - SCREEN_MARGIN) as u32;

        self.lowest_col = (ENEMY_COLS, 0);
        self.largest_col = (0, 0);

        let left_limit = self.direction == EnemiesDirection::Left
            && lowest_enemy.structure.coordinates.x() <= SCREEN_MARGIN as u32;
        if left_limit || right_limit {
            // move down one row, invert direction
            for x in 0..ENEMY_COLS {
                for y in 0..ENEMY_ROWS {
                    let index = (y * ENEMY_COLS + x) as usize;
                    let enemy = &mut self.enemies[index];

                    let new_y = enemy.structure.coordinates.y() + ENEMY_STEP_DOWN as u32;
                    if enemy.structure.alive {
                        enemy.move_to(Coordinates::new(enemy.structure.coordinates.x(), new_y));
                    }
                    if core::cmp::max(self.largest_col.0, x) == x {
                        self.largest_col = (x, y);
                    }
                    if core::cmp::min(self.lowest_col.0, x) == x {
                        self.lowest_col = (x, y);
                    }
                }
            }
            self.direction = self.direction.invert_direction();
            return;
        }

        let offset_x = self.direction.to_offset(delta_ms, speedup);

        for x in 0..ENEMY_COLS {
            for y in 0..ENEMY_ROWS {
                let index = (y * ENEMY_COLS + x) as usize;
                let e = &mut self.enemies[index];
                if !e.structure.alive {
                    continue;
                }

                e.structure.coordinates.add_virtual_x(offset_x as f64);

                if core::cmp::max(self.largest_col.0, x) == x {
                    self.largest_col = (x, y);
                }
                if core::cmp::min(self.lowest_col.0, x) == x {
                    self.lowest_col = (x, y);
                }
            }
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum EnemiesDirection {
    Right,
    Left,
}
impl EnemiesDirection {
    #[inline(always)]
    fn invert_direction(&self) -> Self {
        use EnemiesDirection::{Left, Right};
        match self {
            Right => Left,
            Left => Right,
        }
    }
    #[inline(always)]
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
