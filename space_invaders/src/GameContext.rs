use crate::actor::{Enemy, Hero, ShootOwner, SHOOT_ENEMY_MAX, SHOOT_HERO_MAX, TOTAL_ENEMIES};
use crate::framebuffer::fb_trait::{UI_SCORE_COLOR, UI_SCORE_COORDINATES};
use crate::{move_enemies, FrameBufferInterface, TimeManagerInterface, FPS};
use core::alloc;
use log::info;
use rand::random;
use std::time::Duration;

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
    pub fn new(fb: F, user_input: U, allocator: A, high_score: u32) -> Self {
        Self {
            hero: Hero::new(&fb),
            enemies: vec![],
            current_score: 0,
            allocator,
            user_input,
            high_score,
            fb,
        }
    }
    pub fn play(mut self) -> bool {

    }

    fn loop_game(&mut self) -> bool {
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
        true;
    }
}
*/
