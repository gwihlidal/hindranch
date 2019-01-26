#![allow(dead_code)]

use crate::{BodyHandle, Force2, Point2, Positional, Vector2, World};

use ggez::graphics;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct Movement {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

pub trait Enemy {
    fn update(&mut self, movement: Option<Movement>, world: &mut World<f32>);
    fn rigid_body(&self) -> Option<BodyHandle>;
    fn image(&self) -> Rc<graphics::Image>;
    fn health(&self) -> f32;
    fn damage(&mut self, amount: f32);
    fn alive(&self) -> bool;
    fn closest_target(&self) -> Option<Point2>;
    fn positional(&self) -> Positional;
}

pub trait AiBehavior {
    fn update(&mut self) -> Movement;
}

pub struct Bulldozer {
    movement: Movement,
    rigid_body: BodyHandle,
    image: Rc<graphics::Image>,
    health: f32,
    positional: Positional,
    behavior: Option<Box<dyn AiBehavior>>,
}

impl Bulldozer {
    pub fn new(
        rigid_body: BodyHandle,
        image: Rc<graphics::Image>,
        health: f32,
        positional: Positional,
        behavior: Option<Box<dyn AiBehavior>>,
    ) -> Self {
        Bulldozer {
            movement: Movement {
                left: false,
                right: false,
                up: false,
                down: false,
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

        let mut target_vel = 0.0;
        let mut target_spin = 0.0;

        if movement.right {
            target_spin -= 1.0;
        }
        if movement.left {
            target_spin += 1.0;
        }
        if movement.up {
            target_vel += 1.0;
        }
        if movement.down {
            target_vel -= 1.0;
        }

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
    fn update(&mut self, movement: Option<Movement>, world: &mut World<f32>) {
        if let Some(ref mut behavior) = self.behavior {
            self.movement = behavior.update();
        }

        self.apply_physics_movement(&movement.unwrap_or(self.movement), world);
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
    fn update(&mut self, _movement: Option<Movement>, _world: &mut World<f32>) {
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
