use crate::{
    draw_map_layer, draw_shadowed_text, get_map_layer, graphics, px_to_world, tile_id_to_src_rect,
    BodyHandle, CollisionGroups, Color, Context, Cuboid, Isometry2, KeyCode, MainState, Material,
    Matrix4, MouseButton, PlayerInput, Point2, Positional, RoundData, Settings, ShapeHandle,
    Spring, TileMapLayerView, Vector2, Vector3, VisualState, Volumetric, WallPiece, WorldData,
    COLLIDER_MARGIN, GROUP_WORLD,
};

use nalgebra as na;
use rand::{thread_rng, Rng};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct PreparePhase {
    pub first_update: bool,
    pub round_index: u32,
    pub last_round: bool,
    pub begin_round: bool,
    pub round_data: Rc<RefCell<RoundData>>,
    pub crate_supplies: u32,
    pub rock_supplies: u32,
    pub voice_played: bool,
    pub play_voice_at: Instant,
}

impl PreparePhase {
    pub fn new(
        _ctx: &mut Context,
        round_index: u32,
        last_round: bool,
        round_data: Rc<RefCell<RoundData>>,
    ) -> Self {
        PreparePhase {
            first_update: true,
            round_index,
            last_round,
            begin_round: false,
            round_data,
            crate_supplies: 0,
            rock_supplies: 0,
            voice_played: false,
            play_voice_at: Instant::now() + Duration::from_millis(1500),
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            data.player_input = PlayerInput::default();
            data.player.set_visual(VisualState::Hold);

            let (crate_count, rock_count) = match self.round_index {
                0 => (settings.round1_crates, settings.round1_rocks),
                1 => (settings.round2_crates, settings.round2_rocks),
                2 => (settings.round3_crates, settings.round3_rocks),
                3 => (settings.round4_crates, settings.round4_rocks),
                4 => (settings.round5_crates, settings.round5_rocks),
                _ => unimplemented!(),
            };
            self.crate_supplies = crate_count;
            self.rock_supplies = rock_count;
            self.first_update = false;
        }

        if !self.voice_played && Instant::now() >= self.play_voice_at {
            data.sounds.play_prepare(self.round_index as usize);
            self.voice_played = true;
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

        self.update_camera(data, data.player.positional, 0.0, 0.3);

        data.maintain_walls();

        data.world.step();

        if self.crate_supplies == 0 && self.rock_supplies == 0 {
            self.begin_round = true;
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

        {
            MainState::draw_wall_pieces(&data.wall_pieces, &data.world, &mut data.map_spritebatch);
            graphics::draw(ctx, &data.map_spritebatch, graphics::DrawParam::new()).unwrap();
            data.map_spritebatch.clear();
        }

        {
            draw_map_layer(
                &mut data.map_spritebatch,
                &data.map,
                &data.map_tile_image,
                "Props",
            );
            graphics::draw(ctx, &data.map_spritebatch, graphics::DrawParam::new()).unwrap();
            data.map_spritebatch.clear();
        }

        data.player.draw();

        {
            let character_spritebatch = &mut *data.character_spritebatch.borrow_mut();
            graphics::draw(ctx, character_spritebatch, graphics::DrawParam::new()).unwrap();
            character_spritebatch.clear();
        }

        // Reset to identity transform for text and splash screen
        graphics::set_transform(ctx, identity_transform);
        graphics::apply_transformations(ctx).unwrap();

        let crates_text =
            graphics::Text::new((format!("Crates: {}", self.crate_supplies), data.font, 64.0));
        let rocks_text =
            graphics::Text::new((format!("Rocks: {}", self.rock_supplies), data.font, 64.0));

        let mut height = 0.0;
        draw_shadowed_text(
            ctx,
            Point2::new(50.0, 20.0 + height),
            &crates_text,
            if self.crate_supplies > 0 {
                Color::from((255, 255, 255, 255))
            } else {
                Color::from((255, 0, 0, 255))
            },
        );
        height += 20.0 + crates_text.height(ctx) as f32;
        draw_shadowed_text(
            ctx,
            Point2::new(50.0, 20.0 + height),
            &rocks_text,
            if self.rock_supplies > 0 {
                Color::from((255, 255, 255, 255))
            } else {
                Color::from((255, 0, 0, 255))
            },
        );

        let text = graphics::Text::new(("Prepare!", data.font, 96.0));
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

    pub fn handle_key(
        &mut self,
        _settings: &Settings,
        data: &mut WorldData,
        ctx: &mut Context,
        key_code: KeyCode,
        value: bool,
    ) {
        match key_code {
            KeyCode::W | KeyCode::Up => data.player_input.up = value,
            KeyCode::A | KeyCode::Left => data.player_input.left = value,
            KeyCode::S | KeyCode::Down => data.player_input.down = value,
            KeyCode::D | KeyCode::Right => data.player_input.right = value,
            KeyCode::C => {
                if value && self.crate_supplies > 0 {
                    let player_velocity = data
                        .world
                        .rigid_body(data.player.body_handle)
                        .unwrap()
                        .velocity()
                        .linear;

                    // Try to place behind player
                    let place_offset = if player_velocity.norm() > 1e-5 {
                        player_velocity.normalize() * -0.5
                    } else {
                        Vector2::zeros()
                    };

                    self.place_crate(data.player.positional.position + place_offset, data, ctx);
                    self.crate_supplies -= 1;
                }
            }
            KeyCode::R => {
                if value && self.rock_supplies > 0 {
                    let player_velocity = data
                        .world
                        .rigid_body(data.player.body_handle)
                        .unwrap()
                        .velocity()
                        .linear;

                    // Try to place behind player
                    let place_offset = if player_velocity.norm() > 1e-5 {
                        player_velocity.normalize() * -0.5
                    } else {
                        Vector2::zeros()
                    };

                    self.place_rock(data.player.positional.position + place_offset, data, ctx);
                    self.rock_supplies -= 1;
                }
            }
            KeyCode::Back => data.strategic_view = value,
            KeyCode::Tab => {
                if value {
                    self.begin_round = true;
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
        _data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        //
    }

    pub fn mouse_button_up_event(
        &mut self,
        _data: &mut WorldData,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        //
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

    pub fn place_rock(&mut self, pos: Point2, data: &mut WorldData, _ctx: &mut Context) {
        //let view = TileMapLayerView::new(get_map_layer(&data.map, "Props"));

        data.sounds.play_break2();

        let mut rng = thread_rng();
        let tile_id = 236 + rng.gen_range(0, 3);
        let src = tile_id_to_src_rect(tile_id, &data.map, &data.map_tile_image);

        let rb = {
            let rad = 0.5 - COLLIDER_MARGIN;

            // Sim as balls for less coupling between elements
            //let geom = ShapeHandle::new(Ball::new(rad));
            let geom = ShapeHandle::new(Cuboid::new(Vector2::new(rad, rad)));

            let inertia = geom.inertia(10.0);
            let center_of_mass = geom.center_of_mass();

            let pos = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
            let rb = data.world.add_rigid_body(pos, inertia, center_of_mass);

            let collider_handle = data.world.add_collider(
                COLLIDER_MARGIN,
                geom.clone(),
                rb,
                Isometry2::identity(),
                Material::new(0.3, 0.0),
            );

            let mut col_group = CollisionGroups::new();
            col_group.set_membership(&[GROUP_WORLD]);
            data.world
                .collision_world_mut()
                .set_collision_groups(collider_handle, col_group);

            rb
        };

        let spring = data.world.add_force_generator(Spring::new(
            BodyHandle::ground(),
            rb,
            pos,
            Point2::origin(),
            0.0,
            100.0,
        ));

        data.wall_pieces.push(WallPiece {
            tile_snip: src,
            rb,
            spring,
            hp: 1.0,
        });
    }

    pub fn place_crate(&mut self, pos: Point2, data: &mut WorldData, _ctx: &mut Context) {
        //let view = TileMapLayerView::new(get_map_layer(&data.map, "Props"));

        data.sounds.play_break1();

        let tile_id = 128;
        let src = tile_id_to_src_rect(tile_id, &data.map, &data.map_tile_image);

        let rb = {
            let rad = 0.5 - COLLIDER_MARGIN;

            // Sim as balls for less coupling between elements
            //let geom = ShapeHandle::new(Ball::new(rad));
            let geom = ShapeHandle::new(Cuboid::new(Vector2::new(rad, rad)));

            let inertia = geom.inertia(10.0);
            let center_of_mass = geom.center_of_mass();

            let pos = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
            let rb = data.world.add_rigid_body(pos, inertia, center_of_mass);

            let collider_handle = data.world.add_collider(
                COLLIDER_MARGIN,
                geom.clone(),
                rb,
                Isometry2::identity(),
                Material::new(0.3, 0.0),
            );

            let mut col_group = CollisionGroups::new();
            col_group.set_membership(&[GROUP_WORLD]);
            data.world
                .collision_world_mut()
                .set_collision_groups(collider_handle, col_group);

            rb
        };

        let spring = data.world.add_force_generator(Spring::new(
            BodyHandle::ground(),
            rb,
            pos,
            Point2::origin(),
            0.0,
            100.0,
        ));

        data.wall_pieces.push(WallPiece {
            tile_snip: src,
            rb,
            spring,
            hp: 1.0,
        });
    }
}
