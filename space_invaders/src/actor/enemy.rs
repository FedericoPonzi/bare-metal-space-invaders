use crate::actor::{Actor, ActorStructure, Sprite, HERO_HEIGHT};
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::framebuffer::Coordinates;
use crate::{SCREEN_HEIGHT, SCREEN_MARGIN, SCREEN_WIDTH};

const ENEMY_WIDTH: u32 = 40;
const ENEMY_HEIGHT: u32 = 32;

const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 20;
const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL: u32 = 25;

const ENEMY_ROWS: u32 = 4;
pub const ENEMY_COLS: u32 = ((SCREEN_WIDTH - SCREEN_MARGIN * 2)
    / (ENEMY_WIDTH + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW))
    - 10;
const ENEMY_OFFSET_Y_FROM_MARGIN: u32 = HERO_HEIGHT;

/// by how many pixel should the enemy go down
pub const ENEMY_STEP_DOWN: u32 = (SCREEN_HEIGHT - SCREEN_MARGIN) / ENEMY_HEIGHT;

const ENEMY_SPEED_PER_MS: f64 = 20.0 / 1000.0; // pixels per second

pub const TOTAL_ENEMIES: usize = (ENEMY_ROWS * ENEMY_COLS) as usize;

static GREEN_ENEMY_SPRITE: &[u32] =
    crate::include_bytes_align_as!(u32, "../../../assets/green.data");
static ENEMY_RED_SPRITE: &[u32] = crate::include_bytes_align_as!(u32, "../../../assets/red.data");
pub static ENEMY_SPRITE: &[u32] = crate::include_bytes_align_as!(u32, "../../../assets/alien.data");

#[derive(Copy, Clone, Debug)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
    // every loop iteration, might cause a sub-pixel movement.
}

impl Enemy {
    fn new() -> Self {
        Enemy {
            structure: ActorStructure {
                sprite: Some(Sprite::new(ENEMY_SPRITE)),
                width: ENEMY_WIDTH,
                height: ENEMY_HEIGHT,
                alive: true,
                coordinates: Coordinates::new(0, 0),
            },
        }
    }

    pub fn set_green_alien(&mut self) {
        const ENEMY_GREEN_WIDTH: u32 = 40;
        const ENEMY_GREEN_HEIGHT: u32 = 32;
        self.structure.width = ENEMY_GREEN_WIDTH;
        self.structure.height = ENEMY_GREEN_HEIGHT;
        self.structure.sprite = Some(Sprite::new(GREEN_ENEMY_SPRITE));
    }

    pub fn set_red_alien(&mut self) {
        const ENEMY_RED_WIDTH: u32 = 39;
        const ENEMY_RED_HEIGHT: u32 = 31;
        self.structure.width = ENEMY_RED_WIDTH;
        self.structure.height = ENEMY_RED_HEIGHT;
        self.structure.sprite = Some(Sprite::new(ENEMY_RED_SPRITE));
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

pub struct Enemies {
    pub(crate) enemies: [Enemy; TOTAL_ENEMIES],
    // used for speedup calculation and high score
    pub(crate) enemies_dead: usize,
    lowest_col: (u32, u32),
    largest_col: (u32, u32),
    direction: EnemiesDirection,
}
impl Enemies {
    pub fn all_dead(&self) -> bool {
        TOTAL_ENEMIES - self.enemies_dead == 0
    }
    pub(crate) fn new() -> Self {
        Self {
            enemies: Self::init_enemies(),
            lowest_col: (0, 0),
            largest_col: (0, 0),
            enemies_dead: 0,
            direction: EnemiesDirection::Right,
        }
    }

    pub fn init_enemies() -> [Enemy; TOTAL_ENEMIES] {
        let mut enemies = [Enemy::new(); TOTAL_ENEMIES];

        for x in 0..ENEMY_COLS {
            let offset_x =
                SCREEN_MARGIN + ENEMY_WIDTH * x + (BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW * x);

            for y in 0..ENEMY_ROWS {
                let index = (y * ENEMY_COLS + x) as usize;
                let offset_y = (ENEMY_HEIGHT + BASE_OFFSET_IN_BETWEEN_ALIENS_IN_COL) * y
                    + SCREEN_MARGIN
                    + ENEMY_OFFSET_Y_FROM_MARGIN;

                enemies[index].structure.coordinates = Coordinates::new(offset_x, offset_y);
                if y == 1 {
                    enemies[index].set_green_alien();
                } else if y > 1 {
                    enemies[index].set_red_alien();
                }
            }
        }
        enemies
    }

    /// largest_x is the largest x coordinate of still alive enemy
    /// lowest_x is the lowest x coordinate of still alive enemy
    pub(crate) fn move_enemies(&mut self, delta_ms: u64) {
        // determine the direction.

        let lowest_enemy =
            self.enemies[(self.lowest_col.1 * ENEMY_COLS + self.lowest_col.0) as usize];
        let largest_enemy =
            self.enemies[(self.largest_col.1 * ENEMY_COLS + self.largest_col.0) as usize];
        let right_limit = self.direction == EnemiesDirection::Right
            && largest_enemy.get_coordinates().x() + ENEMY_WIDTH >= (SCREEN_WIDTH - SCREEN_MARGIN);

        self.lowest_col = (ENEMY_COLS, 0);
        self.largest_col = (0, 0);

        let left_limit = self.direction == EnemiesDirection::Left
            && lowest_enemy.get_coordinates().x() <= SCREEN_MARGIN;
        if left_limit || right_limit {
            // move down one row, invert direction
            for x in 0..ENEMY_COLS {
                for y in 0..ENEMY_ROWS {
                    let index = (y * ENEMY_COLS + x) as usize;
                    let enemy = &mut self.enemies[index];

                    let new_y = enemy.get_coordinates().y() + ENEMY_STEP_DOWN as u32;
                    if enemy.is_alive() {
                        enemy.move_to(Coordinates::new(enemy.get_coordinates().x(), new_y));
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
        // speed up per dead enemy
        let speedup = (self.enemies_dead as f64 * 2.0) / TOTAL_ENEMIES as f64;
        let offset_x = self.direction.to_offset(delta_ms, speedup);

        for x in 0..ENEMY_COLS {
            for y in 0..ENEMY_ROWS {
                let index = (y * ENEMY_COLS + x) as usize;
                let e = &mut self.enemies[index];
                if !e.is_alive() {
                    continue;
                }

                e.structure.coordinates.add_virtual_x(offset_x);

                if core::cmp::max(self.largest_col.0, x) == x {
                    self.largest_col = (x, y);
                }
                if core::cmp::min(self.lowest_col.0, x) == x {
                    self.lowest_col = (x, y);
                }
            }
        }
    }
    pub fn draw(&self, fb: &mut impl FrameBufferInterface) {
        for enemy in self.enemies.iter().filter(|e| e.is_alive()) {
            enemy.draw(fb);
        }
    }
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
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
    fn to_offset(&self, delta_ms: u64, speedup: f64) -> f64 {
        use EnemiesDirection::{Left, Right};
        let delta_ms = delta_ms as f64;
        let sign = match self {
            Right => 1.0,
            Left => -1.0,
        };
        let ret = sign * (ENEMY_SPEED_PER_MS + (speedup * ENEMY_SPEED_PER_MS)) * delta_ms;
        ret
    }
}
