#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, MusicTrack, Settings, WorldData};

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

    pub fn update(&mut self, settings: &Settings, _data: &mut WorldData, _ctx: &mut Context) {
        if self.first_update {
            println!("STATE: Menu");
            self.first_update = false;
        }

        if settings.music && !self.music_track.playing() {
            self.music_track.play();
        }
    }

    pub fn draw(&mut self, _settings: &Settings, _data: &mut WorldData, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.7, 0.3, 1.0].into());
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
