use crate::{
    clamp_norm, draw_single_image, exponential_distance, AiBehavior, BodyHandle, Bullet, Color,
    Context, Force2, Movement, PawnInput, Player, Point2, Positional, Settings, Sounds, Vector2,
    World, GROUP_ENEMY, SWAT_INNER_RADIUS, SWAT_OUTER_RADIUS,
};

use super::player::VisualState;

use ggez::audio;
use ggez::graphics;
use nalgebra as na;
use ncollide2d::query::Ray;
use ncollide2d::world::CollisionGroups;
use rand::Rng;
use std::rc::Rc;
use std::time::{Duration, Instant};

const SWAT_MOVE_SPEED: f32 = 0.75;

pub trait Enemy {
    fn update(
        &mut self,
        settings: &Settings,
        player_pos: Positional,
        movement: Option<Movement>,
        world: &mut World<f32>,
        bullets_out: &mut Vec<Bullet>,
        sounds: &mut Sounds,
    );
    fn rigid_body(&self) -> Option<BodyHandle>;
    fn draw(&self, ctx: &mut Context);
    fn color(&self) -> Color {
        Color::new(1.0, 1.0, 1.0, 1.0)
    }
    fn health(&self) -> f32;
    fn damage(&mut self, amount: f32);
    fn alive(&self) -> bool;
    fn closest_target(&self) -> Option<Point2>;
    fn positional(&self) -> Positional;
}

pub struct Bulldozer {
    engine_source: audio::SpatialSource,
    movement: Movement,
    rigid_body: BodyHandle,
    image: Rc<graphics::Image>,
    health: f32,
    positional: Positional,
    behavior: Option<Box<dyn AiBehavior>>,
    time_since_last_damage: f32,
}

impl Bulldozer {
    pub fn new(
        ctx: &mut Context,
        engine_sound: audio::SoundData,
        rigid_body: BodyHandle,
        image: Rc<graphics::Image>,
        positional: Positional,
        behavior: Option<Box<dyn AiBehavior>>,
    ) -> Self {
        let mut engine_source = audio::SpatialSource::from_data(ctx, engine_sound.clone()).unwrap();
        engine_source.set_repeat(true);
        Bulldozer {
            engine_source,
            movement: Movement {
                forward: 0.0,
                right: 0.0,
            },
            rigid_body,
            image,
            health: 1.0,
            positional,
            behavior,
            time_since_last_damage: 10000.0,
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
        const MAX_VEL: f32 = 12.0;

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
    fn update(
        &mut self,
        settings: &Settings,
        player_pos: Positional,
        movement: Option<Movement>,
        world: &mut World<f32>,
        _bullets_out: &mut Vec<Bullet>,
        _sounds: &mut Sounds,
    ) {
        if let Some(ref mut behavior) = self.behavior {
            self.movement = behavior.update(world.rigid_body(self.rigid_body).unwrap());
        }

        self.apply_physics_movement(&movement.unwrap_or(self.movement), world);

        //self.engine_source.set_ears(na::Point3::new(player_pos.position.x, player_pos.position.y, 1.0), na::Point3::new(player_pos.position.x, player_pos.position.y, 1.0));
        //self.engine_source.set_position(na::Point3::new(self.positional.position.x, self.positional.position.y, 1.0));

        if settings.sounds {
            let max = 1000.0;
            let min = 4.0;
            let roll_off = 1.5;

            let ear_distance = na::distance(&player_pos.position, &self.positional.position);
            let volume = exponential_distance(ear_distance, min, max, roll_off);
            //println!("Volume: {}", volume);

            self.engine_source.set_volume(volume);

            if !self.engine_source.playing() {
                self.engine_source.play().unwrap();
            }
        }

        self.time_since_last_damage += 1.0 / 60.0;
    }

    fn rigid_body(&self) -> Option<BodyHandle> {
        Some(self.rigid_body)
    }

    fn draw(&self, ctx: &mut Context) {
        draw_single_image(
            ctx,
            &self.image,
            self.color(),
            self.positional.position,
            3.0,
            self.positional.rotation,
        );
    }

    fn color(&self) -> Color {
        let t = self.time_since_last_damage;
        let t = (1.0 - t * 5.0).max(0.0) * 10.0;
        Color::new(1.0 + t, self.health + t, self.health + t, 1.0)
    }

    fn health(&self) -> f32 {
        self.health
    }

    fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount).max(0.0);
        self.time_since_last_damage = 0.0;
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

pub struct Swat {
    pawn: Player,
    waypoint: Option<Point2>,
    walk_direction: f32,
    keep_direction_until: Instant,
    next_taunt_at: Instant,
    avg_velocity: Vector2,
    avg_velocity_samples: u32,
}

impl Swat {
    pub fn new(pawn: Player) -> Self {
        let mut rng = rand::thread_rng();

        Swat {
            pawn,
            waypoint: None,
            walk_direction: if rng.gen::<bool>() { 1.0 } else { -1.0 },
            keep_direction_until: Instant::now()
                + Duration::from_millis(rng.gen_range(4000, 20000)),
            next_taunt_at: Instant::now() + Duration::from_millis(rng.gen_range(4000, 20000)),
            avg_velocity: Vector2::zeros(),
            avg_velocity_samples: 0,
        }
    }

    fn acquire_waypoint(&mut self) {
        let pos = self.positional().position.coords;
        let center_dist = pos.norm();

        let now = Instant::now();
        if now > self.keep_direction_until {
            let mut rng = rand::thread_rng();
            self.walk_direction = if rng.gen::<bool>() { 1.0 } else { -1.0 };
            self.keep_direction_until =
                Instant::now() + Duration::from_millis(rng.gen_range(4000, 20000));
        }

        let goal: Point2 = if center_dist < SWAT_INNER_RADIUS || center_dist > SWAT_OUTER_RADIUS {
            (pos.normalize() * (SWAT_INNER_RADIUS * 0.5 + SWAT_OUTER_RADIUS * 0.5)).into()
        } else {
            let a = pos.y.atan2(pos.x) + (0.1 + 0.1 * rand::random::<f32>()) * self.walk_direction;
            (Vector2::new(a.cos(), a.sin())
                * (SWAT_INNER_RADIUS
                    + (SWAT_OUTER_RADIUS - SWAT_INNER_RADIUS) * rand::random::<f32>()))
            .into()
        };

        self.waypoint = Some(goal);
    }
}

impl Swat {
    fn is_player_visible(&self, player_pos: &Point2, world: &World<f32>) -> bool {
        let collision_world = world.collision_world();
        let swat_pos = self.positional().position;

        let mut groups = CollisionGroups::new();
        groups.set_blacklist(&[GROUP_ENEMY]);

        let offset = player_pos - swat_pos;
        let offset_len = offset.norm();

        let ray = Ray {
            origin: swat_pos,
            dir: offset * (1.0 / offset_len),
        };

        let min_toi = collision_world
            .interferences_with_ray(&ray, &groups)
            .map(|(_, collision)| collision.toi)
            .min_by(|a, b| a.partial_cmp(b).unwrap());

        if let Some(min_toi) = min_toi {
            (min_toi - offset_len).abs() < 0.5
        } else {
            false
        }
    }
}

impl Enemy for Swat {
    fn update(
        &mut self,
        _settings: &Settings,
        player_pos: Positional,
        _movement: Option<Movement>,
        world: &mut World<f32>,
        bullets_out: &mut Vec<Bullet>,
        sounds: &mut Sounds,
    ) {
        let cur_vel = world
            .rigid_body(self.pawn.body_handle)
            .unwrap()
            .velocity()
            .linear;

        self.avg_velocity = self.avg_velocity * 0.9 + cur_vel * 0.1;
        self.avg_velocity_samples += 1;

        // Some chaos. Helps getting unstuck and whatnot
        if self.avg_velocity.norm() < 0.1 && self.avg_velocity_samples > 10 {
            self.waypoint = None;
            self.walk_direction *= -1.0;
            self.avg_velocity = Vector2::zeros();
            self.avg_velocity_samples = 0;
        }

        if self.waypoint.is_none() {
            self.acquire_waypoint();
        }

        let now = Instant::now();
        if now > self.next_taunt_at {
            let mut rng = rand::thread_rng();
            self.next_taunt_at = Instant::now() + Duration::from_millis(rng.gen_range(4000, 20000));
            sounds.play_swat();
        }

        if let Some(w) = self.waypoint {
            let pos = self.positional().position;
            if (pos - w).norm() < 0.5 {
                self.waypoint = None;
            }

            let player_visible = self.is_player_visible(&player_pos.position, world);

            let offset = if player_visible {
                //Vector2::zeros()
                clamp_norm(w - pos, SWAT_MOVE_SPEED * 0.5)
            } else {
                clamp_norm(w - pos, SWAT_MOVE_SPEED)
            };

            self.pawn.set_input(PawnInput {
                movement: Movement {
                    right: offset.x,
                    forward: offset.y,
                },
                shoot: player_visible,
                aim_pos: if player_visible {
                    player_pos.position
                } else {
                    Point2::origin()
                },
            });
        }

        self.pawn.set_visual(VisualState::Gun);
        self.pawn.update(world, bullets_out);
    }

    fn rigid_body(&self) -> Option<BodyHandle> {
        Some(self.pawn.body_handle)
    }

    fn draw(&self, _ctx: &mut Context) {
        self.pawn.draw();
    }

    fn health(&self) -> f32 {
        self.pawn.health
    }

    fn damage(&mut self, amount: f32) {
        self.pawn.damage(amount)
    }

    fn alive(&self) -> bool {
        self.pawn.alive()
    }

    fn closest_target(&self) -> Option<Point2> {
        None
    }

    fn positional(&self) -> Positional {
        self.pawn.positional
    }
}
