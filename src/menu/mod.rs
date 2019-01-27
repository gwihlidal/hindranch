use crate::{graphics, MusicTrack, Context, KeyCode};

pub struct MenuPhase {
    music_track: MusicTrack,
}

impl MenuPhase {
    pub fn new(ctx: &mut Context) -> Self {
        MenuPhase {
            music_track: MusicTrack::new("cantina", ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        if !self.music_track.playing() {
            self.music_track.play();
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        //
    }

    pub fn handle_key(&mut self, keycode: KeyCode, value: bool) {}
}
