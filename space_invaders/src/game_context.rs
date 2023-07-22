use crate::actor::{Actor, Barricade, Enemies, Hero, Shoots, TOTAL_ENEMIES};
use crate::framebuffer::fb_trait::{FrameBufferInterface, UI_SCORE_COLOR, UI_SCORE_COORDINATES};
use crate::EndOfGame::{Lost, Restarted, Won};
use crate::{EndOfGame, MemoryAllocator, TimeManagerInterface, UserInput, FPS};
use core::cmp;
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
}

impl<'a, T, F> GameContext<'a, T, F>
where
    F: FrameBufferInterface + MemoryAllocator + UserInput,
    T: TimeManagerInterface,
{
    pub fn new(fb: &'a mut F, high_score: u32, current_score: u32, time_manager: &'a T) -> Self {
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
        }
    }

    pub fn play(&mut self) -> EndOfGame {
        loop {
            let now = self.time_manager.now();
            let delta_ms = now.sub(self.last_loop).as_millis() as u64;
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

            // 2. Handle shoots. Create if hero's or enemies' as needed.
            self.shoots.create_shoots(shoot, rnd, &mut self.enemies);

            // 2. Movement
            handle_movements(
                &mut self.shoots,
                &mut self.hero,
                hero_movement_direction,
                delta_ms,
                &mut self.enemies,
            );

            // 3. collision detection
            self.shoots.check_collisions(
                &mut self.hero,
                &mut self.enemies,
                &mut self.barricades,
                &mut self.barricades_alive,
            );

            // check if game is over.
            if let Some(ret) = check_game_over(
                &self.hero,
                &self.enemies,
                &mut self.barricades,
                self.barricades_alive,
            ) {
                return ret;
            }

            // Draw things:
            draw(
                self.fb,
                &self.hero,
                &self.enemies,
                &self.shoots,
                &self.barricades,
            );

            let current_score_updated = self.current_score + self.enemies.enemies_dead as u32;
            let high_score_updated = cmp::max(current_score_updated, self.high_score);
            let message = format!(
                "High Score: {high_score_updated} - Current Score: {current_score_updated}"
            );
            self.fb
                .write_ui(UI_SCORE_COORDINATES, &message, UI_SCORE_COLOR);
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
