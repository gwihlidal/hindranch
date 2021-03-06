use super::types::*;
use crate::Context;
use ggez::audio;
use rand::Rng;
use std::io::Read;

pub struct Bullet {
    pub pos: Positional,
    pub allegiance: usize,
    pub velocity: f32,
    pub life_seconds: f32,
    pub damage: f32,
}

#[derive(Deserialize, Clone)]
pub struct WeaponConfig {
    pub bullets_per_round: u32,
    pub bullet_velocity: f32,
    pub bullet_life_seconds: f32,
    pub bullet_damage: f32,
    pub fire_rate: f32,
    pub spread_degrees: f32,
    pub sound_file: String,
}

impl WeaponConfig {
    pub fn from_toml(path: &str) -> Self {
        let mut file =
            std::fs::File::open(path).expect(&format!("failed to open weapon config {}", path));
        let mut toml = String::new();
        file.read_to_string(&mut toml)
            .expect(&format!("failed to open weapon config {}", path));
        toml::from_str(&toml).unwrap()
    }
}

pub struct Weapon {
    cfg: WeaponConfig,
    cooldown: f32,
    audio_source: audio::Source,
}

impl Weapon {
    pub fn from_config(ctx: &mut Context, cfg: WeaponConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            cooldown: 0.0,
            audio_source: audio::Source::new(ctx, cfg.sound_file).unwrap(),
        }
    }

    pub fn update(
        &mut self,
        shoot: bool,
        pos: &Positional,
        allegiance: usize,
        sink: &mut Vec<Bullet>,
    ) {
        self.cooldown -= 1.0 / 60.0;
        if shoot && self.cooldown <= 0.0 {
            self.audio_source.play().unwrap();

            self.cooldown = 1.0 / self.cfg.fire_rate;
            let mut rng = rand::thread_rng();
            let half_spread_radians = self.cfg.spread_degrees.max(1e-5).to_radians() * 0.5;

            for _ in 0..self.cfg.bullets_per_round {
                sink.push(Bullet {
                    pos: Positional {
                        position: pos.position,
                        rotation: pos.rotation
                            + rng.gen_range(-half_spread_radians, half_spread_radians),
                    },
                    velocity: self.cfg.bullet_velocity,
                    life_seconds: self.cfg.bullet_life_seconds,
                    damage: self.cfg.bullet_damage,
                    allegiance,
                });
            }
        }
    }
}
