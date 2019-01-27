#![allow(unused_imports)]

use crate::{
    audio::Source, draw_map_layer, graphics, graphics::Text, Characters, Color, Context, KeyCode,
    MainState, Matrix4, MouseButton, MusicTrack, PawnInput, PlayerInput, Point2, Positional, Rect,
    Settings, Vector2, Vector3, VoiceQueue, WorldData,
};

pub struct IntroLine {
    pub voice_source: Source,
    pub voice_line: Text,
    pub sheriff_speaking: bool,
    pub started: bool,
}

pub struct IntroPhase {
    pub first_update: bool,
    pub begin_game: bool,
    pub voice_queue: VoiceQueue,
    pub music_track: MusicTrack,
    pub player_pos: Positional,
    pub sheriff_pos: Positional,
    pub player_stand: (Rect, Vector2),
    pub sheriff_stand: (Rect, Vector2),
    pub intro_lines: Vec<IntroLine>,
    pub intro_line: u32,
}

impl IntroPhase {
    pub fn new(data: &mut WorldData, ctx: &mut Context) -> Self {
        let characters = Characters::load(ctx);

        let player_entry = characters.get_entry("woman_green");
        let sheriff_entry = characters.get_entry("man_blue");

        let player_stand = characters.transform(&player_entry.stand);
        let sheriff_stand = characters.transform(&sheriff_entry.stand);

        let player_pos = Point2::new(9.120043, 3.124171);
        let player_rot = -0.000054519685;

        let sheriff_pos = Point2::new(11.19879, 3.0724447);
        let sheriff_rot = -3.138286;

        let mut lines: Vec<IntroLine> = Vec::new();

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_1.ogg").unwrap(),
            voice_line: Text::new(("Ma'am, this is sheriff Buck.", data.font, 54.0)),
            sheriff_speaking: true,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_2.ogg").unwrap(),
            voice_line: Text::new((
                "I'm here to inform you that the bank has foreclosed on your ranch.",
                data.font,
                54.0,
            )),
            sheriff_speaking: true,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_3.ogg").unwrap(),
            voice_line: Text::new(("What in the hell?", data.font, 54.0)),
            sheriff_speaking: false,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_4.ogg").unwrap(),
            voice_line: Text::new((
                "My family has lived here for generations; it's my home!",
                data.font,
                54.0,
            )),
            sheriff_speaking: false,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_5.ogg").unwrap(),
            voice_line: Text::new(("I'm sorry, it's not your home anymore.", data.font, 54.0)),
            sheriff_speaking: true,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_6.ogg").unwrap(),
            voice_line: Text::new(("The hell it isn't!", data.font, 54.0)),
            sheriff_speaking: false,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_7.ogg").unwrap(),
            voice_line: Text::new((
                "You think you can come on to my property and take what is mine?",
                data.font,
                54.0,
            )),
            sheriff_speaking: false,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_8.ogg").unwrap(),
            voice_line: Text::new((
                "Yes, you have one day until we evict you by force.",
                data.font,
                54.0,
            )),
            sheriff_speaking: true,
            started: false,
        });

        lines.push(IntroLine {
            voice_source: Source::new(ctx, "/voice/intro_9.ogg").unwrap(),
            voice_line: Text::new(("I'd like to see you fucking try!", data.font, 54.0)),
            sheriff_speaking: false,
            started: false,
        });

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
            intro_lines: lines,
            intro_line: 0,
        }
    }

    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        if self.first_update {
            data.player_input = PlayerInput::default();
            if settings.voice {
                //self.voice_queue.enqueue("shout", ctx);
                //self.voice_queue.enqueue("defiance", ctx);
            }
            self.first_update = false;
        }

        if settings.music && !self.music_track.playing() {
            self.music_track.play();
        }

        self.music_track.volume(0.2);

        self.voice_queue.process();

        self.calculate_view_transform(data, &ctx, data.camera_pos, 0.1);

        if self.intro_line < self.intro_lines.len() as u32 {
            let sheriff_speaking = { self.intro_lines[self.intro_line as usize].sheriff_speaking };

            if sheriff_speaking {
                self.update_camera(data, self.sheriff_pos, 0.0, 0.3);
            } else {
                self.update_camera(data, self.player_pos, 0.0, 0.3);
            }

            let mut line = &mut self.intro_lines[self.intro_line as usize];
            if !line.started {
                line.voice_source.play().unwrap();
                line.started = true;
            } else {
                if !line.voice_source.playing() {
                    self.intro_line += 1;
                }
            }
        } else {
            self.begin_game = true;
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

        let character_spritebatch = &mut *data.character_spritebatch.borrow_mut();

        character_spritebatch.add(
            graphics::DrawParam::new()
                .src(self.player_stand.0)
                .dest(self.player_pos.position - Vector2::new(0.5, 0.5))
                .scale(self.player_stand.1)
                .offset(Point2::new(0.5, 0.5))
                .rotation(self.player_pos.rotation),
        );

        character_spritebatch.add(
            graphics::DrawParam::new()
                .src(self.sheriff_stand.0)
                .dest(self.sheriff_pos.position - Vector2::new(0.5, 0.5))
                .scale(self.sheriff_stand.1)
                .offset(Point2::new(0.5, 0.5))
                .rotation(self.sheriff_pos.rotation),
        );

        graphics::draw(ctx, character_spritebatch, graphics::DrawParam::new()).unwrap();
        character_spritebatch.clear();

        // Reset to identity transform for text and splash screen
        graphics::set_transform(ctx, identity_transform);
        graphics::apply_transformations(ctx).unwrap();

        if self.intro_line < self.intro_lines.len() as u32 {
            let line = &self.intro_lines[self.intro_line as usize];

            let text_width = line.voice_line.width(ctx) as f32;
            let text_height = line.voice_line.height(ctx) as f32;

            graphics::draw(
                ctx,
                &line.voice_line,
                graphics::DrawParam::new()
                    .dest(Point2::new(
                        ((window_size.0 as f32 / 2.0) - (text_width / 2.0)) + 4.0,
                        (window_size.1 as f32 - text_height - 20.0) + 4.0,
                    ))
                    .color(Color::from((0, 0, 0, 255))),
            )
            .unwrap();

            graphics::draw(
                ctx,
                &line.voice_line,
                graphics::DrawParam::new()
                    .dest(Point2::new(
                        (window_size.0 as f32 / 2.0) - (text_width / 2.0),
                        window_size.1 as f32 - text_height - 20.0,
                    ))
                    .color(Color::from((255, 255, 255, 255))),
            )
            .unwrap();
        }
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
            //self.sheriff_speaking = !self.sheriff_speaking;
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
