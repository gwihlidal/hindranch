#![allow(unused_imports)]

use crate::{
    draw_map_layer, graphics, px_to_world, settings::Settings, Context, KeyCode, MainState,
    Matrix4, MouseButton, MusicTrack, Point2, Positional, Vector2, Vector3, VisualState, WorldData,
    DESIRED_FPS,
};

pub struct RoundPhase {
    music_track: MusicTrack,
}

impl RoundPhase {
    pub fn new(ctx: &mut Context) -> RoundPhase {
        RoundPhase {
            music_track: MusicTrack::new("twisted", ctx),
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if !self.music_track.playing() {
            self.music_track.play();
        }

        data.voice_queue.process();

        self.calculate_view_transform(
            data,
            &ctx,
            data.camera_pos,
            if data.strategic_view { 0.02 } else { 0.1 },
        );

        data.player.update(&settings, &mut data.world);

        data.player_weapon.update(
            data.player.input.shoot,
            &data.player.positional,
            &mut data.bullets,
        );

        for bullet in data.bullets.iter_mut() {
            bullet.pos.position += bullet.pos.forward() * (bullet.velocity / DESIRED_FPS as f32);
            bullet.life_seconds -= 1.0 / (DESIRED_FPS as f32);
        }

        data.bullets.retain(|b| b.life_seconds > 0.0);

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

        data.world.step();
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

        data.player.draw(&mut data.character_spritebatch);

        graphics::draw(ctx, &data.character_spritebatch, graphics::DrawParam::new()).unwrap();
        data.character_spritebatch.clear();

        for enemy in &data.enemies {
            let positional = enemy.positional();
            MainState::draw_single_image(
                ctx,
                &enemy.image(),
                positional.position,
                3.0,
                positional.rotation,
            );
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
        ctx: &mut Context,
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
                        // DEAD! :(
                        data.sounds.play_death();
                    }
                }
            }
            KeyCode::M => {
                if let Some(ref mut track) = data.music_track {
                    track.stop();
                    data.music_track = None;
                } else {
                    let mut music_track = MusicTrack::new("twisted", ctx);
                    music_track.play();
                    data.music_track = Some(music_track);
                }
            }
            KeyCode::W | KeyCode::Up => data.player.input.up = value,
            KeyCode::A | KeyCode::Left => data.player.input.left = value,
            KeyCode::S | KeyCode::Down => data.player.input.down = value,
            KeyCode::D | KeyCode::Right => data.player.input.right = value,
            KeyCode::Back => data.strategic_view = value,
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
