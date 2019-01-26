use crate::{
    graphics::spritebatch::SpriteBatch, graphics::DrawParam, settings::Settings, Ball, BodyHandle,
    Characters, Force2, Isometry2, Material, Point2, Positional, Rect, ShapeHandle, Vector2,
    Volumetric, World,
};
use nalgebra as na;

const COLLIDER_MARGIN: f32 = 0.01;

pub enum VisualState {
    Gun,
    Hold,
    Machine,
    Reload,
    Silencer,
    Stand,
}

pub struct Player {
    pub health: f32,
    pub input: PlayerInput,
    pub body_handle: BodyHandle,
    pub visual: VisualState,
    pub positional: Positional,
    pub gun: (Rect, Vector2),
    pub hold: (Rect, Vector2),
    pub machine: (Rect, Vector2),
    pub reload: (Rect, Vector2),
    pub silencer: (Rect, Vector2),
    pub stand: (Rect, Vector2),
    pub dead_gun: (Rect, Vector2),
    pub dead_hold: (Rect, Vector2),
    pub dead_machine: (Rect, Vector2),
    pub dead_reload: (Rect, Vector2),
    pub dead_silencer: (Rect, Vector2),
    pub dead_stand: (Rect, Vector2),
}

impl Player {
    pub fn new(
        world: &mut World<f32>,
        name: &str,
        health: f32,
        pos: Point2,
        characters: &Characters,
    ) -> Self {
        let entry = characters.get_entry(name);
        let zombie = characters.get_entry("zombie");

        let geom = ShapeHandle::new(Ball::new(0.19));
        let inertia = geom.inertia(0.1);
        let center_of_mass = geom.center_of_mass();

        let pos = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
        let rb = world.add_rigid_body(pos, inertia, center_of_mass);

        world.add_collider(
            COLLIDER_MARGIN,
            geom.clone(),
            rb,
            Isometry2::identity(),
            Material::new(0.0, 0.0),
        );

        Player {
            health,
            input: PlayerInput::default(),
            body_handle: rb,
            visual: VisualState::Stand,
            positional: Positional::default(),
            gun: characters.transform(&entry.gun),
            hold: characters.transform(&entry.hold),
            machine: characters.transform(&entry.machine),
            reload: characters.transform(&entry.reload),
            silencer: characters.transform(&entry.silencer),
            stand: characters.transform(&entry.stand),
            dead_gun: characters.transform(&zombie.gun),
            dead_hold: characters.transform(&zombie.hold),
            dead_machine: characters.transform(&zombie.machine),
            dead_reload: characters.transform(&zombie.reload),
            dead_silencer: characters.transform(&zombie.silencer),
            dead_stand: characters.transform(&zombie.stand),
        }
    }

    pub fn draw(&self, batch: &mut SpriteBatch) {
        let (rect, scale) = match self.visual {
            VisualState::Gun => if self.alive() { self.gun } else { self.dead_gun },
            VisualState::Hold => if self.alive() { self.hold } else { self.dead_hold },
            VisualState::Machine => if self.alive() { self.machine } else { self.dead_machine },
            VisualState::Reload => if self.alive() { self.reload } else { self.dead_reload },
            VisualState::Silencer => if self.alive() { self.silencer } else { self.dead_silencer },
            VisualState::Stand => if self.alive() { self.stand } else { self.dead_stand },
        };
        batch.add(
            DrawParam::new()
                .src(rect)
                .dest(self.positional.position - Vector2::new(0.5, 0.5))
                .scale(scale)
                .offset(Point2::new(0.5, 0.5))
                .rotation(self.positional.rotation),
        );
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn damage(&mut self, amount: f32) {
        self.health -= amount.min(self.health);
    }

    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn set_visual(&mut self, visual: VisualState) {
        self.visual = visual;
    }

    pub fn update(&mut self, settings: &Settings, world: &mut World<f32>) {
        let rigid_body = world.rigid_body_mut(self.body_handle).unwrap();
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
        const TORQUE_RATE: f32 = 0.005;
        const MAX_SPIN: f32 = 5.0;

        const MAX_FORCE: f32 = 100.0;
        const FORCE_RATE: f32 = 0.2;
        const MAX_VEL: f32 = 7.0;

        const SIDEWAYS_DAMPING: f32 = 0.5;

        let mut target_vel = 0.0;
        let mut target_spin = 0.0;

        if !settings.dozer_drive {
            if self.input.right {
                target_spin -= 1.0;
            }
            if self.input.left {
                target_spin += 1.0;
            }
            if self.input.up {
                target_vel += 1.0;
            }
            if self.input.down {
                target_vel -= 1.0;
            }
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

#[derive(Debug)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn new() -> PlayerInput {
        PlayerInput {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

impl Default for PlayerInput {
    fn default() -> Self {
        PlayerInput::new()
    }
}
