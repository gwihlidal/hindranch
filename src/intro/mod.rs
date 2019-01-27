#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, WorldData};

pub struct IntroPhase {}

impl IntroPhase {
    pub fn update(&mut self, _ctx: &mut Context) {
        //
    }

    pub fn draw(&mut self, _ctx: &mut Context) {
        //
    }

    pub fn handle_key(&mut self, _key_code: KeyCode, _value: bool) {}

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
