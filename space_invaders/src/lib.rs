#![feature(let_chains)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]
#![warn(clippy::pedantic)]

extern crate core;

pub mod actor;
mod framebuffer;

mod game_context;
mod platform;
mod time;

use crate::actor::{
    create_shoots, shoots_handle_movement, Barricade, Enemies, EnemiesDirection, Enemy, Hero,
    ShootOwner, ENEMY_COLS, SHOOT_ENEMY_MAX, SHOOT_HERO_MAX, SHOOT_MAX_ALLOC, TOTAL_ENEMIES,
};
pub use crate::actor::{Actor, Shoot};
pub use crate::framebuffer::fb_trait::FrameBufferInterface;
use core::cmp;
use core::ops::Sub;
use core::time::Duration;
pub use framebuffer::{Coordinates, Pixel};

use log::info;

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

use crate::framebuffer::fb_trait::{UI_SCORE_COLOR, UI_SCORE_COORDINATES};
use crate::EndOfGame::{Lost, Restarted, Won};
#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_WIDTH_NO_MARGIN: usize = SCREEN_WIDTH - SCREEN_MARGIN;
pub const SCREEN_HEIGHT: usize = 720;
pub const SCREEN_MARGIN: usize = 20;
// todo: in STD, if FPS is very low (i.e. no sleep at the end of the loop) enemies are stopped
// because the speedup rounds to 0.
const FPS: u128 = 15;

enum EndOfGame {
    Restarted,
    Won(u32),
    Lost(u32),
}
impl EndOfGame {
    fn to_score(&self) -> u32 {
        use EndOfGame::*;
        match self {
            Won(x) | Lost(x) => *x,
            Restarted => 0,
        }
    }
}

pub fn run_game(mut fb: impl FrameBufferInterface, time_manager: impl TimeManagerInterface) {
    let mut high_score = 0;
    let mut current_score: u32 = 0;
    loop {
        info!("Starting game...");

        let result = init_game(&mut fb, &time_manager, high_score, current_score);
        current_score += result.to_score();
        if current_score > high_score {
            high_score = current_score;
        }
        if matches!(result, EndOfGame::Lost(_)) {
            current_score = 0;
        }
    }
}

fn init_game(
    fb: &mut impl FrameBufferInterface,
    time_manager: &impl TimeManagerInterface,
    high_score: u32,
    current_score: u32,
) -> EndOfGame {
    let mut enemies2 = Enemies::new(fb);
    // todo, instead of using option just set alive: false,
    let mut shoots: [Option<Shoot>; SHOOT_MAX_ALLOC] = [None; SHOOT_MAX_ALLOC];
    let mut hero_shoots = 0;

    let mut hero = Hero::new(fb);

    let mut last_loop = time_manager.now();
    // free random :D
    let mut random = [
        35, 13, 65, 16, 15, 23, 84, 79, 65, 85, 99, 8, 63, 74, 57, 75, 9, 92, 25, 29,
    ];
    let mut random_index = 0;

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
            return Restarted;
        }

        // 2. Handle shoots. Create if hero's or enemies' as needed.
        create_shoots(shoot, &mut hero_shoots, rnd, &mut shoots, &mut enemies2);

        // 2. Movement
        handle_movements(
            fb,
            &mut shoots,
            &mut hero_shoots,
            &mut hero,
            hero_movement_direction,
            delta_ms,
            &mut enemies2,
        );

        // 3. collision detection
        // this is not the best way to do it, but it works.
        // The issue here is that if the loop runs really slowly, then the shoot will overlap
        // with the enemies in very few positions. OFC, if the game is running with so few fps,
        // it would be unplayable anyway.

        for sh in &mut shoots {
            if let Some(shoot) = sh {
                match shoot.owner {
                    ShootOwner::Enemy => {
                        if shoot.is_hit(hero.get_structure()) {
                            let _ = sh.take();
                            hero.structure.alive = false;
                            continue;
                        }
                        for b in barricades.iter_mut().filter(|ba| ba.structure.alive) {
                            if shoot.is_hit(b.get_structure()) {
                                let _ = sh.take();
                                b.structure.alive = false;
                                barricades_alive -= 1;
                                enemies2.enemy_shoots -= 1;
                                break;
                            }
                        }
                    }
                    ShootOwner::Hero => {
                        for (actor, is_enemy) in enemies2
                            .enemies
                            .iter_mut()
                            .map(|e| (&mut e.structure, 1))
                            .chain(barricades.iter_mut().map(|e| (&mut e.structure, 0)))
                            .filter(|a| a.0.alive)
                        {
                            if shoot.is_hit(actor) {
                                actor.alive = false;
                                enemies2.enemies_dead += is_enemy;
                                barricades_alive -= usize::from(is_enemy == 0);
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
        let ret = check_game_over(&hero, &enemies2, &mut barricades, barricades_alive);
        if ret.is_some() {
            return ret.unwrap();
        }

        // 4. draw things:
        draw(fb, &hero, &enemies2, &shoots, &barricades);

        let current_score_updated = current_score + enemies2.enemies_dead as u32;
        let high_score_updated = cmp::max(current_score_updated, high_score);
        let message =
            format!("High Score: {high_score_updated} - Current Score: {current_score_updated}");
        fb.write_ui(UI_SCORE_COORDINATES, &message, UI_SCORE_COLOR);
        fb.update();

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

fn handle_movements(
    fb: &mut impl FrameBufferInterface,
    shoots: &mut [Option<Shoot>],
    hero_shoots: &mut usize,
    hero: &mut Hero,
    hero_movement_direction: HeroMovementDirection,
    delta_ms: u64,
    enemies2: &mut Enemies,
) {
    shoots_handle_movement(
        fb,
        shoots,
        &mut enemies2.enemy_shoots,
        hero_shoots,
        delta_ms,
    );

    enemies2.move_enemies(delta_ms);

    hero.handle_movement(hero_movement_direction, delta_ms);
}

#[derive(Clone, Copy, Debug)]
pub enum HeroMovementDirection {
    Left,
    Right,
    Still,
    RestartGame,
}

fn draw(
    fb: &mut impl FrameBufferInterface,
    hero: &Hero,
    enemies2: &Enemies,
    shoots: &[Option<Shoot>],
    barricades: &[Barricade],
) {
    fb.clear_screen();

    for enemy in enemies2.enemies.iter() {
        enemy.draw(fb);
    }

    hero.draw(fb);
    for shoot in shoots.iter().flatten() {
        shoot.draw(fb);
    }

    for b in barricades.iter() {
        b.draw(fb);
    }
}

/// It also check collision of aliens against barricades.
fn check_game_over(
    hero: &Hero,
    enemies2: &Enemies,
    barricades: &mut [Barricade],
    barricades_alive: usize,
) -> Option<EndOfGame> {
    if !hero.structure.alive {
        info!("Game over, you lost! Hero is dead");
        return Some(Lost(enemies2.enemies_dead as u32));
    }

    let all_aliens_dead = TOTAL_ENEMIES - enemies2.enemies_dead == 0;
    if all_aliens_dead {
        info!("Game over, you won! All enemies dead.",);
        return Some(Won(enemies2.enemies_dead as u32));
    }

    for enemy in enemies2.enemies.iter() {
        if !enemy.structure.alive {
            continue;
        }
        let reached_hero = enemy.structure.coordinates.y() + enemy.structure.height
            >= hero.structure.coordinates.y();
        if reached_hero {
            info!("Game over, you lost! Enemy has reached the hero");
            return Some(Lost(enemies2.enemies_dead as u32));
        }
        let reached_barricades = enemy.structure.coordinates.y() + enemy.structure.height
            >= barricades[0].structure.coordinates.y();
        if reached_barricades && barricades_alive > 0 {
            for b in barricades.iter_mut() {
                b.structure.alive = false;
            }
        }
    }
    None
}
