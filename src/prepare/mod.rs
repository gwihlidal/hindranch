#![allow(unused_imports)]

use crate::{graphics, Context, draw_map_layer, MainState, KeyCode, Positional, px_to_world, Point2, Vector2, Vector3, Matrix4, MouseButton, PlayerInput, RoundData, Settings, WorldData};
use std::cell::RefCell;
use std::rc::Rc;

pub struct PreparePhase {
    pub first_update: bool,
    pub round_index: u32,
    pub last_round: bool,
    pub begin_round: bool,
    pub round_data: Rc<RefCell<RoundData>>,
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
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            println!(
                "STATE: Prepare - round_index: {}, last_round: {}",
                self.round_index, self.last_round
            );
            data.player.input = PlayerInput::default();
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

        data.player.update(&settings, &mut data.world);

        self.update_camera(data, data.player.positional, 0.0, 0.3);

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

        {
            MainState::draw_wall_pieces(&data.wall_pieces, &data.world, &mut data.map_spritebatch);
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
    }

    pub fn handle_key(
        &mut self,
        _settings: &Settings,
        data: &mut WorldData,
        _ctx: &mut Context,
        key_code: KeyCode,
        value: bool,
    ) {
        match key_code {
            KeyCode::W | KeyCode::Up => data.player.input.up = value,
            KeyCode::A | KeyCode::Left => data.player.input.left = value,
            KeyCode::S | KeyCode::Down => data.player.input.down = value,
            KeyCode::D | KeyCode::Right => data.player.input.right = value,
            KeyCode::Back => data.strategic_view = value,
            KeyCode::Space => {
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
        data.player.input.aim_pos = px_to_world(data.screen_to_world, x, y);
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
}
