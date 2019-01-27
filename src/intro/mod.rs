#![allow(unused_imports)]

use crate::{graphics, Context, KeyCode, MouseButton, Settings, VoiceQueue, WorldData};

pub struct IntroPhase {
    pub first_update: bool,
    pub begin_game: bool,
    pub voice_queue: VoiceQueue,
}

impl IntroPhase {
    pub fn new(_ctx: &mut Context) -> Self {
        IntroPhase {
            first_update: true,
            begin_game: false,
            voice_queue: VoiceQueue::new(),
        }
    }

    pub fn update(&mut self, settings: &Settings, _data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            println!("STATE: Intro");
            if settings.voice {
                self.voice_queue.enqueue("shout", ctx);
                self.voice_queue.enqueue("defiance", ctx);
            }
            self.first_update = false;
        }

        self.voice_queue.process();
    }

    pub fn draw(&mut self, _settings: &Settings, _data: &mut WorldData, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.2, 0.9, 1.0].into());
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
            self.begin_game = true;
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
