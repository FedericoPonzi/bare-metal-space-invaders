#![feature(let_chains)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]
#![warn(clippy::pedantic)]

pub mod actor;
mod framebuffer;

mod time;

pub use crate::actor::{init_enemies, move_enemies, Actor, Shoot};
use crate::actor::{
    Barricade, EnemiesDirection, Hero, ShootOwner, ENEMY_COLS, SHOOT_ENEMY_MAX, SHOOT_HERO_MAX,
    SHOOT_MAX_ALLOC, TOTAL_ENEMIES,
};
pub use crate::framebuffer::fb_trait::FrameBufferInterface;
use core::ops::Sub;
pub use framebuffer::{Coordinates, Pixel};
use std::time::Duration;

use log::info;
use noto_sans_mono_bitmap::{get_raster_width, FontWeight, RasterHeight};

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

use crate::framebuffer::color;
use crate::framebuffer::fb_trait::{UI_MAX_SCORE_LEN, UI_SCORE_COLOR, UI_SCORE_COORDINATES};
#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_WIDTH_NO_MARGIN: usize = SCREEN_WIDTH - SCREEN_MARGIN;
pub const SCREEN_HEIGHT: usize = 720;
pub const SCREEN_MARGIN: usize = 20;
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
    let mut enemies = init_enemies(fb);

    // todo, instead of using option just set alive: false,
    let mut shoots: [Option<Shoot>; SHOOT_MAX_ALLOC] = [None; SHOOT_MAX_ALLOC];
    let mut hero_shoots = 0;
    let mut enemy_shoots = 0;

    let mut hero = Hero::new(fb);

    let mut direction = EnemiesDirection::Right;
    let mut last_loop = time_manager.now();
    // used for speedup calculation.
    let mut enemies_dead = 0;
    let mut lowest_col = (ENEMY_COLS, 0);
    let mut largest_col = (0, 0);
    let mut random = [0; 10];
    let mut random_index = 0;
    for i in 0..random.len() {
        random[i] = fb.random();
    }

    let mut barricades = Barricade::create_barricades();
    let mut barricades_alive = barricades.len();

    loop {
        let now = time_manager.now();
        let delta_ms = now.sub(last_loop).as_millis() as u64;
        last_loop = now;
        if random_index == random.len() {
            random_index = 0;
        }
        let rnd = random[random_index];
        random_index += 1;

        info!("delta_ms: {}", delta_ms);

        // 1. Get input
        let (hero_movement_direction, shoot) = fb.get_input_keys(&hero.structure.coordinates);

        if matches!(hero_movement_direction, HeroMovementDirection::RestartGame) {
            info!("Restarting game...");
            return;
        }
        if hero_shoots < SHOOT_HERO_MAX && let Some(shoot) = shoot {
            for sh in shoots.iter_mut() {
                if sh.is_none() {
                    sh.replace(shoot);
                    hero_shoots += 1;
                    break;
                }
            }
        }

        if enemy_shoots < SHOOT_ENEMY_MAX {
            let enemy_shooting = rnd as usize % (TOTAL_ENEMIES - enemies_dead);
            for (id, enemy) in enemies.iter().filter(|e| e.structure.alive).enumerate() {
                if enemy_shooting == id {
                    for sh in shoots.iter_mut() {
                        if sh.is_none() {
                            sh.replace(Shoot::from(enemy));
                            enemy_shoots += 1;
                            break;
                        }
                    }
                }
            }
        }

        // 2. Movement
        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh.as_mut() {
                shoot.move_forward(delta_ms);
                if shoot.out_of_screen(fb.height() as u32) {
                    info!("shoot is out of screen!");
                    if shoot.owner == ShootOwner::Hero {
                        hero_shoots -= 1;
                    } else {
                        enemy_shoots -= 1;
                    }
                    //remove it.
                    let _ = sh.take();
                }
            }
        }

        direction = move_enemies(
            &mut enemies,
            direction,
            delta_ms,
            &mut lowest_col,
            &mut largest_col,
            enemies_dead,
        );

        hero.handle_movement(hero_movement_direction, delta_ms);

        // 3. collision detection
        // this is not the best way to do it, but it works.
        // The issue here is that if the loop runs really slowly, then the shoot will overlap
        // with the enemies in very few positions. OFC, if the game is running with so few fps,
        // it would be unplayable anyway.

        for sh in shoots.iter_mut() {
            if let Some(shoot) = sh {
                match shoot.owner {
                    ShootOwner::Enemy => {
                        if shoot.is_hit(hero.get_structure()) {
                            let _ = sh.take();
                            info!("Hero is dead!");
                            hero.structure.alive = false;
                            continue;
                        }
                        for b in barricades.iter_mut().filter(|ba| ba.structure.alive) {
                            if shoot.is_hit(b.get_structure()) {
                                let _ = sh.take();
                                info!("barricade hit!");
                                b.structure.alive = false;
                                barricades_alive -= 1;
                                enemy_shoots -= 1;
                                break;
                            }
                        }
                    }
                    ShootOwner::Hero => {
                        for (actor, is_enemy) in enemies
                            .iter_mut()
                            .map(|e| (&mut e.structure, 1))
                            .chain(barricades.iter_mut().map(|e| (&mut e.structure, 0)))
                            .filter(|a| a.0.alive)
                        {
                            if shoot.is_hit(actor) {
                                actor.alive = false;
                                info!("Alien is dead!");
                                enemies_dead += is_enemy;
                                barricades_alive -= if is_enemy == 0 { 1 } else { 0 };
                                sh.take();
                                hero_shoots -= 1;
                                break;
                            }
                        }
                    }
                }
            }
        }

        // check if game is over.
        if !hero.structure.alive {
            info!("Game over, you lost! Hero is dead");
            return;
        }

        let all_aliens_dead = TOTAL_ENEMIES - enemies_dead == 0;
        if all_aliens_dead {
            info!("Game over, you won! All enemies dead.",);
            return;
        }

        for enemy in enemies.iter() {
            if !enemy.structure.alive {
                continue;
            }
            let reached_hero = enemy.structure.coordinates.y() + enemy.structure.height
                >= hero.structure.coordinates.y();
            if reached_hero {
                info!("Game over, you lost! Enemy has reached the hero");
                return;
            }
            let reached_barricades = enemy.structure.coordinates.y() + enemy.structure.height
                >= barricades[0].structure.coordinates.y();
            if reached_barricades && barricades_alive > 0 {
                for b in barricades.iter_mut() {
                    b.structure.alive = false;
                }
            }
        }

        // 4. draw things:
        fb.clear_screen();

        for enemy in enemies.iter() {
            enemy.draw(fb)
        }

        hero.draw(fb);
        for shoot in shoots.iter_mut().flatten() {
            shoot.draw(fb);
        }

        for b in barricades.iter() {
            b.draw(fb);
        }

        let message = format!("High Score: 9999 - Current Score: 9999");
        fb.write_ui(UI_SCORE_COORDINATES, &message, UI_SCORE_COLOR);
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
