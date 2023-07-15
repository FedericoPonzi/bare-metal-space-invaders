#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]

pub mod actor;
mod framebuffer;

mod time;

pub use crate::actor::{init_enemies, move_enemies, Actor, Shoot};
use crate::actor::{
    EnemiesDirection, Hero, ShootOwner, ALIEN_COLS, SHOOT_MAX_ALLOC, TOTAL_ENEMIES,
};
pub use crate::framebuffer::fb_trait::FrameBufferInterface;
use core::ops::Sub;
pub use framebuffer::{Coordinates, Pixel};
use std::time::Duration;

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
// todo: in STD, if FPS is very low (i.e. no sleep at the end of the loop) enemies are stopped
// because the speedup rounds to 0.
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
    loop {
        let now = time_manager.now();
        let delta_ms = now.sub(last_loop).as_millis() as u64;
        last_loop = now;
        info!("delta_ms: {}", delta_ms);

        // 1. Get input
        let (hero_movement_direction, shoot) = fb.get_input_keys(&hero.structure.coordinates, fb);
        info!(
            "hero_movement_direction: {:?}, shoot: {:?}",
            hero_movement_direction, shoot
        );
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
        info!("Shoots: {:?}", shoots);
        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh.as_mut() {
                info!("shoot: {:?}", shoot);
                shoot.move_forward(delta_ms);
                if shoot.out_of_screen() {
                    info!("shoot is out of screen!");
                    //remove it.
                    let _ = sh.take();
                }
            }
        }

        direction = move_enemies(
            &mut offset_y,
            &mut aliens,
            direction,
            delta_ms,
            &mut lowest_col,
            &mut largest_col,
            enemies_dead,
        );

        hero.handle_movement(hero_movement_direction, delta_ms);

        // 3. collision detection

        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh {
                match shoot.owner {
                    ShootOwner::Enemy => {
                        if shoot.is_hit(&hero) {
                            let _ = sh.take();
                            info!("Hero is dead!");
                            hero.structure.alive = false;
                        }
                    }
                    ShootOwner::Hero => {
                        let mut has_hit = false;
                        for alien in aliens.iter_mut().filter(|a| a.structure.alive) {
                            if shoot.is_hit(alien) {
                                alien.structure.alive = false;
                                info!("Alien is dead!");
                                has_hit = true;
                                enemies_dead += 1;
                                break;
                            }
                        }
                        if has_hit {
                            info!("shoot has hit an enemy!");
                            sh.take();
                        }
                    }
                }
            }
        }

        // check if game is over.
        if !hero.structure.alive {
            info!("Game over, you lost!");
            return;
        }

        let all_aliens_dead = TOTAL_ENEMIES - enemies_dead == 0;
        if all_aliens_dead {
            info!("Game over, you won!");
            return;
        }

        for enemy in aliens.iter() {
            if enemy.structure.alive
                && enemy.structure.coordinates.y() + enemy.structure.height
                    >= hero.structure.coordinates.y()
            {
                info!("Game over, you lost!");
                return;
            }
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

        info!("Updating fb...");
        fb.update();
        info!("Loop completed");

        #[cfg(feature = "std")]
        let delta_next =
            Duration::from_millis(1000 / FPS as u64).saturating_sub(time_manager.since(last_loop));
        #[cfg(feature = "std")]
        if delta_next.as_millis() > 0 {
            #[cfg(feature = "std")]
            std::thread::sleep(delta_next);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
    RestartGame,
}
