mod ios;

use std::f32::consts::PI;

use macroquad::audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams, Sound};

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use macroquad::experimental::coroutines::start_coroutine;

use itertools::Itertools;
use macroquad::rand::ChooseRandom;

mod simple_logger;

mod config;

struct Resources {
    theme_music: Sound,
    lions: Sound,
    sound_wall: Sound,
    sound_gold: Sound,
    sound_black: Sound,
}

impl Resources {
    async fn new() -> Result<Resources, macroquad::Error> {
        let theme_music = load_sound("moza-unfinished.wav").await?;
        let lions = load_sound("lions.wav").await?;
        let sound_wall = load_sound("456563__bumpelsnake__bounce1.wav").await?;
        let sound_gold = load_sound("456564__bumpelsnake__bell2.wav").await?;
        let sound_black = load_sound("456565__bumpelsnake__bell1.wav").await?;

        Ok(Resources {
            theme_music,
            lions,
            sound_wall,
            sound_gold,
            sound_black,
        })
    }

    pub async fn load() -> Result<(), macroquad::Error> {
        let resources_loading = start_coroutine(async move {
            let resources = Resources::new().await.unwrap();
            storage::store(resources);
        });

        while !resources_loading.is_done() {
            clear_background(BLACK);
            let text = format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.) as usize) % 4)
            );
            draw_text(
                &text,
                screen_width() / 2. - 160.,
                screen_height() / 2.,
                40.,
                WHITE,
            );
            next_frame().await;
        }

        Ok(())
    }
}

fn oscillating_alpha(base_color: Color, cycles_per_second: f32) -> Color {
    let alpha = 0.5 * (1.0 + f32::sin(cycles_per_second * get_time() as f32 * PI / 2.0));
    Color::new(base_color.r, base_color.g, base_color.b, alpha)
}

fn draw_game_title(board: &Board) {
    {
        let text = "Black";
        let font_size = 48;
        let text_dimensions = measure_text(text, None, font_size, 1.0);
        draw_text(
            text,
            board.x + 25.0,
            board.y + text_dimensions.height + 25.0,
            font_size as f32,
            BLACK,
        );
    }
    {
        let text = "Gold";
        let font_size = 48;
        let text_dimensions = measure_text(text, None, font_size, 1.0);
        draw_text(
            text,
            board.x + board.width - text_dimensions.width - 25.0,
            board.y + board.height - 25.0,
            font_size as f32,
            GOLD,
        );
    }
    {
        let text = "Click/tap to start";
        let font_size = 24;
        let text_dimensions = measure_text(text, None, font_size, 1.0);
        draw_text(
            text,
            screen_width() / 2.0 - text_dimensions.width / 2.0,
            board.y + 50.0,
            font_size as f32,
            oscillating_alpha(BLACK, 3.0),
        );
    }
}

fn draw_scores(board: &Board) {
    let gold_score = board.tiles.iter().flatten().filter(|&&t| t).count();
    let black_score = board.tiles.iter().flatten().filter(|&&t| !t).count();
    {
        let text = format!("Gold: {}", gold_score);
        let font_size = 18;
        draw_text(&text, board.x + 4.0, board.y - 4.0, font_size as f32, BLACK);
    }
    {
        let text = format!("Black: {}", black_score);
        let font_size = 18;
        let text_dimensions = measure_text(&text, None, font_size, 1.0);
        draw_text(
            &text,
            board.x + board.width - text_dimensions.width - 4.0,
            board.y - 4.0,
            font_size as f32,
            BLACK,
        );
    }
}

fn draw_fps(board: &Board, fps: f32) {
    {
        let text = format!("fps: {}", fps.round() as i32);
        let font_size = 18;
        let text_dimensions = measure_text(&text, None, font_size, 1.0);
        draw_text(
            &text,
            board.x + board.width / 2.0 - text_dimensions.width / 2.0,
            board.y - 4.0,
            font_size as f32,
            BLACK,
        );
    }
}

struct Ball {
    size: f32,
    direction: (f32, f32),
    speed: f32,
    x: f32,
    y: f32,
    color: Color,
    bounce_on: bool,
}

impl Ball {
    fn new(color: Color, bounce_on: bool, x: f32, y: f32) -> Self {
        let direction_x = if rand::gen_range(0, 2) == 0 {
            -1.0
        } else {
            1.0
        };
        let direction_y = if rand::gen_range(0, 2) == 0 {
            -1.0
        } else {
            1.0
        };
        Self {
            size: 10.0,
            direction: (direction_x, direction_y),
            speed: rand::gen_range(0.75, 1.0),
            x,
            y,
            color,
            bounce_on,
        }
    }
}

struct Board {
    tiles: Vec<Vec<bool>>,
    width: f32,
    height: f32,
    x: f32,
    y: f32,
}

impl Board {
    fn new() -> Self {
        let tiles: Vec<Vec<bool>> = (0..config::BOARD_TILES_X - 1)
            .map(|i| {
                (0..config::BOARD_TILES_X)
                    .map(|j| j <= config::BOARD_TILES_X - 2 - i)
                    .collect()
            })
            .collect();

        Self {
            tiles,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    fn tile_width(&self) -> f32 {
        self.width / config::BOARD_TILES_X as f32
    }

    fn tile_at(&self, x: f32, y: f32, tile_width: f32) -> (usize, usize) {
        let tile_x = (x / tile_width).floor() as usize;
        let tile_y = (y / tile_width).floor() as usize;
        (tile_y, tile_x)
    }

    fn update_size_and_position(&mut self) {
        self.width = f32::min(screen_width(), screen_height() - 40.0);
        self.height = self.tile_width() * (config::BOARD_TILES_X - 1) as f32;
        self.x = screen_width() / 2.0 - self.width / 2.0;
        self.y = screen_height() / 2.0 - self.height / 2.0;
    }
}

pub fn draw_circle_100(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 100, r, 0., color);
}

fn draw_board(board: &Board, balls: &mut [Ball]) {
    let tile_size = board.tile_width();
    draw_rectangle(board.x, board.y, board.width, board.height, BLACK);

    for (xi, row) in board.tiles.iter().enumerate() {
        for (yi, &tile) in row.iter().enumerate() {
            let x = board.x + yi as f32 * tile_size;
            let y = board.y + xi as f32 * tile_size;

            if tile {
                draw_rectangle(x, y, tile_size, tile_size, GOLD);
            }
            // else {
            //     draw_rectangle(x, y, tile_size, tile_size, BLACK);
            // }
        }
    }
    for ball in balls.iter() {
        let half_size = ball.size / 2.0;
        draw_rectangle(
            ball.x * board.width + board.x - half_size,
            ball.y * board.height + board.y - half_size,
            ball.size,
            ball.size,
            ball.color,
        );
    }
}

enum GameState {
    Starting,
    Playing,
}

fn move_ball(board: &mut Board, ball: &mut Ball) -> (bool, bool) {
    let frame_time = get_frame_time();
    let movement = (config::MAX_SPEED * ball.speed * frame_time)
        .min(1.0 / (config::BOARD_TILES_X as f32 * 2.1));
    let p_radius = ball.size / 2.0;
    let radius = p_radius / board.width;

    let mut new_x = ball.x + movement * ball.direction.0;
    let mut new_y = ball.y + movement * ball.direction.1;

    let new_px = (new_x * board.width).min(board.width - 1.0);
    let new_py = (new_y * board.height).min(board.height - 1.0);

    let left_x = (new_px - p_radius).max(0.0);
    let right_x = (new_px + p_radius).min(board.width - 1.0);
    let top_y = (new_py - p_radius).max(0.0);
    let bottom_y = (new_py + p_radius).min(board.height - 1.0);

    let mut directions = vec![(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)];
    directions.shuffle();

    let mut hit_wall = false;
    let mut hit_tile = false;

    let tile_width = board.tile_width();

    for (dx, dy) in &directions {
        if ball.direction.1 * dy < 0.0 {
            if (new_y - radius) < config::BOARD_TOP {
                new_y = config::BOARD_TOP + radius;
                ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                hit_wall = true;
            } else {
                let (tile_x, tile_y) = board.tile_at(new_px, top_y, tile_width);
                if ball.bounce_on == board.tiles[tile_x][tile_y] {
                    ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                    board.tiles[tile_x][tile_y] = !ball.bounce_on;
                    hit_tile = true;
                }
            }
        }

        if ball.direction.0 * dx < 0.0 {
            if (new_x - radius) < config::BOARD_LEFT {
                new_x = config::BOARD_LEFT + radius;
                ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                hit_wall = true;
            } else {
                let (tile_x, tile_y) = board.tile_at(left_x, new_py, tile_width);
                if ball.bounce_on == board.tiles[tile_x][tile_y] {
                    ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                    board.tiles[tile_x][tile_y] = !ball.bounce_on;
                    hit_tile = true;
                }
            }
        }

        if ball.direction.1 * dy > 0.0 {
            if (new_y + radius) > config::BOARD_BOTTOM {
                new_y = config::BOARD_BOTTOM - radius;
                ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                hit_wall = true;
            } else {
                let (tile_x, tile_y) = board.tile_at(new_px, bottom_y, tile_width);
                if ball.bounce_on == board.tiles[tile_x][tile_y] {
                    ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                    board.tiles[tile_x][tile_y] = !ball.bounce_on;
                    hit_tile = true;
                }
            }
        }

        if ball.direction.0 * dx > 0.0 {
            if (new_x + radius) > config::BOARD_RIGHT {
                new_x = config::BOARD_RIGHT - radius;
                ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                hit_wall = true;
            } else {
                let (tile_x, tile_y) = board.tile_at(right_x, new_py, tile_width);
                if ball.bounce_on == board.tiles[tile_x][tile_y] {
                    ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
                    board.tiles[tile_x][tile_y] = !ball.bounce_on;
                    hit_tile = true;
                }
            }
        }
    }

    ball.x = new_x;
    ball.y = new_y;

    (hit_wall, hit_tile)
}

fn draw_toggle_button(position: Vec2, text: &str, toggle: &mut bool) -> bool {
    // Draw the button text
    let font_size = 21;
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    let hitbox = Rect::new(
        position.x - 2.0,
        position.y - text_dimensions.height - 2.0,
        text_dimensions.width + 4.0,
        text_dimensions.height + 4.0,
    );
    draw_rectangle(hitbox.x, hitbox.y, hitbox.w, hitbox.h, BLACK);
    draw_text(text, position.x, position.y, font_size as f32, GOLD);
    let (mouse_x, mouse_y) = mouse_position();
    if is_mouse_button_pressed(MouseButton::Left)
        && mouse_x >= hitbox.x
        && mouse_x <= hitbox.x + hitbox.w
        && mouse_y >= hitbox.y
        && mouse_y <= hitbox.y + hitbox.h
    {
        *toggle = !*toggle;
        return true;
    }

    false
}
#[macroquad::main("Black and Gold", Conf {
    // sample_count: 4,
    ..Default::default()
})]
async fn main() -> Result<(), macroquad::Error> {
    rand::srand(miniquad::date::now() as u64);

    simple_logger::setup_logger();

    log::info!("¡Viva la libertad, Carajo!");

    set_pc_assets_folder("assets");

    Resources::load().await?;
    let resources = storage::get::<Resources>();

    let black_balls: Vec<Ball> = (0..config::NUM_BLACK_BALLS)
        .map(|_| {
            Ball::new(
                BLACK,
                false,
                0.25 + rand::gen_range(-1.0 * config::START_FIELD_SIZE, config::START_FIELD_SIZE),
                0.25 + rand::gen_range(-1.0 * config::START_FIELD_SIZE, config::START_FIELD_SIZE),
            )
        })
        .collect();
    let gold_balls: Vec<Ball> = (0..config::NUM_GOLD_BALLS)
        .map(|_| {
            Ball::new(
                GOLD,
                true,
                0.75 + rand::gen_range(-1.0 * config::START_FIELD_SIZE, config::START_FIELD_SIZE),
                0.75 + rand::gen_range(-1.0 * config::START_FIELD_SIZE, config::START_FIELD_SIZE),
            )
        })
        .collect();

    let mut balls = gold_balls
        .into_iter()
        .interleave(black_balls.into_iter())
        .collect::<Vec<_>>();

    let mut board = Board::new();

    let mut game_state = GameState::Starting;

    let mut music_on = true;
    let mut sound_on = config::NUM_BLACK_BALLS + config::NUM_GOLD_BALLS < 10;

    let mut started_music = false;
    let mut started_lions = false;
    let mut lions_start_time = None;

    let mut last_fps = get_fps() as f32;
    let smoothing_factor = 2.0 / (120.0 + 1.0);

    loop {
        clear_background(Color::new(116.0 / 255.0, 172.0 / 255.0, 223.0 / 255.0, 1.0));

        board.update_size_and_position();

        for ball in balls.iter_mut() {
            ball.size = board.tile_width() * 1.0;
        }

        last_fps = (smoothing_factor * (get_fps() as f32 - last_fps)) + last_fps;
        draw_fps(&board, last_fps);

        draw_scores(&board);

        if started_music {
            draw_toggle_button(
                Vec2::new(
                    board.x + board.width / 2.0 - 100.0,
                    board.y + board.height + 16.0,
                ),
                &format!("Music: {}", if music_on { "On" } else { "Off" }),
                &mut music_on,
            );
        }

        draw_toggle_button(
            Vec2::new(board.x + board.width / 2.0, board.y + board.height + 16.0),
            &format!("Sound Fx: {}", if sound_on { "On" } else { "Off" }),
            &mut sound_on,
        );

        if music_on {
            set_sound_volume(&resources.theme_music, 1.0);
        } else {
            set_sound_volume(&resources.theme_music, 0.0);
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if !started_lions {
                started_lions = true;
                lions_start_time = Some(get_time());
                play_sound(
                    &resources.lions,
                    PlaySoundParams {
                        looped: false,
                        volume: 1.0,
                    },
                );
            }
            game_state = GameState::Playing;
        }
        if !started_music {
            if let Some(start_time) = lions_start_time {
                if get_time() - start_time >= 15.0 {
                    started_music = true;
                    play_sound(
                        &resources.theme_music,
                        PlaySoundParams {
                            looped: true,
                            volume: 1.0,
                        },
                    );
                }
            }
        }
        match game_state {
            GameState::Starting => {
                draw_board(&board, &mut balls[..]);
                draw_game_title(&board);
            }
            GameState::Playing => {
                let (mut wall_bounce, mut black_tile_bounce, mut gold_tile_bounce) =
                    (false, false, false);
                for ball in balls.iter_mut() {
                    let (hit_wall, hit_tile) = move_ball(&mut board, ball);
                    wall_bounce = wall_bounce || hit_wall;
                    black_tile_bounce = black_tile_bounce || hit_tile && !ball.bounce_on;
                    gold_tile_bounce = gold_tile_bounce || hit_tile && ball.bounce_on;
                }
                if wall_bounce && sound_on {
                    play_sound(
                        &resources.sound_wall,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.2,
                        },
                    );
                }
                if black_tile_bounce && sound_on {
                    play_sound(
                        &resources.sound_black,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.2,
                        },
                    );
                }
                if gold_tile_bounce && sound_on {
                    play_sound(
                        &resources.sound_gold,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.2,
                        },
                    );
                }
                draw_board(&board, &mut balls[..]);
            }
        }

        next_frame().await
    }
}
