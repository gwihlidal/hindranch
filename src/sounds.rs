use ggez::audio;
use ggez::Context;

pub struct Sounds {
    death: audio::Source,
}

impl Sounds {
    pub fn load(ctx: &mut Context) -> Self {
        Sounds {
            death: audio::Source::new(ctx, "/sound/death.wav").unwrap(),
        }
    }

    pub fn play_death(&mut self) {
        self.death.play().unwrap();
    }
}