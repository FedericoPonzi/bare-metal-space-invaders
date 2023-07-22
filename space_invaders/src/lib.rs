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

use crate::actor::{Barricade, Enemies, Hero, Shoots, TOTAL_ENEMIES};
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
    let mut enemies = Enemies::new(fb);
    let mut shoots = Shoots::new();
    let mut hero = Hero::new(fb);

    let mut barricades = Barricade::create_barricades();
    let mut barricades_alive = barricades.len();

    let mut last_loop = time_manager.now();
    // super fast random :D
    let random = [
        35, 13, 65, 16, 15, 23, 84, 79, 65, 85, 99, 8, 63, 74, 57, 75, 9, 92, 25, 29,
    ];
    let mut random_index = 0;

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
        shoots.create_shoots(shoot, rnd, &mut enemies);

        // 2. Movement
        handle_movements(
            &mut shoots,
            &mut hero,
            hero_movement_direction,
            delta_ms,
            &mut enemies,
        );

        // 3. collision detection
        shoots.check_collisions(
            &mut hero,
            &mut enemies,
            &mut barricades,
            &mut barricades_alive,
        );

        // check if game is over.
        if let Some(ret) = check_game_over(&hero, &enemies, &mut barricades, barricades_alive) {
            return ret;
        }

        // Draw things:
        draw(fb, &hero, &enemies, &shoots, &barricades);

        let current_score_updated = current_score + enemies.enemies_dead as u32;
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
    shoots: &mut Shoots,
    hero: &mut Hero,
    hero_movement_direction: HeroMovementDirection,
    delta_ms: u64,
    enemies: &mut Enemies,
) {
    shoots.handle_movement(delta_ms);
    enemies.move_enemies(delta_ms);
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
    enemies: &Enemies,
    shoots: &Shoots,
    barricades: &[Barricade],
) {
    fb.clear_screen();
    enemies.draw(fb);
    hero.draw(fb);
    shoots.draw(fb);
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
