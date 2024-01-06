use macroquad::experimental::collections::storage;
use macroquad::experimental::coroutines::start_coroutine;
use macroquad::prelude::*;
use macroquad::audio::{load_sound, Sound};
use macroquad::ui::{root_ui, Skin};

pub struct Resources {
  pub milei_texture: Texture2D,
  pub bullet_texture: Texture2D,
  pub explosion_texture: Texture2D,
  pub goon_small_texture: Texture2D,
  pub theme_music: Sound,
  pub sound_explosion: Sound,
  pub sound_laser: Sound,
  pub ui_skin: Skin,
}

impl Resources {
  async fn new() -> Result<Resources, macroquad::Error> {
      let ship_texture: Texture2D = load_texture("milei.png").await?;
      ship_texture.set_filter(FilterMode::Nearest);
      let bullet_texture: Texture2D = load_texture("laser-bolts.png").await?;
      bullet_texture.set_filter(FilterMode::Nearest);
      let explosion_texture: Texture2D = load_texture("explosion.png").await?;
      explosion_texture.set_filter(FilterMode::Nearest);
      let goon_small_texture: Texture2D = load_texture("goon-small.png").await?;
      goon_small_texture.set_filter(FilterMode::Nearest);

      let theme_music = load_sound("8bit-spaceshooter.ogg").await?;
      let sound_explosion = load_sound("explosion.wav").await?;
      let sound_laser = load_sound("laser.wav").await?;

      let window_background = load_image("window_background.png").await?;
      let button_background = load_image("button_background.png").await?;
      let button_clicked_background = load_image("button_clicked_background.png").await?;
      let font = load_file("atari_games.ttf").await?;

      let window_style = root_ui()
          .style_builder()
          .background(window_background)
          .background_margin(RectOffset::new(32.0, 76.0, 44.0, 20.0))
          .margin(RectOffset::new(0.0, -40.0, 0.0, 0.0))
          .build();
      let button_style = root_ui()
          .style_builder()
          .background(button_background)
          .background_clicked(button_clicked_background)
          .background_margin(RectOffset::new(16.0, 16.0, 16.0, 16.0))
          .margin(RectOffset::new(16.0, 0.0, -8.0, -8.0))
          .font(&font)?
          .text_color(WHITE)
          .font_size(64)
          .build();
      let label_style = root_ui()
          .style_builder()
          .font(&font)?
          .text_color(WHITE)
          .font_size(28)
          .build();
      let ui_skin = Skin {
          window_style,
          button_style,
          label_style,
          ..root_ui().default_skin()
      };

      Ok(Resources {
          milei_texture: ship_texture,
          bullet_texture,
          explosion_texture,
          goon_small_texture,
          theme_music,
          sound_explosion,
          sound_laser,
          ui_skin,
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