use crate::{graphics, Sounds, Context, KeyCode};

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
    pub fn update(&mut self, ctx: &mut Context) {
        if self.first_update {
            self.sounds.play_death();
            self.first_update = false;
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        //
    }

    pub fn handle_key(&mut self, key_code: KeyCode, value: bool) {
        if key_code == KeyCode::Space && value {
            self.want_restart = true;
        }
    }
}
