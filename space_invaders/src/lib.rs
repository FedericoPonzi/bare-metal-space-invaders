#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]

pub mod actor;
mod framebuffer;

use alloc::vec::Vec;
use core::ops::Sub;

mod time;

extern crate alloc;

pub use crate::actor::{init_enemies, move_enemies, Actor, Shoot};
use crate::actor::{Hero, ShootOwner};
pub use crate::framebuffer::fb_trait::FrameBufferInterface;
pub use framebuffer::{Coordinates, Pixel};

use log::info;

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub fn run_game(mut fb: impl FrameBufferInterface, time_manager: impl TimeManagerInterface) {
    loop {
        info!("Starting game...");
        init_game(&mut fb, &time_manager);
    }
}

fn init_game(fb: &mut impl FrameBufferInterface, time_manager: &impl TimeManagerInterface) {
    let mut aliens = init_enemies();

    let mut offset_y = 0;
    let mut shoots: Vec<Shoot> = Vec::new();
    let mut hero = Hero::new();

    let mut direction = 0;
    let mut direction_index = 1i32;
    let mut enemy_last_movement = time_manager.now();
    let mut hero_last_movement = time_manager.now();
    let mut last_draw = time_manager.now();
    let mut last_loop = time_manager.now();
    loop {
        let loop_start = time_manager.now();
        let mut delta_ms = last_loop.as_millis();
        info!("delta_ms: {}", delta_ms);

        // 1. Get input
        let (hero_movement_direction, shoot) = fb.get_input_keys(&hero.structure.coordinates);
        if let Some(shoot) = shoot {
            shoots.push(shoot);
        }

        // 2. Movement
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
        if time_manager.since(enemy_last_movement).as_millis() > 100 {
            move_enemies(offset, offset_y, &mut aliens);
            direction = direction.saturating_add_signed(direction_index);
            if direction == 8 || direction == 0 {
                direction_index = -direction_index;
                offset_y += 10;
            }
            enemy_last_movement = time_manager.now();
        }
        hero.handle_movement(hero_movement_direction, delta_ms as u64, fb.width() as u32);
        hero_last_movement = time_manager.now();

        // 3. collision detection
        let mut new_shoots = Vec::new();
        for shoot in shoots {
            match shoot.owner {
                ShootOwner::Enemy => {
                    if shoot.is_hit(&hero.structure.coordinates) {
                        info!("Hero is dead!");
                    } else {
                        new_shoots.push(shoot);
                    }
                }
                ShootOwner::Hero => {
                    let mut has_hit = false;
                    for alien in aliens.iter_mut().filter(|a| a.structure.alive) {
                        if shoot.is_hit(&alien.structure.coordinates) {
                            alien.structure.alive = false;
                            info!("Alien is dead!");
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
            info!("Game over!");
            return;
        }

        let mut alive = false;
        for enemy in aliens.iter() {
            alive = alive || enemy.structure.alive;
            if enemy.structure.coordinates.y + enemy.structure.height
                >= hero.structure.coordinates.y
            {
                info!("Game over!");
                return;
            }
        }
        if !alive {
            info!("Game over, you won!");
            return;
        }

        // 4. draw things:
        for enemy in aliens.iter() {
            if enemy.structure.alive {
                enemy.draw(fb)
            }
        }

        hero.draw(fb);
        for shoot in shoots.iter() {
            shoot.draw(fb);
        }
        const FPS: u128 = 60;
        if time_manager.since(last_draw).as_millis() >= 1000 / FPS {
            fb.update();
            last_draw = time_manager.now();
        }
        last_loop = time_manager.since(loop_start);
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

#[derive(Clone, Copy, Debug)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
}
