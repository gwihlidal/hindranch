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

    pub fn play(&mut self) {
        if self.source.paused() {
            self.source.resume();
        } else {
            self.source.play().unwrap();
        }
    }

    /*pub fn pause(&mut self) {
        if !self.source.paused() {
            self.source.pause();
        }
    }*/

    pub fn stop(&mut self) {
        if self.source.playing() {
            self.source.stop();
        }
    }   
}
