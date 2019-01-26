use super::consts::*;
use super::enemy::Movement;
use super::types::*;

use rand::Rng;
use std::time::{Duration, Instant};

pub trait AiBehavior {
    fn update(&mut self, rb: &RigidBody<f32>) -> Movement;
}

pub struct EnemyDozerBehavior {
    state: DozerState,
    last_vel_mag: f32,

    // TODO: acquire from list of alive wall pieces
    look_at: Point2,
}

impl EnemyDozerBehavior {
    pub fn new() -> Self {
        Self {
            state: DozerState::IdlingUntil(
                Instant::now() + Duration::from_millis(rand::thread_rng().gen_range(1000, 2000)),
            ),
            last_vel_mag: 0.0,
            look_at: Point2::origin(),
        }
    }
}

#[derive(Clone, Copy)]
enum DozerState {
    IdlingUntil(Instant),
    Ramming,
    RammingUntil(Instant),
    BackingAway,
}

impl AiBehavior for EnemyDozerBehavior {
    fn update(&mut self, rb: &RigidBody<f32>) -> Movement {
        let mut rng = rand::thread_rng();

        let vel_mag = rb.velocity().linear.norm();
        let pos = rb.position().translation.vector;
        let dist_to_center = pos.norm();
        let now = Instant::now();

        let mut movement = Movement::default();

        match self.state {
            DozerState::IdlingUntil(t) => {
                if now > t {
                    self.state = DozerState::Ramming;
                }
            }
            DozerState::Ramming => {
                // If we've lost a good chunk of velocity, assume we hit an obstacle, start backing away
                if dist_to_center < DOZER_OUTER_RADIUS && vel_mag < self.last_vel_mag * 0.9 {
                    self.state = DozerState::RammingUntil(
                        now + Duration::from_millis(rng.gen_range(1000, 2000)),
                    )
                }

                movement.forward = 1.0;
            }
            DozerState::RammingUntil(t) => {
                if now > t {
                    self.state = DozerState::BackingAway;
                }

                movement.forward = 1.0;
            }
            DozerState::BackingAway => {
                if dist_to_center > DOZER_OUTER_RADIUS {
                    self.state = DozerState::Ramming;
                }

                movement.forward = -1.0;
            }
        }

        {
            let mut p = Positional::default();
            p.set_from_physics(rb);
            movement.right = p.right().dot(&(self.look_at - p.position));
        }

        self.last_vel_mag = vel_mag;

        movement
    }
}
