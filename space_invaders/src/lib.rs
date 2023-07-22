#![feature(let_chains)]
#![feature(return_position_impl_trait_in_trait)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "no_std", feature(format_args_nl))]
#![warn(clippy::pedantic)]

extern crate core;

pub mod actor;
mod framebuffer;

mod game_context;
mod platform;
mod time;

use core::alloc;
use log::info;

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

use crate::actor::{Shoot, ShootOwner, HERO_WIDTH};
use crate::framebuffer::fb_trait::FrameBufferInterface;
use crate::framebuffer::Coordinates;
use crate::game_context::HeroMovementDirection;
#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub const SCREEN_WIDTH: usize = 1280;
pub const SCREEN_WIDTH_NO_MARGIN: usize = SCREEN_WIDTH - SCREEN_MARGIN;
pub const SCREEN_HEIGHT: usize = 720;
pub const SCREEN_MARGIN: usize = 20;
// todo: in STD, if FPS is very low (i.e. no sleep at the end of the loop) enemies are stopped
// because the speedup rounds to 0.
const FPS: u128 = 15;

pub enum EndOfGame {
    Restarted,
    Won(u32),
    Lost(u32),
}
impl EndOfGame {
    fn to_score(&self) -> u32 {
        use EndOfGame::{Lost, Restarted, Won};
        match self {
            Won(x) | Lost(x) => *x,
            Restarted => 0,
        }
    }
}
pub trait MemoryAllocator {
    fn alloc(&self, layout: alloc::Layout) -> *mut u8;
}
pub trait UserInput {
    fn get_input(&self) -> impl Iterator<Item = KeyPressedKeys>;

    // get input from keyboard
    fn get_input_keys(
        &self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>) {
        let mut hero_movement_direction = HeroMovementDirection::Still;
        let mut shoot = None;
        for key in &mut self.get_input() {
            match key {
                KeyPressedKeys::Left => {
                    hero_movement_direction = HeroMovementDirection::Left;
                }
                KeyPressedKeys::Right => {
                    hero_movement_direction = HeroMovementDirection::Right;
                }
                KeyPressedKeys::Shoot => {
                    let new_shoot = Shoot::new(
                        Coordinates::new(
                            hero_coordinates.x() + HERO_WIDTH / 2,
                            hero_coordinates.y() - 10,
                        ),
                        ShootOwner::Hero,
                    );
                    info!("pew!");
                    shoot = Some(new_shoot);
                }
                _ => {
                    hero_movement_direction = HeroMovementDirection::Still;
                }
            }
        }
        (hero_movement_direction, shoot)
    }
}

pub enum KeyPressedKeys {
    Left,
    Right,
    Shoot,
    Pause,
}

pub fn run_game<F>(mut fb: F, time_manager: impl TimeManagerInterface)
where
    F: FrameBufferInterface + MemoryAllocator + UserInput,
{
    let mut high_score = 0;
    let mut current_score: u32 = 0;
    loop {
        info!("Starting game...");
        let mut game_context =
            game_context::GameContext::new(&mut fb, high_score, current_score, &time_manager);
        let result = game_context.play();
        current_score += result.to_score();
        if current_score > high_score {
            high_score = current_score;
        }
        if matches!(result, EndOfGame::Lost(_)) {
            current_score = 0;
        }
    }
}
