use crate::actor::{Actor, ActorStructure, Enemies, Enemy, HERO_HEIGHT, TOTAL_ENEMIES};
use crate::framebuffer::color::SHOT_COLOR;
use crate::framebuffer::coordinates::Coordinates;
use crate::framebuffer::Color;
use crate::FrameBufferInterface;
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

pub fn create_shoots(
    shoot: Option<Shoot>,
    hero_shoots: &mut usize,
    rnd: u32,
    shoots: &mut [Option<Shoot>],
    enemies2: &mut Enemies,
) {
    handle_hero_shoot(shoot, hero_shoots, shoots);
    enemies2.handle_enemies_shoot(rnd, shoots);
}

fn handle_hero_shoot(shoot: Option<Shoot>, hero_shoots: &mut usize, shoots: &mut [Option<Shoot>]) {
    if *hero_shoots < SHOOT_HERO_MAX && let Some(shoot) = shoot {
        for sh in shoots.iter_mut() {
            if sh.is_none() {
                sh.replace(shoot);
                *hero_shoots += 1;
                break;
            }
        }
    }
}

pub fn shoots_handle_movement(
    fb: &impl FrameBufferInterface,
    shoots: &mut [Option<Shoot>],
    enemy_shoots: &mut usize,
    hero_shoots: &mut usize,
    delta_ms: u64,
) {
    for sh in shoots.iter_mut() {
        if let Some(shoot) = sh.as_mut() {
            shoot.move_forward(delta_ms);
            if shoot.out_of_screen(fb.height() as u32) {
                info!("shoot is out of screen!");
                if shoot.owner == ShootOwner::Hero {
                    *hero_shoots -= 1;
                } else {
                    *enemy_shoots -= 1;
                }
                //remove it.
                let _ = sh.take();
            }
        }
    }
}
