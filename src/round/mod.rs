use crate::{graphics, Context, KeyCode};

pub struct RoundPhase {}

impl RoundPhase {
    pub fn new() -> RoundPhase {
        RoundPhase {}
    }

    pub fn update(&mut self, ctx: &mut Context) {
        //
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let identity_transform = graphics::transform(ctx);
    }

    pub fn handle_key(&mut self, keycode: KeyCode, value: bool) {}
}
