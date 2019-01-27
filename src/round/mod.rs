use crate::{graphics, MusicTrack, Context, KeyCode};

pub struct RoundPhase {
    music_track: MusicTrack,
}

impl RoundPhase {
    pub fn new(ctx: &mut Context) -> RoundPhase {
        RoundPhase {
            music_track: MusicTrack::new("twisted", ctx),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        if !self.music_track.playing() {
            self.music_track.play();
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        let identity_transform = graphics::transform(ctx);
    }

    pub fn handle_key(&mut self, keycode: KeyCode, value: bool) {}
}
