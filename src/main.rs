#[macro_use]
extern crate lazy_static;

mod ios;

use std::f32::consts::PI;

use macroquad::audio::{play_sound, play_sound_once, set_sound_volume, PlaySoundParams};

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad::ui::{hash, root_ui};
use macroquad_particles::{self as particles, AtlasConfig, Emitter, EmitterConfig};

mod resources;
use crate::resources::Resources;

mod simple_logger;

const GAME_TITLE: &str = "¡AFUERA!";
const MOVEMENT_SPEED: f32 = 400.0;
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

fn save_high_score(score: u32) {
    let storage = &mut quad_storage::STORAGE.lock().unwrap();
    storage.set("highscore", &score.to_string());
}

fn load_high_score() -> u32 {
    let storage = &mut quad_storage::STORAGE.lock().unwrap();
    storage
        .get("highscore")
        .unwrap_or("0".to_string())
        .parse::<u32>()
        .unwrap()
}

lazy_static! {
    static ref GOVERNMENT_COLORS: Vec<Color> = vec![
        BEIGE, BLUE, BROWN, DARKBLUE, DARKBROWN, DARKGRAY, DARKGREEN, DARKPURPLE, GRAY, GREEN,
        LIME, MAGENTA, MAROON, ORANGE, PINK, PURPLE, RED, SKYBLUE, VIOLET, YELLOW,
    ];
}

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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
            w: self.w,
            h: self.h,
        }
    }
}

struct Government {
    id: usize,
    shape: Shape,
    bullet_count: usize,
}
struct GovernmentBullet {
    government_id: usize,
    shape: Shape,
}

struct MileiBullet {
    shape: Shape,
}

struct Milei {
    shape: Shape,
    bullet_count: usize,
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
        initial_velocity: 400.0,
        initial_velocity_randomness: 0.8,
        size: 12.0,
        size_randomness: 0.3,
        atlas: Some(AtlasConfig::new(5, 1, 0..)),
        // colors_curve: ColorCurve {
        //     start: RED,
        //     mid: ORANGE,
        //     end: RED,
        // },
        ..Default::default()
    }
}

fn draw_game_objects(
    goons: &[Government],
    bullets: &[Shape],
    government_bullets: &[GovernmentBullet],
    milei: &Milei,
    explosions: &mut [(Emitter, Vec2)],
    bullet_sprite: &AnimatedSprite,
    government_bullet_sprite: &AnimatedSprite,
    ship_sprite: &AnimatedSprite,
    government_small_sprite: &AnimatedSprite,
    resources: &Resources,
) {
    let government_frame: animation::AnimationFrame = government_small_sprite.frame();
    for government in goons {
        draw_texture_ex(
            &resources.government_small_texture,
            government.shape.x - government.shape.size / 2.0,
            government.shape.y - government.shape.size / 2.0,
            WHITE, // square.color,
            DrawTextureParams {
                dest_size: Some(vec2(government.shape.size, government.shape.size)),
                source: Some(government_frame.source_rect),
                ..Default::default()
            },
        );
    }

    let bullet_frame = government_bullet_sprite.frame();
    for bullet in government_bullets {
        draw_texture_ex(
            &resources.bullet_texture,
            bullet.shape.x - bullet.shape.size / 2.0,
            bullet.shape.y - bullet.shape.size / 2.0,
            bullet.shape.color,
            DrawTextureParams {
                dest_size: Some(vec2(bullet.shape.size, bullet.shape.size)),
                source: Some(bullet_frame.source_rect),
                rotation: PI,
                ..Default::default()
            },
        );
    }

    let bullet_frame = bullet_sprite.frame();
    for bullet in bullets {
        draw_texture_ex(
            &resources.bullet_texture,
            bullet.x - bullet.size / 2.0,
            bullet.y - bullet.size / 2.0,
            bullet.color,
            DrawTextureParams {
                dest_size: Some(vec2(bullet.size, bullet.size)),
                source: Some(bullet_frame.source_rect),
                ..Default::default()
            },
        );
    }

    let ship_frame = ship_sprite.frame();
    draw_texture_ex(
        &resources.milei_texture,
        milei.shape.x - ship_frame.dest_size.x,
        milei.shape.y - ship_frame.dest_size.y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(ship_frame.dest_size * 2.0),
            source: Some(ship_frame.source_rect),
            ..Default::default()
        },
    );

    for (explosion, coords) in explosions.iter_mut() {
        explosion.draw(*coords);
    }
}

fn draw_score(score: u32, high_score: u32, high_score_beaten: bool) {
    #[cfg(target_os = "ios")]
    info!("BOOM! safe area insets: {}", ios::get_safe_area_insets());
    let insets = ios::get_safe_area_insets();
    let top = 35.0 + insets.top as f32;

    draw_text(format!("Score: {}", score).as_str(), 10.0, top, 25.0, GOLD);
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
        top,
        25.0,
        GOLD,
    );

    if high_score_beaten {
        let text_dimensions = measure_text(high_score_beaten_text, None, 25, 1.0);
        draw_text(
            high_score_beaten_text,
            screen_width() - text_dimensions.width - 10.0,
            top + text_dimensions.height + text_dimensions.offset_y,
            25.0,
            oscillating_alpha(GOLD, 3.0),
        );
    }
}

#[macroquad::main("¡Viva la libertad, CARAJO!")]
async fn main() -> Result<(), macroquad::Error> {
    rand::srand(miniquad::date::now() as u64);

    simple_logger::setup_logger();

    log::info!("¡AFUERA!");

    let base_width = 750.0;
    let base_goons = 30;

    let mut score: u32 = 0;
    let mut high_score: u32 = load_high_score();
    let mut high_score_beaten = false;

    let mut last_bullet_time = get_time();
    let mut goons = vec![];
    let mut next_government_id = 0;
    let mut bullets: Vec<Shape> = vec![];
    let mut government_bullets: Vec<GovernmentBullet> = vec![];

    let mut direction_modifier: f32 = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
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
    )?;

    let mut explosions: Vec<(Emitter, Vec2)> = vec![];

    let mut game_state = GameState::MainMenu;

    set_pc_assets_folder("assets");
    Resources::load().await?;
    let resources = storage::get::<Resources>();

    play_sound(
        &resources.theme_music,
        PlaySoundParams {
            looped: true,
            volume: 0.1,
        },
    );

    let mut ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left1".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left2".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right1".to_string(),
                row: 3,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right2".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );

    let mut left_direction_time = get_time();
    let mut right_direction_time = get_time();

    let milei_size = BALL_RADIUS * 2.0;
    let ship_sprite_w = ship_sprite.frame().source_rect.w;
    let ship_sprite_h = ship_sprite.frame().source_rect.h;
    let mut milei = Milei {
        shape: Shape {
            size: milei_size,
            speed: MOVEMENT_SPEED,
            x: screen_width() / 2.0,
            y: screen_height() / 2.0,
            w: ship_sprite_w * milei_size / ship_sprite_w,
            h: ship_sprite_h * milei_size / ship_sprite_h,
            color: GOLD,
            collided: false,
        },
        bullet_count: 0,
    };

    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[Animation {
            name: "bolt".to_string(),
            row: 1,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    bullet_sprite.set_animation(0);

    let mut government_bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[Animation {
            name: "bolt".to_string(),
            row: 1,
            frames: 2,
            fps: 12,
        }],
        true,
    );
    government_bullet_sprite.set_animation(0);

    let mut government_small_sprite = AnimatedSprite::new(
        17,
        16,
        &[Animation {
            name: "government_small".to_string(),
            row: 0,
            frames: 2,
            fps: 12,
        }],
        true,
    );

    root_ui().push_skin(&resources.ui_skin);
    let window_size = vec2(370.0, 320.0);

    let mut has_valid_mouse_position = false;

    let mut has_started_steering = false;

    loop {
        clear_background(BLACK);

        let screen_width = screen_width();
        let screen_height = screen_height();
        let scale_x = screen_width / base_width;
        let scale = scale_x;

        let max_goons = (base_goons as f32 * scale).floor() as usize;

        material.set_uniform("iResolution", (screen_width, screen_height));
        material.set_uniform("direction_modifier", direction_modifier);

        gl_use_material(&material);
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width, screen_height)),
                ..Default::default()
            },
        );
        gl_use_default_material();

        let mut exit_game = false;

        match game_state {
            GameState::MainMenu => {
                set_sound_volume(&resources.theme_music, 0.2);
                score = 0;
                high_score_beaten = false;
                root_ui().window(
                    hash!(),
                    vec2(
                        screen_width / 2.0 - window_size.x / 2.0,
                        screen_height / 2.0 - window_size.y / 2.0,
                    ),
                    window_size,
                    |ui| {
                        ui.label(vec2(90.0, -34.0), "Main menu");
                        if ui.button(vec2(66.0, 25.0), "Play") {
                            goons.clear();
                            bullets.clear();
                            government_bullets.clear();
                            explosions.clear();
                            milei.shape.x = screen_width / 2.0;
                            milei.shape.y = screen_height - milei.shape.size;
                            game_state = GameState::Playing;
                            has_valid_mouse_position = false;
                            has_started_steering = false;
                        }
                        if ui.button(vec2(66.0, 125.0), "Exit") {
                            exit_game = true;
                        }
                    },
                );
                draw_game_title();
                draw_score(score, high_score, high_score_beaten);
            }
            GameState::Playing => {
                set_sound_volume(&resources.theme_music, 1.0);
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }
                #[cfg(any(target_os = "ios", target_os = "android"))]
                if has_started_steering && is_mouse_button_released(MouseButton::Left) {
                    game_state = GameState::Paused;
                    next_frame().await;
                }
                #[cfg(any(target_os = "ios", target_os = "android"))]
                if !has_started_steering && is_mouse_button_pressed(MouseButton::Left) {
                    has_valid_mouse_position = true;
                    has_started_steering = true;
                }
                let delta_time = get_frame_time();
                let my_movement_speed = delta_time * MOVEMENT_SPEED;
                let star_movement_speed = delta_time * STARFIELD_SPEED;

                #[cfg(any(target_os = "ios", target_os = "android"))]
                let (mouse_x, mouse_y) = if has_valid_mouse_position {
                    mouse_position()
                } else {
                    (milei.shape.x, milei.shape.y)
                };
                #[cfg(any(target_os = "ios", target_os = "android"))]
                let dir_x = mouse_x - milei.shape.x;
                #[cfg(any(target_os = "ios", target_os = "android"))]
                let dir_y = mouse_y
                    - milei.shape.y
                    - if has_started_steering {
                        milei.size * 0.75
                    } else {
                        0.0
                    };
                #[cfg(not(any(target_os = "ios", target_os = "android")))]
                let dir_x: f32 = if is_key_down(KeyCode::Left) {
                    -MOVEMENT_SPEED
                } else if is_key_down(KeyCode::Right) {
                    MOVEMENT_SPEED
                } else {
                    0.0
                };
                #[cfg(not(any(target_os = "ios", target_os = "android")))]
                let dir_y: f32 = if is_key_down(KeyCode::Up) {
                    -MOVEMENT_SPEED
                } else if is_key_down(KeyCode::Down) {
                    MOVEMENT_SPEED
                } else {
                    0.0
                };

                ship_sprite.set_animation(0);
                if is_key_pressed(KeyCode::Left) {
                    left_direction_time = get_time();
                }
                if dir_x < 0.0 {
                    milei.shape.x -= my_movement_speed.min(dir_x.abs());
                    direction_modifier -= star_movement_speed;
                    ship_sprite.set_animation(if get_time() < left_direction_time + 0.5 {
                        1
                    } else {
                        2
                    });
                }
                if is_key_pressed(KeyCode::Right) {
                    right_direction_time = get_time();
                }
                if dir_x > 0.0 {
                    milei.shape.x += my_movement_speed.min(dir_x);
                    direction_modifier += star_movement_speed;
                    ship_sprite.set_animation(if get_time() < right_direction_time + 0.5 {
                        3
                    } else {
                        4
                    });
                }
                if dir_y > 0.0 {
                    milei.shape.y += my_movement_speed.min(dir_y);
                }
                if dir_y < 0.0 {
                    milei.shape.y -= my_movement_speed.min(dir_y.abs());
                }

                milei.shape.x = milei
                    .shape.x
                    .min(screen_width - BALL_RADIUS)
                    .max(0.0 + BALL_RADIUS);
                milei.shape.y = milei
                    .shape.y
                    .min(screen_height - BALL_RADIUS)
                    .max(0.0 + BALL_RADIUS);

                if get_time() - last_bullet_time > 1.0 / MAX_BULLETS_PER_SECOND {
                    last_bullet_time = get_time();
                    let size = 32.0;
                    let bullet_sprite_w = bullet_sprite.frame().source_rect.w;
                    let bullet_sprite_h = bullet_sprite.frame().source_rect.h;
                    let w = bullet_sprite_w * size / bullet_sprite_w;
                    let h = bullet_sprite_h * size / bullet_sprite_h;
                    bullets.push(Shape {
                        x: milei.shape.x,
                        y: milei.shape.y - 24.0,
                        w,
                        h,
                        speed: milei.shape.speed * 2.0,
                        color: GOLD,
                        size,
                        collided: false,
                    });
                    play_sound_once(&resources.sound_laser);
                }

                if goons.len() < max_goons && rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0) * scale;
                    let ship_sprite_w = government_small_sprite.frame().source_rect.w;
                    let ship_sprite_h = government_small_sprite.frame().source_rect.h;
                    let w = ship_sprite_w * size / ship_sprite_w;
                    let h = ship_sprite_h * size / ship_sprite_h;
                    goons.push(Government {
                        id: next_government_id,
                        bullet_count: 0,
                        shape: Shape {
                            size,
                            speed: rand::gen_range(50.0, 150.0),
                            x: rand::gen_range(size / 2.0, screen_width - size / 2.0),
                            y: -size,
                            w,
                            h,
                            color: *GOVERNMENT_COLORS.choose().unwrap(),
                            collided: false,
                        },
                    });
                    next_government_id += 1;
                }

                for government in &mut goons {
                    government.shape.y += government.shape.speed * delta_time;
                }
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }
                for bullet in &mut government_bullets {
                    bullet.shape.y += bullet.shape.speed * delta_time;
                }

                ship_sprite.update();
                bullet_sprite.update();
                government_small_sprite.update();

                if goons
                    .iter()
                    .any(|government| government.shape.collides_with_circle(&milei.shape))
                {
                    if score == high_score {
                        save_high_score(score);
                    }
                    game_state = GameState::GameOver;
                }

                for government in goons.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collides_with(&government.shape) {
                            bullet.collided = true;
                            government.shape.collided = true;
                            score += government.shape.size.round() as u32;
                            if score > high_score {
                                high_score_beaten = true;
                                high_score = score;
                            }
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: government.shape.size.round() as u32 * 2,
                                    texture: Some(resources.explosion_texture.clone()),
                                    ..particle_explosion()
                                }),
                                vec2(bullet.x, bullet.y),
                            ));
                            play_sound_once(&resources.sound_explosion);
                        }
                    }
                    if milei.shape.x > government.shape.x - government.shape.w / 2.0
                        && milei.shape.x < government.shape.x + government.shape.w / 2.0
                        && government.bullet_count < 1
                    {
                        let size = 16.0;
                        let government_bullet_sprite_w =
                            government_bullet_sprite.frame().source_rect.w;
                        let government_bullet_sprite_h =
                            government_bullet_sprite.frame().source_rect.h;
                        let w = government_bullet_sprite_w * size / government_bullet_sprite_w;
                        let h = government_bullet_sprite_h * size / government_bullet_sprite_h;
                        government_bullets.push(GovernmentBullet {
                            government_id: government.id,
                            shape: Shape {
                                x: government.shape.x,
                                y: government.shape.y + government.shape.size / 2.0,
                                w,
                                h,
                                speed: government.shape.speed * 3.0,
                                color: RED,
                                size,
                                collided: false,
                            },
                        });
                        government.bullet_count += 1;
                    }
                }

                for bullet in government_bullets.iter_mut() {
                    if bullet.shape.collides_with(&milei.shape) {
                        if score == high_score {
                            save_high_score(score);
                        }
                        game_state = GameState::GameOver;
                    }
                }

                government_bullets.retain(|bullet| {
                    let should_keep = bullet.shape.y < screen_height + bullet.shape.size;
                    if !should_keep {
                        if let Some(government) = goons
                            .iter_mut()
                            .find(|government| government.id == bullet.government_id)
                        {
                            government.bullet_count -= 1;
                        }
                    }
                    should_keep
                });

                goons.retain(|government| {
                    government.shape.y < screen_height + government.shape.size
                });
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
                goons.retain(|government| !government.shape.collided);
                bullets.retain(|bullet| !bullet.collided);
                explosions.retain(|(explosion, _)| explosion.config.emitting);

                draw_game_objects(
                    &goons,
                    &bullets,
                    &government_bullets,
                    &milei,
                    &mut explosions,
                    &bullet_sprite,
                    &government_bullet_sprite,
                    &ship_sprite,
                    &government_small_sprite,
                    &resources,
                );
                draw_score(score, high_score, high_score_beaten);
            }
            GameState::Paused => {
                set_sound_volume(&resources.theme_music, 0.2);
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Playing;
                }
                #[cfg(any(target_os = "ios", target_os = "android"))]
                if is_mouse_button_pressed(MouseButton::Left) {
                    game_state = GameState::Playing;
                    has_valid_mouse_position = true;
                }
                draw_game_objects(
                    &goons,
                    &bullets,
                    &government_bullets,
                    &milei,
                    &mut explosions,
                    &bullet_sprite,
                    &government_bullet_sprite,
                    &ship_sprite,
                    &government_small_sprite,
                    &resources,
                );
                draw_score(score, high_score, high_score_beaten);
                let text = "Paused";
                let text_dimensions = measure_text(text, None, 32, 1.0);
                draw_text(
                    text,
                    screen_width / 2.0 - text_dimensions.width / 2.0,
                    screen_height / 2.0,
                    32.0,
                    GOLD,
                );
                draw_game_title();
            }
            GameState::GameOver => {
                set_sound_volume(&resources.theme_music, 0.2);
                if is_key_pressed(KeyCode::Escape) || is_mouse_button_pressed(MouseButton::Left) {
                    game_state = GameState::MainMenu;
                }
                draw_game_objects(
                    &goons,
                    &bullets,
                    &government_bullets,
                    &milei,
                    &mut explosions,
                    &bullet_sprite,
                    &government_bullet_sprite,
                    &ship_sprite,
                    &government_small_sprite,
                    &resources,
                );
                draw_score(score, high_score, high_score_beaten);
                let game_over_text = "GAME OVER!";
                let text_dimensions = measure_text(game_over_text, None, 32, 1.0);

                let text_x = (screen_width - text_dimensions.width) / 2.0;
                let text_y =
                    screen_height / 2.0 - text_dimensions.offset_y + text_dimensions.height;

                draw_text(game_over_text, text_x, text_y, 32.0, GOLD);
                draw_game_title();
            }
        }
        if exit_game {
            return Ok(());
        }
        next_frame().await
    }
}
