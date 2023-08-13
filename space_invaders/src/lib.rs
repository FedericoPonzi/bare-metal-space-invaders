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

use log::info;

#[cfg(feature = "std")]
pub use crate::time::TimeManager;

pub use crate::time::TimeManagerInterface;

use crate::actor::{
    HeroMovementDirection, Shoot, ShootOwner, SHOOT_OFFSET_X_HERO, SHOOT_OFFSET_Y_HERO,
};

pub use crate::framebuffer::fb_trait::FrameBufferInterface;
pub use crate::framebuffer::{Color, Coordinates};

#[cfg(feature = "std")]
pub use framebuffer::StdFrameBuffer;

pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_WIDTH_NO_MARGIN: u32 = SCREEN_WIDTH - SCREEN_MARGIN;
pub const SCREEN_HEIGHT: u32 = 720;
pub const SCREEN_HEIGHT_NO_MARGIN: u32 = SCREEN_HEIGHT - SCREEN_MARGIN;
pub const SCREEN_MARGIN: u32 = 20;

pub(crate) const MAX_LIVES: u8 = 3;

// todo: in STD, if FPS is very low (i.e. no sleep at the end of the loop) enemies are stopped
// because the speedup rounds to 0.
const FPS: u128 = 30;

pub enum EndOfGame {
    Restarted,
    Won(usize),
    Lost(usize),
}
impl EndOfGame {
    fn to_score(&self) -> u32 {
        use EndOfGame::{Lost, Restarted, Won};
        match self {
            Won(x) | Lost(x) => u32::try_from(*x).expect("Conversion failed"),
            Restarted => 0,
        }
    }
}
pub trait UserInput {
    fn get_input(&mut self) -> impl Iterator<Item = KeyPressedKeys>;

    // get input from keyboard
    fn get_input_keys(
        &mut self,
        hero_coordinates: &Coordinates,
    ) -> (HeroMovementDirection, Option<Shoot>) {
        let mut hero_movement_direction = HeroMovementDirection::Still;
        let mut shoot = None;
        let mut restart = None;
        for key in self.get_input() {
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
                            hero_coordinates.x() + SHOOT_OFFSET_X_HERO,
                            hero_coordinates.y() - SHOOT_OFFSET_Y_HERO,
                        ),
                        ShootOwner::Hero,
                    );
                    //info!("pew!");
                    shoot = Some(new_shoot);
                }
                KeyPressedKeys::Restart => {
                    //info!("pressed restart");
                    restart = Some((HeroMovementDirection::RestartGame, None));
                }
            }
        }
        if let Some(restart) = restart {
            return restart;
        }
        (hero_movement_direction, shoot)
    }
}

pub enum KeyPressedKeys {
    Left,
    Right,
    Shoot,
    Restart,
}

pub fn run_game<F>(mut fb: F, time_manager: &impl TimeManagerInterface)
where
    F: FrameBufferInterface + UserInput,
{
    let mut high_score = 0;
    let mut current_score: u32 = 0;
    loop {
        info!("Starting game...");
        let mut game_context = game_context::GameContext::new(
            &mut fb,
            high_score,
            current_score,
            time_manager,
            MAX_LIVES,
        );
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

#[macro_use]
mod macros {
    #[repr(C)] // guarantee 'bytes' comes after '_align'
    pub struct AlignedAs<Align, Bytes: ?Sized> {
        pub _align: [Align; 0],
        pub bytes: Bytes,
    }
    #[macro_export]
    macro_rules! include_bytes_align_as {
        ($align_ty:ty, $path:literal) => {{
            // const block expression to encapsulate the static
            use $crate::macros::AlignedAs;

            // this assignment is made possible by CoerceUnsized
            static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
                _align: [],
                bytes: *include_bytes!($path),
            };

            let as_u8 = &ALIGNED.bytes;
            // safety: the alignment is guaranteed by the above const block expression
            unsafe { core::slice::from_raw_parts(as_u8.as_ptr().cast::<u32>(), as_u8.len() / 4) }
        }};
    }
}
