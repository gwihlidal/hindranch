#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, RoundData, Settings, WorldData};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PreparePhase {
    pub first_update: bool,
    pub round_index: u32,
    pub last_round: bool,
    pub begin_round: bool,
    pub round_data: Rc<RefCell<RoundData>>,
}

impl PreparePhase {
    pub fn new(
        _ctx: &mut Context,
        round_index: u32,
        last_round: bool,
        round_data: Rc<RefCell<RoundData>>,
    ) -> Self {
        PreparePhase {
            first_update: true,
            round_index,
            last_round,
            begin_round: false,
            round_data,
        }
    }

    pub fn update(&mut self, _settings: &Settings, _data: &mut WorldData, _ctx: &mut Context) {
        if self.first_update {
            println!(
                "STATE: Prepare - round_index: {}, last_round: {}",
                self.round_index, self.last_round
            );
            self.first_update = false;
        }

        let round_data = self.round_data.clone();
        let mut round_data = round_data.borrow_mut();
        if !round_data.music_track.playing() {
            round_data.music_track.play();
        }
    }

    pub fn draw(&mut self, _settings: &Settings, _data: &mut WorldData, ctx: &mut Context) {
        graphics::clear(ctx, [0.9, 0.2, 0.9, 1.0].into());
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
            self.begin_round = true;
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
