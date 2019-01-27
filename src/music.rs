use ggez::audio;
use ggez::Context;

pub struct MusicTrack {
    source: audio::Source,
}

impl MusicTrack {
    pub fn new(name: &str, ctx: &mut Context) -> Self {
        MusicTrack {
            source: audio::Source::new(ctx, format!("/music/{}.mp3", name)).unwrap(),
        }
    }

    pub fn playing(&self) -> bool {
        self.source.playing()
    }

    pub fn volume(&mut self, vol: f32) {
        self.source.set_volume(vol);
    }

    pub fn play(&mut self) {
        if self.source.paused() {
            self.source.resume();
        } else {
            self.source.play().unwrap();
        }
    }
}
