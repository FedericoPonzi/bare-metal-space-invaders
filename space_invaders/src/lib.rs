use minifb::{Key, KeyRepeat};
use std::mem;
use std::time::Duration;

mod actor;
mod framebuffer;
mod time;
pub use framebuffer::StdFrameBuffer;

extern crate alloc;

use crate::actor::{Actor, ActorStructure, Enemy, Shoot};
use crate::actor::{Hero, ShootOwner};
use crate::framebuffer::fb_trait::FrameBufferInterface;
use framebuffer::coordinates::Coordinates;
use time::TimeManagerInterface;

const ENEMY: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const ENEMY_WIDTH: u32 = 46;
const ENEMY_HEIGHT: u32 = 29;

const HERO: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const HERO_WIDTH: u32 = 46;
const HERO_HEIGHT: u32 = 29;

const SHOOT: &[u8; 5336] =
    include_bytes!("/home/fponzi/dev/rust/bare-metal-spaceinvaders/assets/alien-1.data");
const SHOOT_WIDTH: u32 = 46;
const SHOOT_HEIGHT: u32 = 29;

pub fn run_game(mut fb: StdFrameBuffer) {
    loop {
        init_game(&mut fb);
    }
}

const ALIEN_ROWS: u32 = 4;
const ALIEN_COLS: u32 = 15;
const TOTAL_ENEMIES: usize = (ALIEN_ROWS * ALIEN_COLS) as usize;

fn init_enemies() -> [Enemy; TOTAL_ENEMIES] {
    let enemy_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(ENEMY) };

    let mut enemies = [Enemy {
        structure: ActorStructure {
            sprite: enemy_sprite,
            width: ENEMY_WIDTH,
            height: ENEMY_HEIGHT,
            alive: true,
            coordinates: Coordinates::new(0, 0),
        },
    }; TOTAL_ENEMIES];
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

fn init_game(fb: &mut StdFrameBuffer) {
    let hero_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(HERO) };
    let shoot_sprite: &[u32; 5336 / 4] = unsafe { mem::transmute(SHOOT) };
    let mut aliens: [Enemy; TOTAL_ENEMIES] = init_enemies();

    const BASE_OFFSET_IN_BETWEEN_ALIENS_IN_ROW: u32 = 10;
    let mut offset_y = 0;
    let mut shoots: Vec<Shoot> = Vec::new();
    let mut hero = Hero {
        structure: ActorStructure {
            sprite: hero_sprite,
            width: HERO_WIDTH,
            height: HERO_HEIGHT,
            alive: true,
            coordinates: Coordinates::new(1000, 1000),
        },
    };

    let mut direction = 0;
    let mut direction_index = 1i32;

    while fb.window.is_open() && !fb.window.is_key_down(Key::Escape) {
        let mut hero_movement_direction = HeroMovementDirection::Still;
        fb.window
            .get_keys()
            //.get_keys_pressed(KeyRepeat::Yes)
            .iter()
            .for_each(|key| match key {
                Key::A | Key::Left => {
                    hero_movement_direction = HeroMovementDirection::Left;
                }
                Key::D | Key::Right => {
                    hero_movement_direction = HeroMovementDirection::Right;
                }
                Key::Space => {
                    let shoot = Shoot {
                        structure: ActorStructure {
                            sprite: shoot_sprite,
                            width: SHOOT_WIDTH,
                            height: SHOOT_HEIGHT,
                            alive: true,
                            coordinates: Coordinates::new(
                                hero.structure.coordinates.x,
                                hero.structure.coordinates.y - 100,
                            ),
                        },
                        owner: ShootOwner::Hero,
                    };
                    println!("pew!");
                    shoots.push(shoot);
                }
                _ => {
                    hero_movement_direction = HeroMovementDirection::Still;
                }
            });
        let mut new_shoots: Vec<Shoot> = Vec::new();
        for mut shoot in shoots {
            shoot.move_forward();
            if !out_of_screen(&shoot) {
                new_shoots.push(shoot);
            }
        }
        shoots = new_shoots;
        fb.clear_screen();
        let offset = 10 * direction;
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

        // collision detection
        let mut new_shoots = Vec::new();
        for shoot in shoots {
            match shoot.owner {
                ShootOwner::Enemy => {
                    if shoot.is_hit(&hero.structure.coordinates) {
                        println!("Hero is dead!");
                    } else {
                        new_shoots.push(shoot);
                    }
                }
                ShootOwner::Hero => {
                    let mut has_hit = false;
                    for alien in aliens.iter_mut().filter(|a| a.structure.alive) {
                        if shoot.is_hit(&alien.structure.coordinates) {
                            alien.structure.alive = false;
                            println!("Alien is dead!");
                            has_hit = true;
                            break;
                        }
                    }
                    if !has_hit {
                        new_shoots.push(shoot);
                    }
                }
            }
        }
        shoots = new_shoots;

        if !hero.structure.alive {
            println!("Game over!");
            return;
        }

        let mut alive = false;
        for enemy in aliens.iter() {
            alive = alive || enemy.structure.alive;
            if enemy.structure.coordinates.y >= hero.structure.coordinates.y {
                println!("Game over!");
                return;
            }
        }
        if !alive {
            println!("Game over, you won!");
            return;
        }

        // draw things:
        for enemy in aliens.iter() {
            if enemy.structure.alive {
                enemy.draw(fb)
            }
        }

        hero.handle_movement(hero_movement_direction);
        hero.draw(fb);
        for shoot in shoots.iter() {
            shoot.draw(fb);
        }
        fb.update();
        direction = direction.saturating_add_signed(direction_index);
        if direction == 8 || direction == 0 {
            direction_index = -direction_index;
            offset_y += 10;
        }
    }
}

fn out_of_screen(shoot: &Shoot) -> bool {
    let structure = shoot.structure;
    let coordinates = structure.coordinates;
    coordinates.x == 0
        || coordinates.x > (structure.width * structure.height)
        || coordinates.y == 0
        || coordinates.y > (structure.width * structure.height)
}

#[derive(Clone, Copy)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
}
