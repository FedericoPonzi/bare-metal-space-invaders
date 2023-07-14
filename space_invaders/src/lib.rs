#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]

pub mod actor;
mod framebuffer;

mod time;

pub use crate::actor::{init_enemies, move_enemies, Actor, Shoot};
use crate::actor::{EnemiesDirection, Hero, ShootOwner, ALIEN_COLS, SHOOT_MAX_ALLOC};
pub use crate::framebuffer::fb_trait::FrameBufferInterface;
use core::time::Duration;
pub use framebuffer::{Coordinates, Pixel};

use log::info;

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_WIDTH_NO_MARGIN: usize = SCREEN_WIDTH - SCREEN_MARGIN;
pub const SCREEN_HEIGHT: usize = 720;
pub const SCREEN_MARGIN: usize = 30;
const FPS: u128 = 15;

pub fn run_game(mut fb: impl FrameBufferInterface, time_manager: impl TimeManagerInterface) {
    loop {
        info!("Starting game...");
        init_game(&mut fb, &time_manager);
    }
}

fn init_game(fb: &mut impl FrameBufferInterface, time_manager: &impl TimeManagerInterface) {
    let mut aliens = init_enemies(fb);

    let mut offset_y = 0;
    let mut shoots: [Option<Shoot>; SHOOT_MAX_ALLOC] = [None; SHOOT_MAX_ALLOC];
    let mut hero = Hero::new(fb);

    let mut direction = EnemiesDirection::Right;
    let mut last_loop = time_manager.now();
    // used for speedup calculation.
    let mut enemies_dead = 0;
    let mut lowest_col = (ALIEN_COLS, 0);
    let mut largest_col = (0, 0);
    use core::ops::Sub;
    loop {
        let now = time_manager.now();
        let delta_ms = now.sub(last_loop).as_millis();
        last_loop = now;
        info!("delta_ms: {}", delta_ms);

        // 1. Get input
        let (hero_movement_direction, shoot) = fb.get_input_keys(&hero.structure.coordinates, fb);
        if matches!(hero_movement_direction, HeroMovementDirection::RestartGame) {
            info!("Restarting game...");
            return;
        }
        if let Some(shoot) = shoot {
            for sh in shoots.iter_mut() {
                if sh.is_none() {
                    sh.replace(shoot);
                    break;
                }
            }
        }

        // 2. Movement
        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh.as_mut() {
                shoot.move_forward(delta_ms as u64);
                if out_of_screen(shoot) {
                    //remove it.
                    let _ = sh.take();
                }
            }
        }

        direction = move_enemies(
            &mut offset_y,
            &mut aliens,
            direction,
            delta_ms as u64,
            &mut lowest_col,
            &mut largest_col,
            enemies_dead,
        );

        hero.handle_movement(hero_movement_direction, delta_ms as u64);

        // 3. collision detection

        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh {
                match shoot.owner {
                    ShootOwner::Enemy => {
                        if shoot.is_hit(&hero.structure.coordinates) {
                            let _ = sh.take();
                            info!("Hero is dead!");
                            hero.structure.alive = false;
                        }
                    }
                    ShootOwner::Hero => {
                        let mut has_hit = false;
                        for alien in aliens.iter_mut().filter(|a| a.structure.alive) {
                            if shoot.is_hit(&alien.structure.coordinates) {
                                alien.structure.alive = false;
                                info!("Alien is dead!");
                                has_hit = true;
                                enemies_dead += 1;
                                break;
                            }
                        }
                        if has_hit {
                            sh.take();
                        }
                    }
                }
            }
        }
        //info!("Collision detection is over");

        if !hero.structure.alive {
            info!("Game over!");
            return;
        }

        let mut alive = false;

        for enemy in aliens.iter() {
            alive = alive || enemy.structure.alive;
            if enemy.structure.coordinates.y() + enemy.structure.height
                >= hero.structure.coordinates.y()
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
        fb.clear_screen();

        for enemy in aliens.iter() {
            if enemy.structure.alive {
                enemy.draw(fb)
            }
        }
        hero.draw(fb);
        for shoot in shoots.iter_mut().flatten() {
            shoot.draw(fb);
        }

        //info!("Updating fb...");
        fb.update();
        //info!("Done updating fb");
    }
}

#[inline(always)]
fn out_of_screen(shoot: &Shoot) -> bool {
    let structure = shoot.structure;
    let coordinates = structure.coordinates;
    coordinates.x() > (structure.width * structure.height)
        || coordinates.y() == 0
        || coordinates.y() > (structure.width * structure.height)
}

#[derive(Clone, Copy, Debug)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
    RestartGame,
}
