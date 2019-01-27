#![allow(unused_imports)]

use super::consts::*;
use super::enemy::Swat;
use crate::{
    draw_map_layer, draw_shadowed_text, graphics, px_to_world, settings::Settings, AiBehavior,
    Bulldozer, Color, Context, Enemy, EnemyDozerBehavior, KeyCode, MainState, Matrix4, MouseButton,
    MusicTrack, Player, PlayerInput, Point2, Positional, RoundData, Vector2, Vector3, VisualState,
    Weapon, WeaponConfig, WorldData, DESIRED_FPS,
};
use std::cell::RefCell;
use std::rc::Rc;

use na::Isometry2;
use nalgebra as na;
use ncollide2d::query::Ray;
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use ncollide2d::world::CollisionGroups;
use nphysics2d::algebra::Force2;
use nphysics2d::force_generator::{ForceGeneratorHandle, Spring};
use nphysics2d::object::{BodyHandle, Material};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;

use ggez::audio;

pub struct RoundPhase {
    pub first_update: bool,
    pub round_index: u32,
    pub last_round: bool,
    pub victory: bool,
    pub failure: bool,
    pub round_data: Rc<RefCell<RoundData>>,
}

enum BulletHitVictim {
    Enemy(usize),
    Player,
    None,
}

impl RoundPhase {
    pub fn new(
        _ctx: &mut Context,
        round_index: u32,
        last_round: bool,
        round_data: Rc<RefCell<RoundData>>,
    ) -> RoundPhase {
        RoundPhase {
            first_update: true,
            round_index,
            last_round,
            victory: false,
            failure: false,
            round_data,
        }
    }

    fn spawn_bulldozers(&mut self, data: &mut WorldData, ctx: &mut Context, count: usize) {
        let a_off = rand::random::<f32>() * std::f32::consts::PI;

        // Stratified circular positioning
        for i in 0..count {
            let amin = i as f32 / count as f32;
            let amax = (i + 1) as f32 / count as f32;
            let a =
                a_off + (amin + (amax - amin) * rand::random::<f32>()) * std::f32::consts::PI * 2.0;

            const SPAWN_DIST: f32 = DOZER_OUTER_RADIUS;

            let dozer = spawn_dozer(
                ctx,
                &mut data.world,
                data.engine_data.clone(),
                data.dozer_image.clone(),
                Point2::new(a.cos() * SPAWN_DIST, a.sin() * SPAWN_DIST),
                std::f32::consts::PI + a,
            );
            data.enemies.push(dozer);
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            data.player_input = PlayerInput::default();

            if settings.enemies {
                self.spawn_bulldozers(data, ctx, 3);

                data.sounds.play_swat_gogogo();
                let swat_pawn = Player::new(
                    &mut data.world,
                    "soldier",
                    0.5,
                    Weapon::from_config(WeaponConfig::from_toml("resources/swat_smg.toml")),
                    Point2::new(30.0, 10.0),
                    GROUP_ENEMY,
                    &data.characters,
                    data.character_spritebatch.clone(),
                );
                data.enemies.push(Box::new(Swat::new(swat_pawn)));
            }

            self.first_update = false;
        }

        let round_data = self.round_data.clone();
        let mut round_data = round_data.borrow_mut();
        if settings.music && !round_data.music_track.playing() {
            round_data.music_track.play();
        }

        self.calculate_view_transform(
            data,
            &ctx,
            data.camera_pos,
            if data.strategic_view { 0.02 } else { 0.1 },
        );

        data.player.set_input((&data.player_input).into());
        data.player.update(&mut data.world, &mut data.bullets);

        for (i, enemy) in &mut data.enemies.iter_mut().enumerate() {
            if settings.dozer_drive && i == 0 {
                // TODO: Player controlled hack
                enemy.update(
                    settings,
                    enemy.positional(),
                    Some((&data.player_input).into()),
                    &mut data.world,
                    &mut data.bullets,
                    &mut data.sounds,
                );
            } else {
                enemy.update(
                    settings,
                    data.player.positional,
                    None,
                    &mut data.world,
                    &mut data.bullets,
                    &mut data.sounds,
                );
            }
        }

        {
            let (camera_positional, look_ahead, stiffness) =
                if settings.dozer_drive && data.enemies.len() > 0 {
                    (data.enemies[0].positional(), 4.0, 0.07)
                } else {
                    (data.player.positional, 0.0, 0.3)
                };

            self.update_camera(data, camera_positional, look_ahead, stiffness);
        }

        self.maintain_weapons(data);
        self.maintain_walls(data);
        self.maintain_enemies(data);

        if !data.player.alive() {
            self.failure = true;
        }

        data.world.step();
    }

    fn maintain_weapons(&mut self, data: &mut WorldData) {
        let collision_world = data.world.collision_world();
        for bullet in data.bullets.iter_mut() {
            let mut hit_victim = BulletHitVictim::None;

            let mut hit_anything = false;
            let mut groups = CollisionGroups::new();
            groups.set_blacklist(&[bullet.allegiance]);
            for (other_collider, collision) in collision_world.interferences_with_ray(
                &Ray {
                    origin: bullet.pos.position,
                    dir: bullet.pos.forward(),
                },
                &groups,
            ) {
                if collision.toi < bullet.velocity / 60.0 {
                    let other_body = data.world.collider_body_handle(other_collider.handle());
                    if other_body == Some(data.player.body_handle) {
                        hit_victim = BulletHitVictim::Player;
                    } else {
                        for (enemy_i, enemy) in data.enemies.iter().enumerate() {
                            if enemy.rigid_body() == other_body {
                                hit_victim = BulletHitVictim::Enemy(enemy_i);
                            }
                        }
                    }

                    hit_anything = true;
                    break;
                }
            }

            if hit_anything {
                bullet.life_seconds = 0.0;
            }

            match hit_victim {
                BulletHitVictim::Enemy(enemy_i) => data.enemies[enemy_i].damage(bullet.damage),
                BulletHitVictim::Player => data.player.damage(bullet.damage),
                BulletHitVictim::None => (),
            }
        }

        for bullet in data.bullets.iter_mut() {
            bullet.pos.position += bullet.pos.forward() * (bullet.velocity / DESIRED_FPS as f32);
            bullet.life_seconds -= 1.0 / (DESIRED_FPS as f32);
        }

        data.bullets.retain(|b| b.life_seconds > 0.0);
    }

    fn maintain_walls(&mut self, data: &mut WorldData) {
        // Dampen wall piece physics and calculate damage
        for wall_piece in data.wall_pieces.iter_mut() {
            if let Some(rb) = data.world.rigid_body_mut(wall_piece.rb) {
                let mut vel = rb.velocity().clone();

                wall_piece.hp =
                    (wall_piece.hp - MainState::wall_velocity_to_damage(&vel.linear)).max(0.0);

                vel.linear *= 0.95;
                vel.angular *= 0.95;
                rb.set_velocity(vel);
                let mut pos = rb.position().clone();
                pos.rotation = nalgebra::UnitComplex::from_angle(pos.rotation.angle() * 0.95);
                rb.set_position(pos);
            }
        }

        let wall_pieces_to_remove: Vec<_> = data
            .wall_pieces
            .iter()
            .enumerate()
            .filter_map(|(i, wp)| if wp.hp <= 0.0 { Some(i) } else { None })
            .collect();

        for i in wall_pieces_to_remove.into_iter().rev() {
            let wp = &data.wall_pieces[i];
            data.world.remove_bodies(&[wp.rb]);
            data.world.remove_force_generator(wp.spring);
            data.wall_pieces.swap_remove(i);
        }

        if data.wall_pieces.is_empty() {
            self.failure = true;
        }
    }

    fn maintain_enemies(&mut self, data: &mut WorldData) {
        let mut enemies_killed = Vec::new();
        for (i, e) in data.enemies.iter().enumerate() {
            if e.health() <= 0.0 {
                data.world.remove_bodies(&[e.rigid_body().unwrap()]);
                enemies_killed.push(i);
            }
        }

        for i in enemies_killed.iter().rev() {
            data.enemies.swap_remove(*i);
        }

        if !enemies_killed.is_empty() {
            data.sounds.play_taunt();
            if data.enemies.is_empty() {
                self.victory = true;
            }
        }
    }

    pub fn draw(&mut self, _settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        let window_size = graphics::drawable_size(ctx);
        let identity_transform = graphics::transform(ctx);

        // Apply our custom transform
        MainState::apply_view_transform(ctx, data.world_to_screen);

        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        {
            draw_map_layer(
                &mut data.map_spritebatch,
                &data.map,
                &data.map_tile_image,
                "Background",
            );
            graphics::draw(ctx, &data.map_spritebatch, graphics::DrawParam::new()).unwrap();
            data.map_spritebatch.clear();
        }

        self.draw_bullets(data, ctx);

        {
            MainState::draw_wall_pieces(&data.wall_pieces, &data.world, &mut data.map_spritebatch);
            graphics::draw(ctx, &data.map_spritebatch, graphics::DrawParam::new()).unwrap();
            data.map_spritebatch.clear();
        }

        //data.player.draw(&mut data.character_spritebatch);
        data.player.draw();

        for enemy in &data.enemies {
            //let positional = enemy.positional();
            enemy.draw(ctx);
        }

        {
            let character_spritebatch = &mut *data.character_spritebatch.borrow_mut();
            graphics::draw(ctx, character_spritebatch, graphics::DrawParam::new()).unwrap();
            character_spritebatch.clear();
        }

        // Reset to identity transform for text and splash screen
        graphics::set_transform(ctx, identity_transform);
        graphics::apply_transformations(ctx).unwrap();

        let health_text = graphics::Text::new((
            format!("Health: {:.0}", data.player.health() * 100.0),
            data.font,
            64.0,
        ));

        let enemies_text =
            graphics::Text::new((format!("Enemies: {}", data.enemies.len()), data.font, 64.0));

        let mut height = 0.0;
        draw_shadowed_text(
            ctx,
            Point2::new(50.0, 20.0 + height),
            &health_text,
            Color::from((255, 255, 255, 255)),
        );
        height += 20.0 + health_text.height(ctx) as f32;
        draw_shadowed_text(
            ctx,
            Point2::new(50.0, 20.0 + height),
            &enemies_text,
            Color::from((255, 255, 255, 255)),
        );

        let text =
            graphics::Text::new((format!("Round {}", self.round_index + 1), data.font, 96.0));
        let text_width = text.width(ctx) as f32;
        let text_height = text.height(ctx) as f32;

        draw_shadowed_text(
            ctx,
            Point2::new(
                ((window_size.0 as f32 / 2.0) - (text_width / 2.0)) + 4.0,
                (window_size.1 as f32 - text_height - 20.0) + 4.0,
            ),
            &text,
            Color::from((255, 255, 255, 255)),
        );
    }

    pub fn draw_bullets(&mut self, data: &mut WorldData, ctx: &mut Context) {
        for bullet in data.bullets.iter() {
            data.bullet_batch
                .add(bullet.pos.position, 0.5, bullet.pos.rotation);
        }

        data.bullet_batch.draw_and_clear(ctx);
    }

    pub fn update_camera(
        &mut self,
        data: &mut WorldData,
        target: Positional,
        look_ahead: f32,
        stiffness: f32,
    ) {
        let mut pos = target.position.coords;
        pos += target.forward() * look_ahead;

        data.camera_pos = Vector2::lerp(&data.camera_pos.coords, &pos, stiffness).into();
    }

    pub fn calculate_view_transform(
        &mut self,
        data: &mut WorldData,
        ctx: &Context,
        origin: Point2,
        scale: f32,
    ) {
        let window_size = graphics::drawable_size(ctx);

        let viewport_transform = Matrix4::new_translation(&Vector3::new(
            window_size.0 as f32 * 0.5,
            window_size.1 as f32 * 0.5,
            0.0,
        )) * Matrix4::new_nonuniform_scaling(&Vector3::new(
            window_size.1 as f32 * 0.5,
            window_size.1 as f32 * 0.5,
            1.0,
        ));

        data.world_to_screen = viewport_transform
            * Matrix4::new_nonuniform_scaling(&Vector3::new(scale, -scale, 1.0))
            * Matrix4::new_translation(&Vector3::new(-origin.x, -origin.y, 0.0));

        data.screen_to_world = data.world_to_screen.try_inverse().unwrap();
    }

    pub fn handle_key(
        &mut self,
        _settings: &Settings,
        data: &mut WorldData,
        _ctx: &mut Context,
        keycode: KeyCode,
        value: bool,
    ) {
        match keycode {
            KeyCode::Key1 => data.player.set_visual(VisualState::Gun),
            KeyCode::Key2 => data.player.set_visual(VisualState::Hold),
            KeyCode::Key3 => data.player.set_visual(VisualState::Machine),
            KeyCode::Key4 => data.player.set_visual(VisualState::Reload),
            KeyCode::Key5 => data.player.set_visual(VisualState::Silencer),
            KeyCode::Key6 => data.player.set_visual(VisualState::Stand),
            KeyCode::Key7 => data.sounds.play_break1(),
            KeyCode::Key8 => data.sounds.play_break2(),
            KeyCode::Key9 => {
                if value {
                    data.sounds.play_taunt()
                }
            }
            KeyCode::Key0 => {
                if data.player.alive() {
                    data.player.damage(13.0);
                }
            }
            KeyCode::W | KeyCode::Up => data.player_input.up = value,
            KeyCode::A | KeyCode::Left => data.player_input.left = value,
            KeyCode::S | KeyCode::Down => data.player_input.down = value,
            KeyCode::D | KeyCode::Right => data.player_input.right = value,
            KeyCode::Back => data.strategic_view = value,
            KeyCode::Space => {
                if value {
                    self.victory = true;
                }
            }
            _ => (),
        }
    }

    pub fn mouse_motion_event(
        &mut self,
        data: &mut WorldData,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _xrel: f32,
        _yrel: f32,
    ) {
        data.player_input.aim_pos = px_to_world(data.screen_to_world, x, y);
    }

    pub fn mouse_button_down_event(
        &mut self,
        data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        data.player_input.shoot = true;
    }

    pub fn mouse_button_up_event(
        &mut self,
        data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        data.player_input.shoot = false;
    }
}

fn spawn_dozer(
    ctx: &mut Context,
    world: &mut World<f32>,
    engine_sound: audio::SoundData,
    image: Rc<graphics::Image>,
    pos: Point2,
    rotation: f32,
) -> Box<dyn Enemy> {
    let size = {
        let rad = 3.0 / 2.0;
        let size = Vector2::new(image.width() as f32, image.height() as f32);
        rad * size / size.x.min(size.y)
    };

    let geom = ShapeHandle::new(Cuboid::new(size));
    let inertia = geom.inertia(1.0);
    let center_of_mass = geom.center_of_mass();

    let pos = Isometry2::new(Vector2::new(pos.x, pos.y), rotation);
    let rb = world.add_rigid_body(pos, inertia, center_of_mass);

    let collider_handle = world.add_collider(
        COLLIDER_MARGIN,
        geom.clone(),
        rb,
        Isometry2::identity(),
        Material::new(0.3, 0.5),
    );

    let mut col_group = CollisionGroups::new();
    col_group.set_membership(&[GROUP_ENEMY]);
    world
        .collision_world_mut()
        .set_collision_groups(collider_handle, col_group);

    Box::new(Bulldozer::new(
        ctx,
        engine_sound,
        rb,
        image.clone(),
        Positional::default(),
        Some(Box::new(EnemyDozerBehavior::new())),
    ))
}
