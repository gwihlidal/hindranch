#![allow(dead_code)]

use crate::{linear_distance, inverse_distance, exponential_distance, AiBehavior, Context, BodyHandle, Force2, Point2, Positional, Vector2, World};

use ggez::graphics;
use ggez::audio;
use std::default::Default;
use std::rc::Rc;
use nalgebra as na;

#[derive(Clone, Copy)]
pub struct Movement {
    pub forward: f32,
    pub right: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            forward: 0.0,
            right: 0.0,
        }
    }
}

pub trait Enemy {
    fn update(&mut self, player_pos: Positional, movement: Option<Movement>, world: &mut World<f32>);
    fn rigid_body(&self) -> Option<BodyHandle>;
    fn image(&self) -> Rc<graphics::Image>;
    fn health(&self) -> f32;
    fn damage(&mut self, amount: f32);
    fn alive(&self) -> bool;
    fn closest_target(&self) -> Option<Point2>;
    fn positional(&self) -> Positional;
}

pub struct Bulldozer {
    driving: bool,
    engine_source: audio::SpatialSource,
    movement: Movement,
    rigid_body: BodyHandle,
    image: Rc<graphics::Image>,
    health: f32,
    positional: Positional,
    behavior: Option<Box<dyn AiBehavior>>,
}

impl Bulldozer {
    pub fn new(
        ctx: &mut Context,
        engine_sound: audio::SoundData,
        rigid_body: BodyHandle,
        image: Rc<graphics::Image>,
        health: f32,
        positional: Positional,
        behavior: Option<Box<dyn AiBehavior>>,
    ) -> Self {
        let mut engine_source = audio::SpatialSource::from_data(ctx, engine_sound.clone()).unwrap();
        engine_source.set_repeat(true);
        Bulldozer {
            driving: false,
            engine_source,
            movement: Movement {
                forward: 0.0,
                right: 0.0,
            },
            rigid_body,
            image,
            health,
            positional,
            behavior,
        }
    }

    fn apply_physics_movement(&mut self, movement: &Movement, world: &mut World<f32>) {
        let rigid_body = world.rigid_body_mut(self.rigid_body).unwrap();
        let pos = rigid_body.position();
        self.positional.position = pos.translation.vector.into();
        self.positional.rotation = pos.rotation.angle();

        let forward = self.positional.forward();
        let right = self.positional.right();

        let velocity = rigid_body.velocity().linear;
        let fwd_vel = Vector2::dot(&forward, &velocity);
        let right_vel = Vector2::dot(&right, &velocity);

        let spin = rigid_body.velocity().angular;

        const MAX_TORQUE: f32 = 1000.0;
        const TORQUE_RATE: f32 = 500.0;
        const MAX_SPIN: f32 = 2.0;

        const MAX_FORCE: f32 = 500.0;
        const FORCE_RATE: f32 = 200.0;
        const MAX_VEL: f32 = 10.0;

        // Bulldozers technically don't strafe, but we have a need for speed.
        const SIDEWAYS_DAMPING: f32 = 0.1;

        let mut target_vel = movement.forward.min(1.0).max(-1.0);
        let mut target_spin = (-movement.right).min(1.0).max(-1.0);

        target_spin *= MAX_SPIN;
        target_spin -= spin;

        target_vel *= MAX_VEL;
        target_vel -= fwd_vel;

        let torque = (target_spin * TORQUE_RATE).max(-MAX_TORQUE).min(MAX_TORQUE);
        let force = forward * (target_vel * FORCE_RATE).max(-MAX_FORCE).min(MAX_FORCE);

        rigid_body.activate();
        rigid_body.set_linear_velocity(velocity - right_vel * right * SIDEWAYS_DAMPING);
        rigid_body.apply_force(&Force2::new(force, torque));
    }
}

impl Enemy for Bulldozer {
    fn update(&mut self, player_pos: Positional, movement: Option<Movement>, world: &mut World<f32>) {
        if let Some(ref mut behavior) = self.behavior {
            self.movement = behavior.update(world.rigid_body(self.rigid_body).unwrap());
        }

        self.apply_physics_movement(&movement.unwrap_or(self.movement), world);

        //self.engine_source.set_ears(na::Point3::new(player_pos.position.x, player_pos.position.y, 1.0), na::Point3::new(player_pos.position.x, player_pos.position.y, 1.0));
        //self.engine_source.set_position(na::Point3::new(self.positional.position.x, self.positional.position.y, 1.0));

        let max = 1000.0;
        let min = 4.0;
        let roll_off = 1.5;
        
        let ear_distance = na::distance(&player_pos.position, &self.positional.position);
        let volume = exponential_distance(ear_distance, min, max, roll_off);
        //println!("Volume: {}", volume);

        self.engine_source.set_volume(volume);

        //if self.driving {
            //self.engine_source.set_pitch(1.0);
        //} else {
        //    self.engine_source.set_pitch(0.7);
        //}

        if !self.engine_source.playing() {
            self.engine_source.play().unwrap();
        }
    }

    fn rigid_body(&self) -> Option<BodyHandle> {
        Some(self.rigid_body)
    }

    fn image(&self) -> Rc<graphics::Image> {
        self.image.clone()
    }

    fn health(&self) -> f32 {
        self.health
    }

    fn damage(&mut self, amount: f32) {
        self.health -= amount.min(self.health);
    }

    fn alive(&self) -> bool {
        self.health > 0.0
    }

    fn closest_target(&self) -> Option<Point2> {
        None
    }

    fn positional(&self) -> Positional {
        self.positional.clone()
    }
}

pub struct Sheriff {
    image: Rc<graphics::Image>,
    health: f32,
    positional: Positional,
}

impl Sheriff {
    pub fn new(image: Rc<graphics::Image>, health: f32, positional: Positional) -> Self {
        Sheriff {
            image,
            health,
            positional,
        }
    }
}

impl Enemy for Sheriff {
    fn update(&mut self, _player_pos: Positional, _movement: Option<Movement>, _world: &mut World<f32>) {
        //
    }

    fn rigid_body(&self) -> Option<BodyHandle> {
        None
    }

    fn image(&self) -> Rc<graphics::Image> {
        self.image.clone()
    }

    fn health(&self) -> f32 {
        self.health
    }

    fn damage(&mut self, amount: f32) {
        self.health -= amount.min(self.health);
    }

    fn alive(&self) -> bool {
        self.health > 0.0
    }

    fn closest_target(&self) -> Option<Point2> {
        None
    }

    fn positional(&self) -> Positional {
        self.positional.clone()
    }
}
