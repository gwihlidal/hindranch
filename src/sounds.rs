use ggez::audio;
use ggez::Context;

pub struct Sounds {
    death: audio::Source,
    break1: audio::Source,
    break2: audio::Source,
}

impl Sounds {
    pub fn load(ctx: &mut Context) -> Self {
        Sounds {
            death: audio::Source::new(ctx, "/sound/death.wav").unwrap(),
            break1: audio::Source::new(ctx, "/sound/barrel_break.wav").unwrap(),
            break2: audio::Source::new(ctx, "/sound/crate_break.wav").unwrap(),
        }
    }

    pub fn play_death(&mut self) {
        self.death.play().unwrap();
    }

    pub fn play_break1(&mut self) {
        self.break1.play().unwrap();
    }

    pub fn play_break2(&mut self) {
        self.break2.play().unwrap();
    }

}

