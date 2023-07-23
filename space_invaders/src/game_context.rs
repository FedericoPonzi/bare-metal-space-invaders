use crate::actor::{
    Actor, Barricade, Enemies, Hero, Shoots, HERO_ALIGNED, HERO_WIDTH, TOTAL_ENEMIES,
};
use crate::framebuffer::fb_trait::{
    FrameBufferInterface, UI_MAX_SCORE_LEN, UI_SCORE_COLOR, UI_SCORE_COORDINATES,
};
use crate::framebuffer::Coordinates;
use crate::EndOfGame::{Lost, Restarted, Won};
#[cfg(feature = "std")]
use crate::FPS;
use crate::{EndOfGame, MemoryAllocator, TimeManagerInterface, UserInput, SCREEN_MARGIN};
use core::cmp;
use core::mem;
use core::ops::Sub;
use core::time::Duration;
use log::info;

pub struct GameContext<'a, T, F>
where
    F: FrameBufferInterface + MemoryAllocator + UserInput,
    T: TimeManagerInterface,
    // F: FrameBufferInterface,
    // A: MemoryAllocator,
    // U: UserInput,
{
    pub hero: Hero,
    pub high_score: u32,
    pub current_score: u32,
    pub time_manager: &'a T,
    fb: &'a mut F,
    //allocator: A,
    //user_input: U,
    shoots: Shoots,
    barricades: [Barricade; 56],
    barricades_alive: usize,
    last_loop: Duration,
    enemies: Enemies,
    random: [u32; 20],
    random_index: usize,
    current_lifes: u8,
}

impl<'a, T, F> GameContext<'a, T, F>
where
    F: FrameBufferInterface + MemoryAllocator + UserInput,
    T: TimeManagerInterface,
{
    pub fn new(
        fb: &'a mut F,
        high_score: u32,
        current_score: u32,
        time_manager: &'a T,
        current_lifes: u8,
    ) -> Self {
        let enemies = Enemies::new(fb);
        let shoots = Shoots::new();
        let hero = Hero::new(fb);

        let barricades = Barricade::create_barricades();
        let barricades_alive = barricades.len();

        let last_loop = time_manager.now();

        // super fast random :D
        let random = [
            35, 13, 65, 16, 15, 23, 84, 79, 65, 85, 99, 8, 63, 74, 57, 75, 9, 92, 25, 29,
        ];
        let random_index = 0;

        Self {
            hero,
            high_score,
            current_score,
            time_manager,
            fb,
            shoots,
            barricades,
            barricades_alive,
            last_loop,
            enemies,
            random,
            random_index,
            current_lifes,
        }
    }

    pub fn play(&mut self) -> EndOfGame {
        loop {
            let now = self.time_manager.now();
            let delta_ms =
                u64::try_from(now.sub(self.last_loop).as_millis()).expect("Conversion failed");
            self.last_loop = now;
            if self.random_index == self.random.len() {
                self.random_index = 0;
            }
            let rnd = self.random[self.random_index];
            self.random_index += 1;

            info!("delta_ms: {}", delta_ms);

            // 1. Get input
            let (hero_movement_direction, shoot) =
                self.fb.get_input_keys(&self.hero.structure.coordinates);

            if matches!(hero_movement_direction, HeroMovementDirection::RestartGame) {
                info!("Restarting game...");
                return Restarted;
            }

            //info!("Creating shoots");
            // 2. Handle shoots. Create if hero's or enemies' as needed.
            self.shoots.create_shoots(shoot, rnd, &mut self.enemies);
            //info!("handling movement");
            // 2. Movement
            handle_movements(
                &mut self.shoots,
                &mut self.hero,
                hero_movement_direction,
                delta_ms,
                &mut self.enemies,
            );
            //info!("Collision detection");
            // 3. collision detection
            self.shoots.check_collisions(
                &mut self.hero,
                &mut self.enemies,
                &mut self.barricades,
                &mut self.barricades_alive,
            );
            //info!("Checking if it's game over");
            // check if game is over.
            if let Some(ret) = check_game_over(
                &mut self.hero,
                &self.enemies,
                &mut self.barricades,
                self.barricades_alive,
                &mut self.current_lifes,
            ) {
                return ret;
            }
            //info!("Drawing things");
            // Draw things:
            draw(
                self.fb,
                &self.hero,
                &self.enemies,
                &self.shoots,
                &self.barricades,
            );
            //info!("Updating score and writing ui");
            let current_score_updated = self.current_score
                + u32::try_from(self.enemies.enemies_dead).expect("Conversion failed");
            let high_score_updated = cmp::max(current_score_updated, self.high_score);
            let mut write_ui = |m: &str| self.fb.write_ui(UI_SCORE_COORDINATES, m, UI_SCORE_COLOR);
            let mut message_buf = [0u8; UI_MAX_SCORE_LEN * mem::size_of::<char>()];
            let score_ui =
                format_to_buffer(&mut message_buf, high_score_updated, current_score_updated)
                    .expect("TODO: panic message");
            //info!("writing ui...");
            write_ui(score_ui);
            self.draw_lifes();
            //info!("Done updating ui");
            self.fb.update();

            #[cfg(feature = "std")]
            let delta_next = Duration::from_millis(1000 / FPS as u64)
                .saturating_sub(self.time_manager.since(self.last_loop));
            #[cfg(feature = "std")]
            if delta_next.as_millis() > 0 {
                #[cfg(feature = "std")]
                std::thread::sleep(delta_next);
            }
        }
    }
    fn draw_lifes(&mut self) {
        const UI_LIFES_X: u32 = SCREEN_MARGIN as u32 / 2;
        const UI_LIFES_Y: u32 = SCREEN_MARGIN as u32 / 2;
        const UI_LIFES_X_OFFSET_BETWEEN_LIFES: u32 = 20;
        for i in 0..self.current_lifes {
            unsafe {
                self.fb.display_image(
                    Coordinates::new(
                        UI_LIFES_X + i as u32 * (HERO_WIDTH + UI_LIFES_X_OFFSET_BETWEEN_LIFES),
                        UI_LIFES_Y,
                    ),
                    HERO_ALIGNED.unwrap(),
                    HERO_WIDTH,
                );
            }
        }
    }
}

// Function to write formatted data into a buffer
fn format_to_buffer(
    buffer: &mut [u8],
    high_score: u32,
    current_score: u32,
) -> Result<&str, core::fmt::Error> {
    use core::fmt::Write;
    let mut output = BufferWrite::new(buffer);
    write!(
        output,
        "High Score: {high_score} - Current Score: {current_score}"
    )?;

    // Convert the buffer slice into a &str
    let written_length = output.written_length();
    let formatted_str = core::str::from_utf8(&buffer[..written_length]).unwrap();
    Ok(formatted_str)
}

// A custom implementation of core::fmt::Write for writing into a buffer
struct BufferWrite<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> BufferWrite<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        BufferWrite {
            buffer,
            position: 0,
        }
    }

    // Get the total number of bytes written so far
    fn written_length(&self) -> usize {
        self.position
    }
}

// Implement the Write trait for BufferWrite
impl<'a> core::fmt::Write for BufferWrite<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining_space = self.buffer.len() - self.position;

        if bytes.len() <= remaining_space {
            self.buffer[self.position..self.position + bytes.len()].copy_from_slice(bytes);
            self.position += bytes.len();
            Ok(())
        } else {
            Err(core::fmt::Error)
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
    hero: &mut Hero,
    enemies: &Enemies,
    barricades: &mut [Barricade],
    barricades_alive: usize,
    current_lifes: &mut u8,
) -> Option<EndOfGame> {
    if !hero.structure.alive {
        if *current_lifes == 0 {
            info!("Game over, you lost! You're out of lifes.");
            return Some(Lost(enemies.enemies_dead));
        }
        *current_lifes -= 1;
        info!("Ouch! Lost a life, {current_lifes} left");
        hero.structure.alive = true;
    }

    let all_aliens_dead = TOTAL_ENEMIES - enemies.enemies_dead == 0;
    if all_aliens_dead {
        info!("Game over, you won! All enemies dead.",);
        return Some(Won(enemies.enemies_dead));
    }

    for enemy in enemies.enemies.iter() {
        if !enemy.structure.alive {
            continue;
        }
        let reached_hero = enemy.structure.coordinates.y() + enemy.structure.height
            >= hero.structure.coordinates.y();
        if reached_hero {
            info!("Game over, you lost! Enemy has reached the hero");
            return Some(Lost(enemies.enemies_dead));
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
