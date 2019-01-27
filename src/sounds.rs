use ggez::audio;
use ggez::Context;
use rand::{thread_rng, Rng};
//use rand::seq::SliceRandom;

pub struct Sounds {
    death: audio::Source,
    break1: audio::Source,
    break2: audio::Source,
    swat_gogogo: audio::Source,
    taunts: Vec<audio::Source>,
    swat: Vec<audio::Source>,
}

impl Sounds {
    pub fn load(ctx: &mut Context) -> Self {
        let mut taunts: Vec<audio::Source> = Vec::new();
        taunts.push(audio::Source::new(ctx, "/voice/must_hurt.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/get_some.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/blow_ass.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/bite_dust.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/off_lawn.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/ass_grass.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/eat_shit.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/get_wrecked.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/you_suck.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/cake.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/what_mess.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/want_some.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/piss_off.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/country_justice.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/rip_em.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/swallow_soul.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/boomstick.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/hail_king.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/shall_die.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/groovy.ogg").unwrap());
        taunts.push(audio::Source::new(ctx, "/voice/headshot.ogg").unwrap());

        let mut swat: Vec<audio::Source> = Vec::new();
        swat.push(audio::Source::new(ctx, "/voice/swat1.ogg").unwrap());
        swat.push(audio::Source::new(ctx, "/voice/swat2.ogg").unwrap());
        swat.push(audio::Source::new(ctx, "/voice/swat3.ogg").unwrap());
        swat.push(audio::Source::new(ctx, "/voice/swat4.ogg").unwrap());

        Sounds {
            death: audio::Source::new(ctx, "/sound/death.wav").unwrap(),
            break1: audio::Source::new(ctx, "/sound/barrel_break.wav").unwrap(),
            break2: audio::Source::new(ctx, "/sound/crate_break.wav").unwrap(),
            swat_gogogo: audio::Source::new(ctx, "/voice/swat_gogogo.ogg").unwrap(),
            taunts,
            swat,
        }
    }

    pub fn play_taunt(&mut self) {
        let mut rng = thread_rng();
        let index = rng.gen_range(0, self.taunts.len());
        let taunt = &mut self.taunts[index as usize];
        //if !taunt.playing() {
        taunt.set_volume(1.0);
        taunt.play().unwrap();
        //}
    }

    pub fn play_swat(&mut self) {
        let mut rng = thread_rng();
        let index = rng.gen_range(0, self.swat.len());
        let taunt = &mut self.swat[index as usize];
        //if !taunt.playing() {
        taunt.set_volume(1.0);
        taunt.play().unwrap();
        //}
    }

    pub fn play_swat_gogogo(&mut self) {
        self.swat_gogogo.play().unwrap();
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

pub fn inverse_distance(distance: f32, min_distance: f32, max_distance: f32, roll_off: f32) -> f32 {
    let distance = distance.max(min_distance);
    let distance = distance.min(max_distance);
    min_distance / (min_distance + roll_off * (distance - min_distance))
}

pub fn linear_distance(distance: f32, min_distance: f32, max_distance: f32, roll_off: f32) -> f32 {
    let distance = distance.max(min_distance);
    let distance = distance.min(max_distance);
    1.0 - roll_off * (distance - min_distance) / (max_distance - min_distance)
}

pub fn exponential_distance(
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    roll_off: f32,
) -> f32 {
    let distance = distance.max(min_distance);
    let distance = distance.min(max_distance);
    (distance / min_distance).powf(-roll_off)
}
