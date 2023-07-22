use crate::actor::{
    Barricade, EnemiesDirection, Enemy, Hero, ShootOwner, ENEMY_COLS, SHOOT_ENEMY_MAX,
    SHOOT_HERO_MAX, SHOOT_MAX_ALLOC, TOTAL_ENEMIES,
};
use crate::framebuffer::fb_trait::{UI_SCORE_COLOR, UI_SCORE_COORDINATES};
use crate::EndOfGame::Restarted;
use crate::{
    init_enemies, move_enemies, Actor, EndOfGame, FrameBufferInterface, Shoot,
    TimeManagerInterface, FPS,
};
use core::alloc;
use core::cmp;
use core::ops::Sub;
use core::time::Duration;
use log::info;
use rand::random;
/*
enum KeyPressedKeys {
    Left,
    Right,
    Shoot,
    Pause,
}
trait MemoryAllocator {
    fn alloc(&self, layout: alloc::Layout) -> *mut u8;
}
trait UserInput {
    fn get_input(&self) -> KeyPressedKeys;
}

struct GameContext<F, A, U, T>
where
    T: TimeManagerInterface,
    F: FrameBufferInterface,
    A: MemoryAllocator,
    U: UserInput,
{
    pub hero: Hero,
    pub enemies: Vec<Enemy>,
    pub high_score: u32,
    pub current_score: u32,
    pub time_manager: T,
    fb: F,
    allocator: A,
    user_input: U,
}

impl<F, A, U, T> GameContext<F, A, U, T>
where
    T: TimeManagerInterface,
    F: FrameBufferInterface,
    A: MemoryAllocator,
    U: UserInput,
{
    pub fn new(
        fb: F,
        user_input: U,
        allocator: A,
        time_manager: T,
        high_score: u32,
        current_score: u32,
    ) -> Self {
        Self {
            hero: Hero::new(&fb),
            enemies: vec![],
            current_score,
            allocator,
            user_input,
            high_score,
            fb,
            time_manager,
        }
    }

    fn init_game(&mut self) -> EndOfGame {
        let mut enemies = init_enemies(&self.fb);

        // todo, instead of using option just set alive: false,
        let mut shoots: [Option<Shoot>; SHOOT_MAX_ALLOC] = [None; SHOOT_MAX_ALLOC];
        let mut hero_shoots = 0;
        let mut enemy_shoots = 0;

        let mut hero = Hero::new(&self.fb);

        let mut direction = EnemiesDirection::Right;
        let mut last_loop = self.time_manager.now();
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
                return Restarted;
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
                return EndOfGame::Lost(enemies_dead as u32);
            }

            let all_aliens_dead = TOTAL_ENEMIES - enemies_dead == 0;
            if all_aliens_dead {
                info!("Game over, you won! All enemies dead.",);
                return EndOfGame::Won(enemies_dead as u32);
            }

            for enemy in enemies.iter() {
                if !enemy.structure.alive {
                    continue;
                }
                let reached_hero = enemy.structure.coordinates.y() + enemy.structure.height
                    >= hero.structure.coordinates.y();
                if reached_hero {
                    info!("Game over, you lost! Enemy has reached the hero");
                    return EndOfGame::Lost(enemies_dead as u32);
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
            let current_score_updated = current_score + enemies_dead as u32;
            let high_score_updated = cmp::max(current_score_updated, high_score);
            let message = format!(
                "High Score: {high_score_updated} - Current Score: {current_score_updated}"
            );
            fb.write_ui(UI_SCORE_COORDINATES, &message, UI_SCORE_COLOR);
            info!("Updating fb...");
            fb.update();
            info!("Loop completed");

            #[cfg(feature = "std")]
            let delta_next = Duration::from_millis(1000 / FPS as u64)
                .saturating_sub(time_manager.since(last_loop));
            #[cfg(feature = "std")]
            if delta_next.as_millis() > 0 {
                #[cfg(feature = "std")]
                std::thread::sleep(delta_next);
            }
        }
    }
}
*/
