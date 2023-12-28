#[macro_use]
extern crate lazy_static;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad_particles::{self as particles, ColorCurve, Emitter, EmitterConfig};
use std::f32::consts::PI;
use std::fs;

const GAME_TITLE: &str = "¡AFUERA!";
const MOVEMENT_SPEED: f32 = 200.0;
const STARFIELD_SPEED: f32 = 0.01;
const BALL_RADIUS: f32 = 16.0;
const MAX_BULLETS_PER_SECOND: f64 = 4.0;

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
// attribute vec2 texcoord;
// attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

lazy_static! {
    static ref ENEMY_COLORS: Vec<Color> = vec![
        BEIGE, BLUE, BROWN, DARKBLUE, DARKBROWN, DARKGRAY, DARKGREEN, DARKPURPLE, GRAY, GREEN,
        LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, SKYBLUE, VIOLET, YELLOW,
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

fn oscillating_alpha(base_color: Color, cycles_per_second: f32) -> Color {
    let alpha = 0.5 * (1.0 + f32::sin(cycles_per_second * get_time() as f32 * PI / 2.0));
    Color::new(base_color.r, base_color.g, base_color.b, alpha)
}

fn draw_game_title() {
    let text = GAME_TITLE;
    let font_size = 144;
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - text_dimensions.width / 2.0,
        screen_height() / 4.0,
        font_size as f32,
        GOLD,
    );
}

fn particle_explosion() -> particles::EmitterConfig {
    particles::EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.6,
        lifetime_randomness: 0.3,
        explosiveness: 0.65,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 300.0,
        initial_velocity_randomness: 0.8,
        size: 3.0,
        size_randomness: 0.3,
        colors_curve: ColorCurve {
            start: RED,
            mid: ORANGE,
            end: RED,
        },
        ..Default::default()
    }
}

fn draw_game_objects(
    squares: &[Shape],
    bullets: &[Shape],
    circle: &Shape,
    explosions: &mut [(Emitter, Vec2)],
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
    
    for (explosion, coords) in explosions.iter_mut() {
        explosion.draw(*coords);
    }

    draw_circle(circle.x, circle.y, circle.size / 2.0, circle.color);

    draw_text(format!("Score: {}", score).as_str(), 10.0, 35.0, 25.0, GOLD);
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
        GOLD,
    );

    if high_score_beaten {
        let text_dimensions = measure_text(high_score_beaten_text, None, 25, 1.0);
        draw_text(
            high_score_beaten_text,
            screen_width() - text_dimensions.width - 10.0,
            35.0 + text_dimensions.height + text_dimensions.offset_y,
            25.0,
            oscillating_alpha(GOLD, 3.0),
        );
    }
}

#[macroquad::main("¡Viva la libertad, CARAJO!")]
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
        color: GOLD,
        collided: false,
    };

    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let mut material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                ("iResolution".to_owned(), UniformType::Float2),
                ("direction_modifier".to_owned(), UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    let (tx, rx) = channel();
    let config = Config::default().with_poll_interval(Duration::from_secs(2));
    let mut watcher: RecommendedWatcher = Watcher::new(tx, config).unwrap();
    watcher
        .watch(
            Path::new("src/starfield-shader.glsl"),
            RecursiveMode::Recursive,
        )
        .unwrap();

    let mut explosions: Vec<(Emitter, Vec2)> = vec![];

    let mut game_state = GameState::MainMenu;

    loop {
        clear_background(BLACK);

        match rx.try_recv() {
            Ok(event) => {
                println!("File change detected: {:?}", event);
                match fs::read_to_string("src/starfield-shader.glsl") {
                    Ok(shader_code) => {
                        material = load_material(
                            ShaderSource::Glsl {
                                vertex: VERTEX_SHADER,
                                fragment: &shader_code,
                            },
                            MaterialParams {
                                uniforms: vec![
                                    ("iResolution".to_owned(), UniformType::Float2),
                                    ("direction_modifier".to_owned(), UniformType::Float1),
                                ],
                                ..Default::default()
                            },
                        )
                        .unwrap_or_else(|e| {
                            println!("Error reloading shader: {:?}", e);
                            material
                        });
                    }
                    Err(e) => println!("Error reading shader file: {:?}", e),
                }
            }
            Err(_) => {}
        }

        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);

        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    squares.clear();
                    bullets.clear();
                    explosions.clear();
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

                draw_text(text, text_x, text_y, 32.0, GOLD);
                draw_game_title();
            }
            GameState::Playing => {
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }
                let delta_time = get_frame_time();
                let my_movement = delta_time * MOVEMENT_SPEED;
                let star_movement = delta_time * STARFIELD_SPEED;

                if is_key_down(KeyCode::Right) {
                    circle.x += my_movement;
                    direction_modifier += star_movement;
                }
                if is_key_down(KeyCode::Left) {
                    circle.x -= my_movement;
                    direction_modifier -= star_movement;
                }
                if is_key_down(KeyCode::Down) {
                    circle.y += my_movement;
                }
                if is_key_down(KeyCode::Up) {
                    circle.y -= my_movement;
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
                        color: GOLD,
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
                        color: *ENEMY_COLORS.choose().unwrap(),
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
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 2,
                                    ..particle_explosion()
                                }),
                                vec2(bullet.x, bullet.y),
                            ));
                        }
                    }
                }

                squares.retain(|square| square.y < screen_width() + square.size);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| !bullet.collided);
                explosions.retain(|(explosion, _)| explosion.config.emitting);

                draw_game_objects(
                    &squares,
                    &bullets,
                    &circle,
                    &mut explosions,
                    score,
                    high_score,
                    high_score_beaten,
                );
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                draw_game_objects(
                    &squares,
                    &bullets,
                    &circle,
                    &mut explosions,
                    score,
                    high_score,
                    high_score_beaten,
                );
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 32, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    32.0,
                    GOLD,
                );
                draw_game_title();
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::MainMenu;
                }
                draw_game_objects(
                    &squares,
                    &bullets,
                    &circle,
                    &mut explosions,
                    score,
                    high_score,
                    high_score_beaten,
                );
                let game_over_text = "GAME OVER!";
                let text_dimensions = measure_text(game_over_text, None, 32, 1.0);

                let text_x = (screen_width() - text_dimensions.width) / 2.0;
                let text_y =
                    screen_height() / 2.0 - text_dimensions.offset_y + text_dimensions.height;

                draw_text(game_over_text, text_x, text_y, 32.0, GOLD);
                draw_game_title();
            }
        }

        next_frame().await
    }
}
