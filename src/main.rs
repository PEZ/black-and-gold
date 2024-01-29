mod ios;

use std::f32::consts::PI;

use macroquad::audio::{
    load_sound, play_sound, play_sound_once, set_sound_volume, PlaySoundParams, Sound,
};

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use macroquad::experimental::coroutines::start_coroutine;

mod simple_logger;

const GAME_TITLE: &str = "Black & Gold";
const MOVEMENT_SPEED: f32 = 1.1;
const BOARD_TILES_X: usize = 25;

const BOARD_LEFT: f32 = 0.0;
const BOARD_RIGHT: f32 = 1.0;
const BOARD_TOP: f32 = 0.0;
const BOARD_BOTTOM: f32 = 1.0;

struct Resources {
    pub theme_music: Sound,
    pub sound_explosion: Sound,
    pub sound_laser: Sound,
}

impl Resources {
    async fn new() -> Result<Resources, macroquad::Error> {
        let theme_music = load_sound("8bit-spaceshooter.wav").await?;
        let sound_explosion = load_sound("explosion.wav").await?;
        let sound_laser = load_sound("laser.wav").await?;

        Ok(Resources {
            theme_music,
            sound_explosion,
            sound_laser,
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

fn draw_game_title() {
    let text = GAME_TITLE;
    let font_size = 48;
    let text_dimensions = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - text_dimensions.width / 2.0,
        screen_height() / 4.0,
        font_size as f32,
        oscillating_alpha(GOLD, 3.0),
    );
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
        let direction_x = if rand::gen_range(0, 2) == 0 { -1.0 } else { 1.0 };
        let direction_y = if rand::gen_range(0, 2) == 0 { -1.0 } else { 1.0 };
        Self {
            size: 10.0,
            direction: (direction_x, direction_y),
            speed: 5.0,
            x,
            y,
            color,
            bounce_on,
        }
    }

    pub fn collides_with_circle(&self, circle: &Ball) -> bool {
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
            w: self.size,
            h: self.size,
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
        self.width = f32::min(screen_width(), screen_height());
        self.height = self.tile_width() * (BOARD_TILES_X - 1) as f32;
        self.x = screen_width() / 2.0 - self.width / 2.0;
        self.y = screen_height() / 2.0 - self.height / 2.0;
    }
}

pub fn draw_circle_100(x: f32, y: f32, r: f32, color: Color) {
    draw_poly(x, y, 100, r, 0., color);
}

fn draw_board(board: &Board, black: &Ball, gold: &Ball) {
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
    draw_circle_100(
        gold.x * board.width + board.x,
        gold.y * board.height + board.y,
        gold.size / 2.0,
        gold.color,
    );
    draw_circle_100(
        black.x * board.width + board.x,
        black.y * board.height + board.y,
        black.size / 2.0,
        black.color,
    );
}

enum GameState {
    Starting,
    Playing,
}

fn move_ball(board: &mut Board, ball: &mut Ball) {
    let frame_time = get_frame_time().min(0.005);
    let movement = MOVEMENT_SPEED * frame_time;
    let p_radius =  ball.size / 2.0; 
    let radius = p_radius / board.width;

    let mut new_x = ball.x + movement * ball.direction.0;
    let mut new_y = ball.y + movement * ball.direction.1;
    
    let new_px = (new_x * board.width);
    let new_py = (new_y * board.height);
    
    let left_x = if new_px > p_radius { new_px - p_radius } else { 0.0 };
    let right_x = if new_px + p_radius < board.width { new_px + p_radius } else { board.width - 1.0 };
    let top_y = if new_py > p_radius { new_py - p_radius } else { 0.0 };
    let bottom_y = if new_py + p_radius < board.height { new_py + p_radius } else { board.height - 1.0 };

    if ball.bounce_on == board.tile_at(left_x, new_py) {
        ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(left_x, new_py, !ball.bounce_on);
    } else if ball.bounce_on == board.tile_at(right_x, new_py) {
        ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(right_x, new_py, !ball.bounce_on);
    }
    
    if ball.bounce_on == board.tile_at(new_px, top_y) {
        ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(new_px, top_y, !ball.bounce_on);
    } else if ball.bounce_on == board.tile_at(new_px, bottom_y) {
        ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
        board.set_tile_at(new_px, bottom_y, !ball.bounce_on);
    }

    if (new_x - radius) < BOARD_LEFT {
        new_x = BOARD_LEFT + radius;
        ball.direction.0 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
    } else if (new_x + radius) > BOARD_RIGHT {
        new_x = BOARD_RIGHT - radius;
        ball.direction.0 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
    }

    if (new_y - radius) < BOARD_TOP {
        new_y = BOARD_TOP + radius;
        ball.direction.1 = 1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
    } else if (new_y + radius) > BOARD_BOTTOM {
        new_y = BOARD_BOTTOM - radius;
        ball.direction.1 = -1.0 * (1.0 + rand::gen_range(-0.1, 0.1));
    }

    ball.x = new_x;
    ball.y = new_y;
}

#[macroquad::main("Black and Gold", Conf {
    sample_count: 4,
    ..Default::default()
})]
async fn main() -> Result<(), macroquad::Error> {
    rand::srand(miniquad::date::now() as u64);

    simple_logger::setup_logger();

    log::info!("¡Viva la libertad, Carajo!");

    set_pc_assets_folder("assets");

    Resources::load().await?;
    let resources = storage::get::<Resources>();

    play_sound(
        &resources.theme_music,
        PlaySoundParams {
            looped: true,
            volume: 1.0,
        },
    );

    let mut gold = Ball::new(GOLD, true, 0.75, 0.75);
    let mut black = Ball::new(BLACK, false, 0.25, 0.25);

    let mut board = Board::new();

    let mut game_state = GameState::Starting;
    let start_time = get_time();

    loop {
        clear_background(Color::new(116.0 / 255.0, 172.0 / 255.0, 223.0 / 255.0, 1.0));

        board.update_size_and_position();

        gold.size = board.tile_width();
        black.size = board.tile_width();

        if get_time() - start_time > 1.0 {
            game_state = GameState::Playing;
        }

        match game_state {
            GameState::Starting => {
                draw_game_title();
            }
            GameState::Playing => {
                move_ball(&mut board, &mut black);
                move_ball(&mut board, &mut gold);
                draw_board(&board, &black, &gold);
            }
        }

        next_frame().await
    }
}
