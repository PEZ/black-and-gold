#[macro_use]
extern crate lazy_static;

use macroquad::rand::ChooseRandom;
use macroquad::prelude::*;
use std::f32::consts::PI;
use std::fs;

const MOVEMENT_SPEED: f32 = 200.0;
const BALL_RADIUS: f32 = 32.0;
const MAX_BULLETS_PER_SECOND: f64 = 4.0;

fn oscillating_alpha(base_color: Color, cycles_per_second: f32) -> Color {
    let alpha = 0.5 * (1.0 + f32::sin(cycles_per_second * get_time() as f32 * PI / 2.0));
    Color::new(base_color.r, base_color.g, base_color.b, alpha)
}

lazy_static! {
    static ref COLORS: Vec<Color> = vec![
        BEIGE, BLACK, BLUE, BROWN, DARKBLUE, DARKBROWN, DARKGRAY, DARKGREEN, DARKPURPLE, GRAY,
        GREEN, LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, SKYBLUE, VIOLET, BLACK, YELLOW,
    ];
}

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
    collided: bool,
}

impl Shape {
    fn collides_with_circle(&self, circle: &Shape) -> bool {
        let half = self.size / 2.0;
        let dx = (self.x - circle.x).abs().max(half) - half;
        let dy = (self.y - circle.y).abs().max(half) - half;
        dx * dx + dy * dy <= circle.size * circle.size / 4.0
    }
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

enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

fn draw_game_objects(
    squares: &[Shape],
    bullets: &[Shape],
    circle: &Shape,
    score: u32,
    high_score: u32,
    high_score_beaten: bool,
) {
    for square in squares {
        draw_rectangle(
            square.x - square.size / 2.0,
            square.y - square.size / 2.0,
            square.size,
            square.size,
            square.color,
        );
    }

    for bullet in bullets {
        draw_rectangle(
            bullet.x - bullet.size / 2.0,
            bullet.y - bullet.size / 2.0,
            bullet.size,
            bullet.size,
            bullet.color,
        );
    }
    draw_circle(circle.x, circle.y, circle.size / 2.0, circle.color);

    draw_text(
        format!("Score: {}", score).as_str(),
        10.0,
        35.0,
        25.0,
        BLACK,
    );
    let high_score_text = format!("High score: {}", high_score);
    let high_score_beaten_text = if high_score_beaten {
        "New high score!"
    } else {
        ""
    };

    let text_dimensions = measure_text(high_score_text.as_str(), None, 25, 1.0);
    draw_text(
        high_score_text.as_str(),
        screen_width() - text_dimensions.width - 10.0,
        35.0,
        25.0,
        BLACK,
    );

    if high_score_beaten {
        let text_dimensions = measure_text(high_score_beaten_text, None, 25, 1.0);
        draw_text(
            high_score_beaten_text,
            screen_width() - text_dimensions.width - 10.0,
            35.0 + text_dimensions.height + text_dimensions.offset_y,
            25.0,
            oscillating_alpha(BLACK, 3.0),
        );
    }
}

#[macroquad::main("My first Macroquad game")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);

    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    let mut high_score_beaten = false;

    let mut last_bullet_time = get_time();
    let mut squares = vec![];
    let mut bullets: Vec<Shape> = vec![];
    let mut circle = Shape {
        size: BALL_RADIUS * 2.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        color: DARKPURPLE,
        collided: false,
    };

    let mut game_state = GameState::MainMenu;

    loop {
        clear_background(GOLD);

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                if is_key_pressed(KeyCode::Space) {
                    squares.clear();
                    bullets.clear();
                    circle.x = screen_width() / 2.0;
                    circle.y = screen_height() / 2.0;
                    score = 0;
                    high_score_beaten = false;
                    game_state = GameState::Playing;
                }
                let text = "Press space to start.";
                let text_dimensions = measure_text(text, None, 32, 1.0);

                let text_x = (screen_width() - text_dimensions.width) / 2.0;
                let text_y =
                    screen_height() / 2.0 - text_dimensions.offset_y + text_dimensions.height;

                draw_text(text, text_x, text_y, 32.0, BLACK);
            }
            GameState::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }
                let delta_time = get_frame_time();
                let movement = delta_time * MOVEMENT_SPEED;

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
                    .min(screen_width() - BALL_RADIUS)
                    .max(0.0 + BALL_RADIUS);
                circle.y = circle
                    .y
                    .min(screen_height() - BALL_RADIUS)
                    .max(0.0 + BALL_RADIUS);

                if is_key_pressed(KeyCode::Space)
                    && get_time() - last_bullet_time > 1.0 / MAX_BULLETS_PER_SECOND
                {
                    last_bullet_time = get_time();
                    bullets.push(Shape {
                        x: circle.x,
                        y: circle.y,
                        speed: circle.speed * 2.0,
                        color: MAROON,
                        size: 5.0,
                        collided: false,
                    });
                }

                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    squares.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        color: *COLORS.choose().unwrap(),
                        collided: false,
                    });
                }

                for square in &mut squares {
                    square.y += square.speed * delta_time;
                }
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }

                if squares
                    .iter()
                    .any(|square| square.collides_with_circle(&circle))
                {
                    if score == high_score {
                        fs::write("highscore.dat", high_score.to_string()).ok();
                    }
                    game_state = GameState::GameOver;
                }

                for square in squares.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collides_with(square) {
                            bullet.collided = true;
                            square.collided = true;
                            score += square.size.round() as u32;
                            if score > high_score {
                                high_score_beaten = true;
                                high_score = score;
                            }
                        }
                    }
                }

                squares.retain(|square| square.y < screen_width() + square.size);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| !bullet.collided);

                draw_game_objects(&squares, &bullets, &circle, score, high_score, high_score_beaten);
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                draw_game_objects(&squares, &bullets, &circle, score, high_score, high_score_beaten);
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 32, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    32.0,
                    BLACK,
                );
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::MainMenu;
                }
                draw_game_objects(&squares, &bullets, &circle, score, high_score, high_score_beaten);
                let game_over_text = "GAME OVER!";
                let text_dimensions = measure_text(game_over_text, None, 32, 1.0);

                let text_x = (screen_width() - text_dimensions.width) / 2.0;
                let text_y =
                    screen_height() / 2.0 - text_dimensions.offset_y + text_dimensions.height;

                draw_text(game_over_text, text_x, text_y, 32.0, BLACK);
            }
        }

        next_frame().await
    }
}
