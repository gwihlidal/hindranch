use crate::Point2;
use crate::Positional;
use crate::RigidBody;

pub trait Enemy {
    fn health(&self) -> f32;
    fn damage(&mut self, amount: f32);
    fn alive(&self) -> bool;
    fn closest_target(&self) -> Option<Point2>;
    fn position(&self) -> Point2;
    fn rotation(&self) -> f32;
    fn physics_move(&mut self, rigid_body: &RigidBody<f32>);
    fn absolute_move(&mut self, position: Point2);
    fn delta_move(&mut self, delta: Point2);
}

pub struct Bulldozer {
    health: f32,
    positional: Positional,
}

impl Bulldozer {
    pub fn new(health: f32, positional: Positional) -> Self {
        Bulldozer { health, positional }
    }
}

impl Enemy for Bulldozer {
    fn health(&self) -> f32 {
        self.health
    }

    fn damage(&mut self, amount: f32) {
        self.health -= 
        amount.min(self.health);
    }

    fn alive(&self) -> bool {
        self.health > 0.0
    }

    fn closest_target(&self) -> Option<Point2> {
        None
    }

    fn position(&self) -> Point2 {
        self.positional.position
    }

    fn rotation(&self) -> f32 {
        self.positional.rotation
    }

    fn physics_move(&mut self, rigid_body: &RigidBody<f32>) {
        let pos = rigid_body.position();
        self.positional.position = pos.translation.vector.into();
        self.positional.rotation = pos.rotation.angle();
    }

    fn absolute_move(&mut self, position: Point2) {
        self.positional.position = position;
    }

    fn delta_move(&mut self, delta: Point2) {
        self.positional.position = Point2::new(
            self.positional.position.x + delta.x,
            self.positional.position.y + delta.y,
        );
    }
}

pub struct Sheriff {
    health: f32,
    positional: Positional,
}

impl Sheriff {
    pub fn new(health: f32, positional: Positional) -> Self {
        Sheriff { health, positional }
    }
}

impl Enemy for Sheriff {
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

    fn position(&self) -> Point2 {
        self.positional.position
    }

    fn rotation(&self) -> f32 {
        self.positional.rotation
    }

    fn physics_move(&mut self, rigid_body: &RigidBody<f32>) {
        let pos = rigid_body.position();
        self.positional.position = pos.translation.vector.into();
        self.positional.rotation = pos.rotation.angle();
    }

    fn absolute_move(&mut self, position: Point2) {
        self.positional.position = position;
    }

    fn delta_move(&mut self, delta: Point2) {
        self.positional.position = Point2::new(
            self.positional.position.x + delta.x,
            self.positional.position.y + delta.y,
        );
    }
}
