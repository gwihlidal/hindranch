#![allow(unused_imports)]

use crate::{
    graphics, Color, Context, KeyCode, MainState, MouseButton, MusicTrack, PlayerInput, Point2,
    Settings, Vector2, WorldData,
};

pub struct MenuPhase {
    pub first_update: bool,
    pub start_game: bool,
    pub music_track: MusicTrack,
}

impl MenuPhase {
    pub fn new(ctx: &mut Context) -> Self {
        MenuPhase {
            first_update: true,
            start_game: false,
            music_track: MusicTrack::new("twisted", ctx),
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, _ctx: &mut Context) {
        if self.first_update {
            println!("STATE: Menu");
            data.player.input = PlayerInput::default();
            self.first_update = false;
        }

        if settings.music && !self.music_track.playing() {
            self.music_track.play();
        }
    }

    pub fn draw(&mut self, _settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        let window_size = graphics::drawable_size(ctx);

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let img_width = data.splash.width() as f32;
        let img_height = data.splash.height() as f32;
        let scale_x = window_size.0 as f32 / img_width;
        let scale_y = window_size.1 as f32 / img_height;

        graphics::draw(
            ctx,
            &data.splash,
            graphics::DrawParam::new().scale(Vector2::new(scale_x, scale_y)),
        )
        .unwrap();

        let text = graphics::Text::new(("Press Space To Begin", data.font, 96.0));

        let text_width = text.width(ctx) as f32;
        let text_height = text.height(ctx) as f32;

        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Point2::new(
                    ((window_size.0 as f32 / 2.0) - (text_width / 2.0)) + 4.0,
                    (window_size.1 as f32 - text_height - 20.0) + 4.0,
                ))
                .color(Color::from((0, 0, 0, 255))),
        )
        .unwrap();

        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Point2::new(
                    (window_size.0 as f32 / 2.0) - (text_width / 2.0),
                    window_size.1 as f32 - text_height - 20.0,
                ))
                .color(Color::from((255, 255, 255, 255))),
        )
        .unwrap();
    }

    pub fn handle_key(
        &mut self,
        _settings: &Settings,
        _data: &mut WorldData,
        _ctx: &mut Context,
        key_code: KeyCode,
        value: bool,
    ) {
        if key_code == KeyCode::Space && value {
            self.start_game = true;
        }
    }

    pub fn mouse_motion_event(
        &mut self,
        _data: &mut WorldData,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
        _xrel: f32,
        _yrel: f32,
    ) {
        //
    }

    pub fn mouse_button_down_event(
        &mut self,
        _data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        //
    }

    pub fn mouse_button_up_event(
        &mut self,
        _data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        //
    }
}
