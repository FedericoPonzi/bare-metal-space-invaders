use crate::actor::{
    Actor, Barricade, Enemies, Hero, HeroMovementDirection, LivesCount, ScoreCount, Shoots,
};
use crate::EndOfGame::{Lost, Restarted, Won};
#[cfg(feature = "std")]
use crate::FPS;
use crate::{EndOfGame, FrameBufferInterface, TimeManagerInterface, UserInput};
use core::ops::Sub;
use core::time::Duration;
use log::info;

pub struct GameContext<'a, T, F>
where
    F: FrameBufferInterface + UserInput,
    T: TimeManagerInterface,
{
    pub hero: Hero,
    pub time_manager: &'a T,
    fb: &'a mut F,
    shoots: Shoots,
    barricades: [Barricade; 56],
    barricades_alive: usize,
    last_loop: Duration,
    enemies: Enemies,
    random: [u32; 20],
    random_index: usize,
    lives_count: LivesCount,
    score_count: ScoreCount,
}

impl<'a, T, F> GameContext<'a, T, F>
where
    F: FrameBufferInterface + UserInput,
    T: TimeManagerInterface,
{
    pub fn new(
        fb: &'a mut F,
        high_score: u32,
        current_score: u32,
        time_manager: &'a T,
        current_lives: u8,
    ) -> Self {
        let enemies = Enemies::new();
        let shoots = Shoots::new();
        let hero = Hero::new();

        let barricades = Barricade::create_barricades();
        let barricades_alive = barricades.len();
        let score_count = ScoreCount::new(current_score, high_score);
        let lives_count = LivesCount::new(current_lives);

        let last_loop = time_manager.now();

        // super fast random :D
        let random = [
            35, 13, 65, 16, 15, 23, 84, 79, 65, 85, 99, 8, 63, 74, 57, 75, 9, 92, 25, 29,
        ];
        let random_index = 0;

        Self {
            hero,
            time_manager,
            fb,
            shoots,
            barricades,
            barricades_alive,
            last_loop,
            enemies,
            random,
            random_index,
            lives_count,
            score_count,
        }
    }

    pub fn play(&mut self) -> EndOfGame {
        let mut last_draw_loop: Duration = self.time_manager.now();
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
            self.handle_movements(hero_movement_direction, delta_ms);

            // 3. collision detection
            self.shoots.check_collisions(
                &mut self.hero,
                &mut self.enemies,
                &mut self.barricades,
                &mut self.barricades_alive,
            );
            self.score_count.update(self.enemies.enemies_dead);

            // check if game is over.
            if let Some(ret) = self.check_game_over() {
                return ret;
            }
            #[cfg(feature = "no_std")]
            if now.sub(last_draw_loop).as_millis() >= 1000 / crate::FPS {
                info!(
                    "delta since last draw: {}",
                    self.time_manager.since(last_draw_loop).as_millis()
                );
                last_draw_loop = now;

                // Draw things:
                self.fb.clear_screen();
                self.draw();
                self.fb.update();
            }

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

    fn draw(&mut self) {
        self.enemies.draw(self.fb);
        self.hero.draw(self.fb);
        self.shoots.draw(self.fb);
        for b in self.barricades.iter().filter(|b| b.is_alive()) {
            b.draw(self.fb);
        }
        self.score_count.draw(self.fb);
        self.lives_count.draw(self.fb);
    }

    fn handle_movements(&mut self, hero_movement_direction: HeroMovementDirection, delta_ms: u64) {
        self.shoots.handle_movement(delta_ms);
        self.enemies.move_enemies(delta_ms);
        self.hero.handle_movement(hero_movement_direction, delta_ms);
    }

    /// It also check collision of aliens against barricades.
    fn check_game_over(&mut self) -> Option<EndOfGame> {
        if !self.hero.is_alive() {
            if self.lives_count.is_out_of_lives() {
                info!("Game over, you lost! You're out of lifes.");
                return Some(Lost(self.enemies.enemies_dead));
            }
            self.lives_count.decrease();
            //info!("Ouch! Lost a life, {} left", self.current_lifes);
            self.hero.structure.alive = true;
        }

        if self.enemies.all_dead() {
            info!("Game over, you won! All enemies dead.",);
            return Some(Won(self.enemies.enemies_dead));
        }

        for enemy in self.enemies.enemies.iter().filter(|e| e.is_alive()) {
            let reached_hero = enemy.get_coordinates().y() + enemy.structure.height
                >= self.hero.get_coordinates().y();
            if reached_hero {
                info!("Game over, you lost! Enemy has reached the hero");
                return Some(Lost(self.enemies.enemies_dead));
            }
            let reached_barricades = enemy.get_coordinates().y() + enemy.structure.height
                >= self.barricades[0].get_coordinates().y();
            if reached_barricades && self.barricades_alive > 0 {
                for b in self.barricades.iter_mut() {
                    b.structure.alive = false;
                }
            }
        }
        None
    }
}
