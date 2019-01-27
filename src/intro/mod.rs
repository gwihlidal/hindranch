#![allow(unused_imports)]

use crate::{
    draw_map_layer, graphics, Characters, Color, Context, KeyCode, MainState, Matrix4, MouseButton,
    MusicTrack, PlayerInput, Point2, Positional, Rect, Settings, Vector2, Vector3, VoiceQueue,
    WorldData,
};

pub struct IntroPhase {
    pub first_update: bool,
    pub begin_game: bool,
    pub voice_queue: VoiceQueue,
    pub music_track: MusicTrack,
    pub player_pos: Positional,
    pub sheriff_pos: Positional,
    pub player_stand: (Rect, Vector2),
    pub sheriff_stand: (Rect, Vector2),
    pub sheriff_speaking: bool,
}

impl IntroPhase {
    pub fn new(ctx: &mut Context) -> Self {
        let characters = Characters::load(ctx);

        let player_entry = characters.get_entry("woman_green");
        let sheriff_entry = characters.get_entry("man_blue");

        let player_stand = characters.transform(&player_entry.stand);
        let sheriff_stand = characters.transform(&sheriff_entry.stand);

        let player_pos = Point2::new(9.120043, 3.124171);
        let player_rot = -0.000054519685;

        let sheriff_pos = Point2::new(11.19879, 3.0724447);
        let sheriff_rot = -3.138286;

        IntroPhase {
            first_update: true,
            begin_game: false,
            voice_queue: VoiceQueue::new(),
            music_track: MusicTrack::new("cantina", ctx),
            player_pos: Positional {
                position: player_pos,
                rotation: player_rot,
            },
            sheriff_pos: Positional {
                position: sheriff_pos,
                rotation: sheriff_rot,
            },
            player_stand,
            sheriff_stand,
            sheriff_speaking: true,
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            println!("STATE: Intro");
            data.player.input = PlayerInput::default();
            if settings.voice {
                self.voice_queue.enqueue("shout", ctx);
                self.voice_queue.enqueue("defiance", ctx);
            }
            self.first_update = false;
        }

        if settings.music && !self.music_track.playing() {
            self.music_track.play();
        }

        self.voice_queue.process();

        self.calculate_view_transform(data, &ctx, data.camera_pos, 0.1);

        if self.sheriff_speaking {
            self.update_camera(data, self.sheriff_pos, 0.0, 0.3);
        } else {
            self.update_camera(data, self.player_pos, 0.0, 0.3);
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
            draw_map_layer(
                &mut data.map_spritebatch,
                &data.map,
                &data.map_tile_image,
                "Walls",
            );
            graphics::draw(ctx, &data.map_spritebatch, graphics::DrawParam::new()).unwrap();
            data.map_spritebatch.clear();
        }

        data.character_spritebatch.add(
            graphics::DrawParam::new()
                .src(self.player_stand.0)
                .dest(self.player_pos.position - Vector2::new(0.5, 0.5))
                .scale(self.player_stand.1)
                .offset(Point2::new(0.5, 0.5))
                .rotation(self.player_pos.rotation),
        );

        data.character_spritebatch.add(
            graphics::DrawParam::new()
                .src(self.sheriff_stand.0)
                .dest(self.sheriff_pos.position - Vector2::new(0.5, 0.5))
                .scale(self.sheriff_stand.1)
                .offset(Point2::new(0.5, 0.5))
                .rotation(self.sheriff_pos.rotation),
        );

        graphics::draw(ctx, &data.character_spritebatch, graphics::DrawParam::new()).unwrap();
        data.character_spritebatch.clear();

        // Reset to identity transform for text and splash screen
        graphics::set_transform(ctx, identity_transform);
        graphics::apply_transformations(ctx).unwrap();

        let text = if self.sheriff_speaking {
            graphics::Text::new(("SHERIFF: This is the sheriff.", data.font, 96.0))
        } else {
            graphics::Text::new(("PLAYER: Get the fuck out of here.", data.font, 96.0))
        };

        //let text_width = text.width(ctx) as f32;
        let text_height = text.height(ctx) as f32;

        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Point2::new(
                    24.0,
                    (window_size.1 as f32 - text_height - 20.0) + 4.0,
                ))
                .color(Color::from((0, 0, 0, 255))),
        )
        .unwrap();

        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::new()
                .dest(Point2::new(20.0, window_size.1 as f32 - text_height - 20.0))
                .color(Color::from((255, 255, 255, 255))),
        )
        .unwrap();
    }

    pub fn handle_key(
        &mut self,
        _settings: &Settings,
        _data: &mut WorldData,
        _ctx: &mut Context,
        key_code: KeyCode,
        value: bool,
    ) {
        if key_code == KeyCode::Space && value {
            self.begin_game = true;
        } else if key_code == KeyCode::P {
            self.sheriff_speaking = !self.sheriff_speaking;
        }
    }

    pub fn mouse_motion_event(
        &mut self,
        _data: &mut WorldData,
        _ctx: &mut Context,
        _x: f32,
        _y: f32,
        _xrel: f32,
        _yrel: f32,
    ) {
        //
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
