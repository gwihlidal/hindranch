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

fn clamp_norm(v: Vector2, max_norm: f32) -> Vector2 {
    let n = Vector2::norm(&v);
    if n > max_norm {
        v * (max_norm / n)
    } else {
        v
    }
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

        let velocity = rigid_body.velocity().linear;

        const MAX_FORCE: f32 = 1.0;
        const FORCE_RATE: f32 = 0.2;
        const MAX_VEL: f32 = 7.0;

        let mut target_vel = Vector2::zeros();

        if !settings.dozer_drive {
            if self.input.right {
                target_vel.x += 1.0;
            }
            if self.input.left {
                target_vel.x -= 1.0;
            }
            if self.input.up {
                target_vel.y += 1.0;
            }
            if self.input.down {
                target_vel.y -= 1.0;
            }
        }

        target_vel *= MAX_VEL;
        target_vel -= velocity;

        let force = clamp_norm(target_vel * FORCE_RATE, MAX_FORCE);

        rigid_body.activate();
        rigid_body.apply_force(&Force2::new(force, 0.0));
        let mut pos = rigid_body.position();
        pos.rotation = nalgebra::UnitComplex::from_angle(0.0);
        rigid_body.set_position(pos);
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
