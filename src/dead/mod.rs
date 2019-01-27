#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, Settings, Sounds, WorldData};

pub struct DeadPhase {
    sounds: Sounds,
    first_update: bool,
    want_restart: bool,
}

impl DeadPhase {
    pub fn new(ctx: &mut Context) -> Self {
        DeadPhase {
            sounds: Sounds::load(ctx),
            first_update: true,
            want_restart: false,
        }
    }
    pub fn update(&mut self, _settings: &Settings, _data: &mut WorldData, _ctx: &mut Context) {
        if self.first_update {
            self.sounds.play_death();
            self.first_update = false;
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
            self.want_restart = true;
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
