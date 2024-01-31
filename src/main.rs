mod ios;

use std::f32::consts::PI;

use macroquad::audio::{load_sound, play_sound, set_sound_volume, PlaySoundParams, Sound};

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use macroquad::experimental::coroutines::start_coroutine;

mod simple_logger;

const MOVEMENT_SPEED: f32 = 1.5;
const BOARD_TILES_X: usize = 75;

const BOARD_LEFT: f32 = 0.0;
const BOARD_RIGHT: f32 = 1.0;
const BOARD_TOP: f32 = 0.0;
const BOARD_BOTTOM: f32 = 1.0;

const NUM_BLACK_BALLS: usize = 200;
const NUM_GOLD_BALLS: usize = 200;

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
        let tiles: Vec<Vec<bool>> = (0..BOARD_TILES_X - 1)
            .map(|i| {
                (0..BOARD_TILES_X)
                    .map(|j| j <= BOARD_TILES_X - 2 - i)
                    .collect()
            })
            .collect();

        let board = Self {
            tiles,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };

        board
    }

    fn tile_width(&self) -> f32 {
        self.width / BOARD_TILES_X as f32
    }

    fn tile_at(&self, x: f32, y: f32) -> bool {
        let tile_x = (x / self.tile_width()).floor() as usize;
        let tile_y = (y / self.tile_width()).floor() as usize;

        self.tiles[tile_y][tile_x]
    }

    pub fn set_tile_at(&mut self, x: f32, y: f32, v: bool) {
        let tile_x = (x / self.tile_width()).floor() as usize;
        let tile_y = (y / self.tile_width()).floor() as usize;
        self.tiles[tile_y][tile_x] = v;
    }

    fn update_size_and_position(&mut self) {
        self.width = f32::min(screen_width(), screen_height() - 40.0);
        self.height = self.tile_width() * (BOARD_TILES_X - 1) as f32;
        self.x = screen_width() / 2.0 - self.width / 2.0;
        self.y = screen_height() / 2.0 - self.height / 2.0;
    }
}

pub fn draw_circle_100(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 100, r, 0., color);
}

fn draw_board(board: &Board, black_balls: &[Ball], gold_balls: &[Ball]) {
    let tile_size = board.tile_width();

    for (xi, row) in board.tiles.iter().enumerate() {
        for (yi, &tile) in row.iter().enumerate() {
            let x = board.x + yi as f32 * tile_size;
            let y = board.y + xi as f32 * tile_size;

            if tile {
                draw_rectangle(x, y, tile_size, tile_size, GOLD);
            } else {
                draw_rectangle(x, y, tile_size, tile_size, BLACK);
            }
        }
    }
    for ball in black_balls.iter() {
        draw_circle_100(
            ball.x * board.width + board.x,
            ball.y * board.height + board.y,
            ball.size / 2.0,
            ball.color,
        );
    }
    for ball in gold_balls.iter() {
        draw_circle_100(
            ball.x * board.width + board.x,
            ball.y * board.height + board.y,
            ball.size / 2.0,
            ball.color,
        );
    }
}

enum GameState {
    Starting,
    Playing,
}

fn move_ball(
    board: &mut Board,
    ball: &mut Ball,
    tile_sound: &Sound,
    wall_sound: &Sound,
    bounce_volume: f32,
) {
    let frame_time = get_frame_time().min(0.0035);
    let movement = MOVEMENT_SPEED * ball.speed * frame_time;
    let p_radius = ball.size / 2.0;
    let radius = p_radius / board.width;

    let mut new_x = ball.x + movement * ball.direction.0;
    let mut new_y = ball.y + movement * ball.direction.1;

    let new_px = new_x * board.width;
    let new_py = new_y * board.height;

    let left_x = if new_px > p_radius {
        new_px - p_radius
    } else {
        0.0
    };
    let right_x = if new_px + p_radius < board.width {
        new_px + p_radius
    } else {
        board.width - 1.0
    };
    let top_y = if new_py > p_radius {
        new_py - p_radius
    } else {
        0.0
    };
    let bottom_y = if new_py + p_radius < board.height {
        new_py + p_radius
    } else {
        board.height - 1.0
    };

    if ball.bounce_on == board.tile_at(left_x, new_py) {
        ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(left_x, new_py, !ball.bounce_on);
        play_sound(
            tile_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    } else if ball.bounce_on == board.tile_at(right_x, new_py) {
        ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(right_x, new_py, !ball.bounce_on);
        play_sound(
            tile_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    }

    if ball.bounce_on == board.tile_at(new_px, top_y) {
        ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(new_px, top_y, !ball.bounce_on);
        play_sound(
            tile_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    } else if ball.bounce_on == board.tile_at(new_px, bottom_y) {
        ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(new_px, bottom_y, !ball.bounce_on);
        play_sound(
            tile_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    }

    if (new_x - radius) < BOARD_LEFT {
        new_x = BOARD_LEFT + radius;
        ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        play_sound(
            wall_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    } else if (new_x + radius) > BOARD_RIGHT {
        new_x = BOARD_RIGHT - radius;
        ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        play_sound(
            wall_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    }

    if (new_y - radius) < BOARD_TOP {
        new_y = BOARD_TOP + radius;
        ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        play_sound(
            wall_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    } else if (new_y + radius) > BOARD_BOTTOM {
        new_y = BOARD_BOTTOM - radius;
        ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        play_sound(
            wall_sound,
            PlaySoundParams {
                volume: bounce_volume,
                looped: false,
            },
        );
    }

    ball.x = new_x;
    ball.y = new_y;
}

fn draw_toggle_button(position: Vec2, text: &str, toggle: &mut bool) -> bool {
    // Draw the button text
    let font_size = 21;
    let text_dimensions = measure_text(&text, None, font_size, 1.0);
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

    let mut black_balls: Vec<Ball> = (0..NUM_BLACK_BALLS)
        .map(|_| {
            Ball::new(
                BLACK,
                false,
                0.25 + rand::gen_range(-0.1, 0.1),
                0.25 + rand::gen_range(-0.1, 0.1),
            )
        })
        .collect();
    let mut gold_balls: Vec<Ball> = (0..NUM_GOLD_BALLS)
        .map(|_| {
            Ball::new(
                GOLD,
                true,
                0.75 + rand::gen_range(-0.1, 0.1),
                0.75 + rand::gen_range(-0.1, 0.1),
            )
        })
        .collect();

    let mut board = Board::new();

    let mut game_state = GameState::Starting;

    let mut music_on = true;
    let mut sound_on = NUM_BLACK_BALLS + NUM_GOLD_BALLS < 10;

    let mut started_music = false;
    let mut started_lions = false;
    let mut lions_start_time = None;

    loop {
        clear_background(Color::new(116.0 / 255.0, 172.0 / 255.0, 223.0 / 255.0, 1.0));

        board.update_size_and_position();

        for ball in black_balls.iter_mut() {
            ball.size = board.tile_width() * 1.0;
        }
        for ball in gold_balls.iter_mut() {
            ball.size = board.tile_width() * 1.0;
        }

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
                draw_board(&board, &black_balls, &gold_balls);
                draw_game_title(&board);
            }
            GameState::Playing => {
                for ball in gold_balls.iter_mut() {
                    move_ball(
                        &mut board,
                        ball,
                        &resources.sound_gold,
                        &resources.sound_wall,
                        if sound_on { 0.05 } else { 0.0 },
                    );
                }
                for ball in black_balls.iter_mut() {
                    move_ball(
                        &mut board,
                        ball,
                        &resources.sound_black,
                        &resources.sound_wall,
                        if sound_on { 0.05 } else { 0.0 },
                    );
                }
                draw_board(&board, &black_balls, &gold_balls);
            }
        }

        next_frame().await
    }
}
