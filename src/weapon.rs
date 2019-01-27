use super::types::*;
use rand::Rng;
use std::io::Read;

pub struct Bullet {
    pub pos: Positional,
    pub velocity: f32,
    pub life_seconds: f32,
}

#[derive(Deserialize)]
pub struct WeaponConfig {
    pub bullets_per_round: u32,
    pub bullet_velocity: f32,
    pub bullet_life_seconds: f32,
    pub fire_rate: f32,
    pub spread_degrees: f32,
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
}

impl Weapon {
    pub fn from_config(cfg: WeaponConfig) -> Self {
        Self { cfg, cooldown: 0.0 }
    }

    pub fn update<'a>(&mut self, shoot: bool, pos: &Positional, sink: &'a mut Vec<Bullet>) {
        self.cooldown -= 1.0 / 60.0;
        if shoot && self.cooldown <= 0.0 {
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
                });
            }
        }
    }
}
