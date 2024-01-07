use macroquad::experimental::animation::AnimatedSprite;
use macroquad::prelude::*;

use crate::screen_object::ScreenObject;

pub enum Vastness {
    XXS = 0,
    XS = 1,
    S = 2,
    M = 3,
    L = 4,
    XL = 5,
    XXL = 6,
    XXXL = 7,
    XXXXL = 8,
    XXXXXL = 9,
}

impl Vastness {
    fn from_int(value: usize) -> Option<Self> {
        match value {
            0 => Some(Vastness::XXS),
            1 => Some(Vastness::XS),
            2 => Some(Vastness::S),
            3 => Some(Vastness::M),
            4 => Some(Vastness::L),
            5 => Some(Vastness::XL),
            6 => Some(Vastness::XXL),
            7 => Some(Vastness::XXXL),
            8 => Some(Vastness::XXXXL),
            9 => Some(Vastness::XXXXXL),
            _ => None,
        }
    }

    pub fn to_float(&self) -> f32 {
        match self {
            Vastness::XXS => 16.0,
            Vastness::XS => 20.0,
            Vastness::S => 24.0,
            Vastness::M => 28.0,
            Vastness::L => 32.0,
            Vastness::XL => 36.0,
            Vastness::XXL => 40.0,
            Vastness::XXXL => 48.0,
            Vastness::XXXXL => 56.0,
            Vastness::XXXXXL => 64.0,
        }
    }

    pub fn choose_one() -> Self {
        let value = rand::gen_range(0, 9);
        Vastness::from_int(value).unwrap()
    }
}

pub struct Goon {
    pub id: usize,
    pub vastness: Vastness,
    pub screen_object: ScreenObject,
    pub bullet_count: usize,
}

impl Goon {}

pub struct GovernmentBullet {
    pub goon_id: usize,
    pub screen_object: ScreenObject,
}

pub struct Government {
    pub goons: Vec<Goon>,
    pub bullets: Vec<GovernmentBullet>,
    next_goon_id: usize,
    _ship_sprite: AnimatedSprite,
    ship_sprite_w: f32,
    ship_sprite_h: f32,
}

impl Government {
    pub fn new(ship_sprite: AnimatedSprite) -> Self {
        let ship_sprite_w = ship_sprite.frame().source_rect.w;
        let ship_sprite_h = ship_sprite.frame().source_rect.h;

        Self {
            goons: vec![],
            bullets: vec![],
            next_goon_id: 0,
            _ship_sprite: ship_sprite,
            ship_sprite_w,
            ship_sprite_h,
        }
    }

    pub fn spawn_goon(&mut self, vastness: Vastness, size: f32, speed: f32, x: f32, y: f32) {
        let w = self.ship_sprite_w * size / self.ship_sprite_w;
        let h = self.ship_sprite_h * size / self.ship_sprite_h;

        self.goons.push(Goon {
            id: self.next_goon_id,
            vastness,
            bullet_count: 0,
            screen_object: ScreenObject {
                size,
                speed,
                x,
                y,
                w,
                h,
                color: WHITE,
                collided: false,
            },
        });
        self.next_goon_id += 1;
    }

    pub fn start(&mut self) {
        self.goons.clear();
        self.bullets.clear();
        self.next_goon_id = 0;
    }

    pub fn num_goons(&self) -> usize {
        self.goons.len()
    }

    pub fn update_goons(&mut self, delta_time: f32) {
        for goon in &mut self.goons {
            goon.screen_object.y += goon.screen_object.speed * delta_time;
        }
    }

    pub fn update_bullets(&mut self, delta_time: f32) {
        for bullet in &mut self.bullets {
            bullet.screen_object.y += bullet.screen_object.speed * delta_time;
        }
    }

    pub fn has_hit_screen_object(&self, other: &ScreenObject) -> bool {
        self.goons
            .iter()
            .any(|goon| goon.screen_object.collides_with_circle(other))
    }

    pub fn has_bullet_hit_screen_object(&self, other: &ScreenObject) -> bool {
        self.bullets
            .iter()
            .any(|bullet| bullet.screen_object.collides_with_circle(other))
    }
}
