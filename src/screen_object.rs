use macroquad::prelude::*;

pub struct ScreenObject {
  pub size: f32,
  pub speed: f32,
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
  pub color: Color,
  pub collided: bool,
}

impl ScreenObject {
  pub fn collides_with_circle(&self, circle: &ScreenObject) -> bool {
      let half = self.size / 2.0;
      let dx = (self.x - circle.x).abs().max(half) - half;
      let dy = (self.y - circle.y).abs().max(half) - half;
      dx * dx + dy * dy <= circle.size * circle.size / 4.0
  }

  pub fn collides_with(&self, other: &Self) -> bool {
      self.rect().overlaps(&other.rect())
  }

  fn rect(&self) -> Rect {
      Rect {
          x: self.x - self.size / 2.0,
          y: self.y - self.size / 2.0,
          w: self.w,
          h: self.h,
      }
  }
}
