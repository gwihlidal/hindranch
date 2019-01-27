#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, MusicTrack, Settings, WorldData};

pub struct MenuPhase {
    start_game: bool,
    music_track: MusicTrack,
}

impl MenuPhase {
    pub fn new(ctx: &mut Context) -> Self {
        MenuPhase {
            start_game: false,
            music_track: MusicTrack::new("cantina", ctx),
        }
    }

    pub fn update(&mut self, _settings: &Settings, _data: &mut WorldData, _ctx: &mut Context) {
        if !self.music_track.playing() {
            self.music_track.play();
        }
    }

    pub fn draw(&mut self, _settings: &Settings, _data: &mut WorldData, _ctx: &mut Context) {
        //
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
