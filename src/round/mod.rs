#![allow(unused_imports)]

use super::consts::*;
use crate::{
    draw_map_layer, graphics, px_to_world, settings::Settings, AiBehavior, Bulldozer, Context,
    Enemy, EnemyDozerBehavior, KeyCode, MainState, Matrix4, MouseButton, MusicTrack, Point2,
    Positional, RoundData, Vector2, Vector3, VisualState, WorldData, DESIRED_FPS,
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
            println!(
                "STATE: Round - round_index: {}, last_round: {}",
                self.round_index, self.last_round
            );
            self.first_update = false;

            if settings.enemies {
                println!("spawn_bulldozers");
                self.spawn_bulldozers(data, ctx, 8);
            }
        }

        let round_data = self.round_data.clone();
        let mut round_data = round_data.borrow_mut();
        if !round_data.music_track.playing() {
            round_data.music_track.play();
        }

        self.calculate_view_transform(
            data,
            &ctx,
            data.camera_pos,
            if data.strategic_view { 0.02 } else { 0.1 },
        );

        data.player.update(&settings, &mut data.world);

        for (i, enemy) in &mut data.enemies.iter_mut().enumerate() {
            if settings.dozer_drive && i == 0 {
                // TODO: Player controlled hack
                enemy.update(
                    enemy.positional(),
                    Some((&data.player.input).into()),
                    &mut data.world,
                );
            } else {
                enemy.update(data.player.positional, None, &mut data.world);
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

        data.world.step();
    }

    fn maintain_weapons(&mut self, data: &mut WorldData) {
        data.player.weapon.update(
            data.player.input.shoot,
            &data.player.positional,
            &mut data.bullets,
        );

        let collision_world = data.world.collision_world();
        for bullet in data.bullets.iter_mut() {
            let mut enemy_hit = None;

            let mut hit_anything = false;
            let mut groups = CollisionGroups::new();
            groups.set_blacklist(&[COLLISION_GROUP_PLAYER]);
            for (other_collider, collision) in collision_world.interferences_with_ray(
                &Ray {
                    origin: bullet.pos.position,
                    dir: bullet.pos.forward(),
                },
                &groups,
            ) {
                if collision.toi < bullet.velocity / 60.0 {
                    let other_body = data.world.collider_body_handle(other_collider.handle());
                    for (enemy_i, enemy) in data.enemies.iter().enumerate() {
                        if enemy.rigid_body() == other_body {
                            enemy_hit = Some(enemy_i);
                            //println!("Enemy hit!");
                        }
                    }

                    hit_anything = true;
                    break;
                }
            }

            if hit_anything {
                bullet.life_seconds = 0.0;
            }

            if let Some(enemy_i) = enemy_hit {
                data.enemies[enemy_i].damage(bullet.damage);
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
            // TODO: play a happy sound!
            data.sounds.play_death();
            if data.enemies.is_empty() {
                self.victory = true;
            }
        }
    }

    pub fn draw(&mut self, _settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
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

        //Self::draw_single_image(ctx, &self.splash, Point2::new(0.0, 0.0), 2.0, 0.0);

        let text2 = graphics::Text::new((
            format!("Health: {:.0}", data.player.health()),
            data.font,
            48.0,
        ));

        let mut height = 0.0;
        //for (_key, text) in &self.texts {
        graphics::queue_text(ctx, &data.text, Point2::new(20.0, 20.0 + height), None);
        height += 20.0 + data.text.height(ctx) as f32;

        graphics::queue_text(ctx, &text2, Point2::new(20.0, 20.0 + height), None);
        //height += 20.0 + text2.height(ctx) as f32;
        //}
        // When drawing via `draw_queued()`, `.offset` in `DrawParam` will be
        // in screen coordinates, and `.color` will be ignored.
        graphics::draw_queued_text(ctx, graphics::DrawParam::default()).unwrap();

        /*graphics::draw(
            ctx,
            &self.text,
            graphics::DrawParam::new()
                .dest(Point2::new(10.0, 10.0))
                .color(Color::from((0, 0, 0, 255))),
        )?;*/
    }

    pub fn draw_bullets(&mut self, data: &mut WorldData, ctx: &mut Context) {
        for bullet in data.bullets.iter() {
            data.bullet_batch
                .add(bullet.pos.position, 1.0, bullet.pos.rotation);
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
            KeyCode::Key0 => {
                if data.player.alive() {
                    data.player.damage(13.0);
                    if !data.player.alive() {
                        self.failure = true;
                    }
                }
            }
            KeyCode::W | KeyCode::Up => data.player.input.up = value,
            KeyCode::A | KeyCode::Left => data.player.input.left = value,
            KeyCode::S | KeyCode::Down => data.player.input.down = value,
            KeyCode::D | KeyCode::Right => data.player.input.right = value,
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
        data.player.input.aim_pos = px_to_world(data.screen_to_world, x, y);
    }

    pub fn mouse_button_down_event(
        &mut self,
        data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        data.player.input.shoot = true;
    }

    pub fn mouse_button_up_event(
        &mut self,
        data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        data.player.input.shoot = false;
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
    col_group.set_membership(&[COLLISION_GROUP_ENEMY]);
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
