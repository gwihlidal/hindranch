use ggez::audio;
use ggez::Context;

pub struct MusicTrack {
    source: audio::Source,
}

impl MusicTrack {
    pub new(name: &str, ctx: &mut Context) -> Self {
        MusicTrack {
            source: audio::Source::new(ctx, format!("/music/{}.ogg", name)).unwrap(),
        }
    }

    pub fn play(&mut self) {

    }

    pub fn pause(&mut self) {
        
    }

    pub fn stop(&mut self) {
        
    }
}
