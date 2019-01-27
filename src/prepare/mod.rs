use crate::{graphics, Context, KeyCode};

pub struct PreparePhase {}

impl PreparePhase {
    pub fn update(&mut self, ctx: &mut Context) {
        //
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        //
    }

    pub fn handle_key(&mut self, keycode: KeyCode, value: bool) {}
}
