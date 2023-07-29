use crate::actor::{Actor, ActorStructure, Sprite, HERO_HEIGHT};
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::framebuffer::Coordinates;
use crate::{MemoryAllocator, SCREEN_HEIGHT, SCREEN_MARGIN, SCREEN_WIDTH};

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

static mut GREEN_ENEMY_ALIGNED: Option<&'static [u32]> = None;
static mut RED_ENEMY_ALIGNED: Option<&'static [u32]> = None;
static mut ENEMY_ALIGNED: Option<&'static [u32]> = None;

#[derive(Copy, Clone, Debug)]
pub struct Enemy {
    pub(crate) structure: ActorStructure,
    // every loop iteration, might cause a sub-pixel movement.
}

impl Enemy {
    fn new() -> Self {
        Enemy {
            structure: ActorStructure {
                sprite: Some(unsafe { Sprite::new(ENEMY_ALIGNED.unwrap()) }),
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
        unsafe {
            self.structure.sprite = Some(Sprite::new(GREEN_ENEMY_ALIGNED.unwrap()));
        }
    }

    pub fn set_red_alien(&mut self) {
        const ENEMY_RED_WIDTH: u32 = 39;
        const ENEMY_RED_HEIGHT: u32 = 31;
        self.structure.width = ENEMY_RED_WIDTH;
        self.structure.height = ENEMY_RED_HEIGHT;
        unsafe {
            self.structure.sprite = Some(Sprite::new(RED_ENEMY_ALIGNED.unwrap()));
        }
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
    direction: EnemiesDirection,
}
impl Enemies {
    pub(crate) fn new<A>(fb: &A) -> Self
    where
        A: MemoryAllocator,
    {
        Self {
            enemies: Self::init_enemies(fb),
            lowest_col: (0, 0),
            largest_col: (0, 0),
            enemies_dead: 0,
            direction: EnemiesDirection::Right,
        }
    }

    #[inline(always)]
    pub fn init_enemies<A>(fb: &A) -> [Enemy; TOTAL_ENEMIES]
    where
        A: MemoryAllocator,
    {
        unsafe {
            if GREEN_ENEMY_ALIGNED.is_none() {
                const ENEMY_GREEN: &[u8] = include_bytes!(
                    "/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/green.data"
                );

                GREEN_ENEMY_ALIGNED = Some(Sprite::align_allocated_u32(ENEMY_GREEN, fb));
            }
            if RED_ENEMY_ALIGNED.is_none() {
                const ENEMY_RED: &[u8] = include_bytes!(
                    "/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/red.data"
                );
                RED_ENEMY_ALIGNED = Some(Sprite::align_allocated_u32(ENEMY_RED, fb));
            }
            if ENEMY_ALIGNED.is_none() {
                pub const ENEMY: &[u8] = include_bytes!(
                    "/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien.data"
                );
                ENEMY_ALIGNED = Some(Sprite::align_allocated_u32(ENEMY, fb));
            }
        }

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
    #[inline(always)]
    pub(crate) fn move_enemies(&mut self, delta_ms: u64) {
        // determine the direction.

        let lowest_enemy =
            self.enemies[(self.lowest_col.1 * ENEMY_COLS + self.lowest_col.0) as usize];
        let largest_enemy =
            self.enemies[(self.largest_col.1 * ENEMY_COLS + self.largest_col.0) as usize];
        let right_limit = self.direction == EnemiesDirection::Right
            && largest_enemy.structure.coordinates.x() + ENEMY_WIDTH
                >= (SCREEN_WIDTH - SCREEN_MARGIN);

        self.lowest_col = (ENEMY_COLS, 0);
        self.largest_col = (0, 0);

        let left_limit = self.direction == EnemiesDirection::Left
            && lowest_enemy.structure.coordinates.x() <= SCREEN_MARGIN;
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
        // speed up per dead enemy
        let speedup = (self.enemies_dead as f64 * 2.0) / TOTAL_ENEMIES as f64;
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
    #[inline(always)]
    pub fn draw(&self, fb: &mut impl FrameBufferInterface) {
        for enemy in self.enemies {
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
    #[inline(always)]
    fn invert_direction(&self) -> Self {
        use EnemiesDirection::{Left, Right};
        match self {
            Right => Left,
            Left => Right,
        }
    }
    #[inline(always)]
    fn to_offset(&self, delta_ms: u64, speedup: f64) -> i32 {
        use EnemiesDirection::{Left, Right};
        let delta_ms = delta_ms as f64;
        let sign = match self {
            Right => 1.0,
            Left => -1.0,
        };
        let ret = sign * (ENEMY_SPEED_PER_MS + (speedup * ENEMY_SPEED_PER_MS)) * delta_ms;
        ret as i32
    }
}
