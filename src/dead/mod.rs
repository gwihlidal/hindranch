use crate::{
    audio, graphics, Color, Context, DrawParam, KeyCode, MouseButton, PlayerInput, Point2,
    Settings, Sounds, WorldData,
};

pub struct DeadPhase {
    pub sounds: Sounds,
    pub first_update: bool,
    pub want_restart: bool,
    pub wasted: audio::Source,
}

impl DeadPhase {
    pub fn new(ctx: &mut Context) -> Self {
        DeadPhase {
            sounds: Sounds::load(ctx),
            first_update: true,
            want_restart: false,
            wasted: audio::Source::new(ctx, "/voice/wasted.ogg").unwrap(),
        }
    }
    pub fn update(&mut self, settings: &Settings, data: &mut WorldData, _ctx: &mut Context) {
        if self.first_update {
            data.player_input = PlayerInput::default();
            if settings.sounds {
                self.sounds.play_death();
                self.wasted.play().unwrap();
            }
            self.first_update = false;
        }
    }

    pub fn draw(&mut self, _settings: &Settings, data: &mut WorldData, ctx: &mut Context) {
        let window_size = graphics::drawable_size(ctx);

        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let text = graphics::Text::new(("WASTED", data.font, 256.0));

        let text_width = text.width(ctx) as f32;
        let text_height = text.height(ctx) as f32;

        graphics::draw(
            ctx,
            &text,
            DrawParam::new()
                .dest(Point2::new(
                    (window_size.0 as f32 / 2.0) - (text_width / 2.0),
                    (window_size.1 as f32 / 2.0) - (text_height / 2.0),
                ))
                .color(Color::from((255, 0, 0, 255))),
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
            self.want_restart = true;
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
}
