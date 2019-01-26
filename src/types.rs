pub use nalgebra as na;
pub use nphysics2d::object::RigidBody;

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;
pub type Vector3 = na::Vector3<f32>;
pub type Matrix4 = na::Matrix4<f32>;

#[derive(Debug, Clone, Copy)]
pub struct Positional {
    pub position: Point2,
    pub rotation: f32,
}

impl Positional {
    pub fn set_from_physics(&mut self, rigid_body: &RigidBody<f32>) {
        let pos = rigid_body.position();
        self.position = pos.translation.vector.into();
        self.rotation = pos.rotation.angle();
    }

    // Assumes sprites face right
    pub fn forward(&self) -> Vector2 {
        Vector2::new(self.rotation.cos(), self.rotation.sin())
    }

    pub fn right(&self) -> Vector2 {
        let forward = self.forward();
        Vector2::new(forward.y, -forward.x)
    }
}

impl Default for Positional {
    fn default() -> Self {
        Self {
            position: Point2::origin(),
            rotation: 0.0,
        }
    }
}
