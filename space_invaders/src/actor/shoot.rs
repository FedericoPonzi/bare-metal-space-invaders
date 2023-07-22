use crate::actor::{
    Actor, ActorStructure, Barricade, Enemies, Enemy, Hero, HERO_HEIGHT, TOTAL_ENEMIES,
};
use crate::framebuffer::color::SHOT_COLOR;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::{FrameBufferInterface, SCREEN_HEIGHT};
use log::info;

pub const SHOOT_BOX_WIDTH: u32 = 3;
pub const SHOOT_BOX_HEIGHT: u32 = 7;
const SHOOT_BOX_COLOR: Color = SHOT_COLOR;

// pixels per millisecond.
const SHOOT_SPEED: f64 = 400.0 / 1000.0;

pub const SHOOT_SPAWN_OFFSET_Y: u32 = HERO_HEIGHT + 10;

pub const SHOOT_ENEMY_MAX: usize = 3;
pub const SHOOT_HERO_MAX: usize = 4;

// max shots available to render at a time
pub const SHOOT_MAX_ALLOC: usize = SHOOT_ENEMY_MAX + SHOOT_HERO_MAX;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ShootOwner {
    Hero,
    Enemy,
}

impl From<&Enemy> for Shoot {
    fn from(enemy: &Enemy) -> Self {
        let enemy_coordinates = enemy.structure.coordinates;
        Self {
            owner: ShootOwner::Enemy,
            structure: Shoot::structure(Coordinates::new(
                enemy_coordinates.x(),
                enemy_coordinates.y() + enemy.structure.height,
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shoot {
    pub(crate) structure: ActorStructure,
    pub(crate) owner: ShootOwner,
}

impl Actor for Shoot {
    fn get_structure(&self) -> &ActorStructure {
        &self.structure
    }

    fn set_coordinates(&mut self, coordinates: Coordinates) {
        self.structure.coordinates = coordinates;
    }
    fn draw(&self, fb: &mut impl FrameBufferInterface) {
        fb.draw_rect_fill(
            self.structure.coordinates,
            self.structure.width,
            self.structure.height,
            SHOOT_BOX_COLOR,
        );
    }
}
impl Default for Shoot {
    fn default() -> Self {
        Self {
            structure: Self::structure(Coordinates::new(0, 0)),
            owner: ShootOwner::Hero,
        }
    }
}

impl Shoot {
    #[inline(always)]
    pub fn new(coordinates: Coordinates, owner: ShootOwner) -> Self {
        Shoot {
            structure: Self::structure(coordinates),
            owner,
        }
    }

    const fn structure(coordinates: Coordinates) -> ActorStructure {
        ActorStructure {
            sprite: None,
            width: SHOOT_BOX_WIDTH,
            height: SHOOT_BOX_HEIGHT,
            alive: true,
            coordinates,
        }
    }

    #[inline(always)]
    pub(crate) fn out_of_screen(&self, screen_height: u32) -> bool {
        let coordinates = self.structure.coordinates;
        (coordinates.y() as i32) - (self.structure.height as i32) <= 0
            || (coordinates.y() + self.structure.height) >= (screen_height)
    }

    #[inline(always)]
    pub(crate) fn move_forward(&mut self, delta: u64) {
        match &self.owner {
            ShootOwner::Hero => {
                self.structure.coordinates.sub_virtual_y(SHOOT_SPEED, delta);
            }
            ShootOwner::Enemy => {
                self.structure.coordinates.add_virtual_y(SHOOT_SPEED, delta);
            }
        }
    }
}
pub struct Shoots {
    hero_shoots: [Shoot; SHOOT_HERO_MAX],
    hero_shoots_alive: usize,
    enemy_shoots: [Shoot; SHOOT_ENEMY_MAX],
    enemy_shoots_alive: usize,
}
impl Shoots {
    pub const fn new() -> Self {
        let hero_shoots: [Shoot; SHOOT_HERO_MAX] =
            [Shoot::new(Coordinates::new(0, 0), ShootOwner::Hero); SHOOT_HERO_MAX];
        let enemy_shoots: [Shoot; SHOOT_ENEMY_MAX] =
            [Shoot::new(Coordinates::new(0, 0), ShootOwner::Enemy); SHOOT_ENEMY_MAX];

        Self {
            hero_shoots,
            enemy_shoots,
            hero_shoots_alive: 0,
            enemy_shoots_alive: 0,
        }
    }
    pub fn create_shoots(&mut self, shoot: Option<Shoot>, rnd: u32, enemies: &mut Enemies) {
        self.handle_hero_shoot(shoot);
        self.handle_enemies_shoot(rnd, enemies);
    }

    fn handle_hero_shoot(&mut self, shoot: Option<Shoot>) {
        if self.hero_shoots_alive >= SHOOT_HERO_MAX || shoot.is_none() {
            return;
        }
        for sh in self.hero_shoots.iter_mut() {
            if !sh.structure.alive {
                sh.structure.alive = true;
                sh.structure = shoot.unwrap().structure;
                self.hero_shoots_alive += 1;
                break;
            }
        }
    }
    fn handle_enemies_shoot(&mut self, rnd: u32, enemies: &mut Enemies) {
        if self.enemy_shoots_alive >= SHOOT_ENEMY_MAX {
            return;
        }
        let enemy_shooting = rnd as usize % (TOTAL_ENEMIES - enemies.enemies_dead);
        for (id, enemy) in enemies
            .enemies
            .iter()
            .filter(|e| e.structure.alive)
            .enumerate()
        {
            if enemy_shooting != id {
                continue;
            }
            for sh in self.enemy_shoots.iter_mut().filter(|sh| sh.structure.alive) {
                self.enemy_shoots_alive += 1;
                sh.structure.alive = true;
                sh.structure.coordinates = enemy.structure.coordinates;
                break;
            }
        }
    }

    pub fn handle_movement(&mut self, delta_ms: u64) {
        for sh in self
            .enemy_shoots
            .iter_mut()
            .chain(self.hero_shoots.iter_mut())
            .filter(|sh| sh.structure.alive)
        {
            sh.move_forward(delta_ms);
            if sh.out_of_screen(SCREEN_HEIGHT as u32) {
                info!("shoot is out of screen!");
                if sh.owner == ShootOwner::Hero {
                    self.hero_shoots_alive -= 1;
                } else {
                    self.enemy_shoots_alive -= 1;
                }
                //remove it.
                sh.structure.alive = false;
            }
        }
    }

    pub fn check_collisions(
        &mut self,
        hero: &mut Hero,
        enemies: &mut Enemies,
        barricades: &mut [Barricade],
        barricades_alive: &mut usize,
    ) {
        // this is not the best way to do it, but it works.
        // The issue here is that if the loop runs really slowly, then the shoot will overlap
        // with the enemies in very few positions. OFC, if the game is running with so few fps,
        // it would be unplayable anyway.

        for shoot in &mut self.hero_shoots.iter_mut().filter(|sh| sh.structure.alive) {
            for (actor, is_enemy) in enemies
                .enemies
                .iter_mut()
                .map(|e| (&mut e.structure, 1))
                .chain(barricades.iter_mut().map(|e| (&mut e.structure, 0)))
                .filter(|a| a.0.alive)
                .filter(|(actor, _)| shoot.is_hit(actor))
            {
                actor.alive = false;
                enemies.enemies_dead += is_enemy;
                *barricades_alive -= usize::from(is_enemy == 0);
                shoot.structure.alive = false;
                self.hero_shoots_alive -= 1;
                break;
            }
        }
        for shoot in &mut self.enemy_shoots.iter_mut().filter(|sh| sh.structure.alive) {
            if shoot.is_hit(hero.get_structure()) {
                shoot.structure.alive = false;
                hero.structure.alive = false;
                // no need to continue, game is over anyway.
                return;
            }
            for b in barricades.iter_mut().filter(|ba| ba.structure.alive) {
                if shoot.is_hit(b.get_structure()) {
                    shoot.structure.alive = false;
                    b.structure.alive = false;
                    *barricades_alive -= 1;
                    enemies.enemy_shoots -= 1;
                    break;
                }
            }
        }
    }
    pub fn draw(&self, fb: &mut impl FrameBufferInterface) {
        for shoot in self
            .enemy_shoots
            .iter()
            .chain(self.hero_shoots.iter())
            .filter(|sh| sh.structure.alive)
        {
            shoot.draw(fb);
        }
    }
}
