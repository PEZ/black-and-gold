use macroquad::experimental::animation::AnimatedSprite;
use macroquad::prelude::*;

use crate::screen_object::ScreenObject;

const MILEI_SIZE: f32 = 32.0;

#[derive(PartialEq)]
pub enum LateralDirection {
    Left,
    None,
    Right,
}

pub struct MileiBullet {
    pub screen_object: ScreenObject,
}

pub struct Milei {
    pub screen_object: ScreenObject,
    pub bullets: Vec<MileiBullet>,
    pub last_bullet_time: f64,
}

impl Milei {
    pub fn new(ship_sprite: &AnimatedSprite, speed: f32) -> Self {
        let ship_sprite_w = ship_sprite.frame().source_rect.w;
        let ship_sprite_h = ship_sprite.frame().source_rect.h;

        Self {
            screen_object: ScreenObject {
                size: MILEI_SIZE,
                speed,
                x: screen_width() / 2.0,
                y: screen_height() / 2.0,
                w: ship_sprite_w * MILEI_SIZE / ship_sprite_w,
                h: ship_sprite_h * MILEI_SIZE / ship_sprite_h,
                color: GOLD,
                collided: false,
            },
            bullets: vec![],
            last_bullet_time: get_time(),
        }
    }

    pub fn start(&mut self, screen_width: f32, screen_height: f32) {
        self.bullets.clear();
        self.screen_object.x = screen_width / 2.0;
        self.screen_object.y = screen_height - self.screen_object.size;
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    pub fn position(&self) -> (f32, f32) {
        (self.screen_object.x, self.screen_object.y)
    }

    pub fn constrain_to_screen(&mut self, screen_width: f32, screen_height: f32) {
      let half_width = self.screen_object.w / 2.0;
      let half_height = self.screen_object.h / 2.0;
        self.screen_object.x = self
            .screen_object
            .x
            .min(screen_width - half_width)
            .max(0.0 + half_width);
        self.screen_object.y = self
            .screen_object
            .y
            .min(screen_height - half_height)
            .max(0.0 + half_height);
    }
}