use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

const MOVEMENT_SPEED: f32 = 200.0;
const BALL_RADIUS: f32 = 32.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}

#[macroquad::main("Mitt spel")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    let colors: Vec<Color> = vec![BLACK, RED, BLUE, GREEN, PINK, SKYBLUE, DARKBLUE];

    let mut squares = vec![];
    let mut circle = Shape {
        size: BALL_RADIUS,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        color: DARKPURPLE,
    };

    let mut game_over = false;

    loop {
        clear_background(GOLD);
        let delta_time = get_frame_time();
        let movement = delta_time * MOVEMENT_SPEED;

        if game_over && is_key_pressed(KeyCode::Space) {
            squares.clear();
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            game_over = false;
        }

        if !game_over {
            if is_key_down(KeyCode::Right) {
                circle.x += movement;
            }
            if is_key_down(KeyCode::Left) {
                circle.x -= movement;
            }
            if is_key_down(KeyCode::Down) {
                circle.y += movement;
            }
            if is_key_down(KeyCode::Up) {
                circle.y -= movement;
            }

            circle.x = circle
                .x
                .min(screen_width() - BALL_RADIUS / 2.0)
                .max(0.0 + BALL_RADIUS / 2.0);
            circle.y = circle
                .y
                .min(screen_height() - BALL_RADIUS / 2.0)
                .max(0.0 + BALL_RADIUS / 2.0);

            if rand::gen_range(0, 99) >= 95 {
                let size = rand::gen_range(16.0, 64.0);
                squares.push(Shape {
                    size,
                    speed: rand::gen_range(50.0, 150.0),
                    x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                    y: -size,
                    color: *colors.choose().unwrap(),
                });
            }

            for square in &mut squares {
                square.y += square.speed * delta_time;
            }

            if squares.iter().any(|square| circle.collides_with(square)) {
                game_over = true;
            }

            squares.retain(|square| square.y < screen_width() + square.size);
        }

        for square in &squares {
            draw_rectangle(
                square.x - square.size / 2.0,
                square.y - square.size / 2.0,
                square.size,
                square.size,
                square.color,
            );
        }

        draw_circle(circle.x, circle.y, circle.size, circle.color);

        if game_over {
            let game_over_text = "GAME OVER! Press space to restart.";
            let text_width = measure_text(game_over_text, None, 32, 1.0).width;
            let text_x = (screen_width() - text_width) / 2.0;
            let text_y = screen_height() / 2.0;

            draw_text(game_over_text, text_x, text_y, 32.0, WHITE);
        }

        next_frame().await
    }
}
