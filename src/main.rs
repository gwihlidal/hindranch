extern crate ggez;
extern crate rand;

use ggez::audio;
use ggez::conf;
use ggez::event;
use ggez::graphics;
#[allow(unused_imports)]
use ggez::graphics::{Color, Point2, Vector2};
use ggez::timer;
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path;

#[allow(dead_code)]
type Vector3 = na::Vector3<f32>;
type Matrix4 = na::Matrix4<f32>;

struct MainState {
    a: i32,
    direction: i32,
    image: graphics::Image,
    text: graphics::Text,
    bmptext: graphics::Text,
    pixel_sized_text: graphics::Text,
    // Not actually dead, see BUGGO below
    #[allow(dead_code)]
    sound: audio::Source,

    world_to_screen: Matrix4,
    screen_to_world: Matrix4,

    derp_rot: f32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        ctx.print_resource_stats();

        let image = graphics::Image::new(ctx, "/dragon1.png").unwrap();

        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
        let bmpfont =
            graphics::Font::new_bitmap(ctx, "/arial.png", "ABCDEFGHIJKLMNOPQRSTUVWXYZ").unwrap();
        let bmptext = graphics::Text::new(ctx, "ZYXWVYTSRQPONMLKJIHGFEDCBA", &bmpfont).unwrap();
        let sound = audio::Source::new(ctx, "/sound.ogg").unwrap();

        let pixel_font = graphics::Font::new_px(ctx, "/DejaVuSerif.ttf", 32).unwrap();
        let pixel_sized_text =
            graphics::Text::new(ctx, "This text is 32 pixels high", &pixel_font).unwrap();

        //let _ = sound.play();

        let s = MainState {
            a: 0,
            direction: 1,
            image,
            text,
            bmptext,
            pixel_sized_text,
            // BUGGO: We never use sound again,
            // but we have to hang on to it, Or Else!
            // The optimizer will decide we don't need it
            // since play() has "no side effects" and free it.
            // Or something.
            sound,
            world_to_screen: Matrix4::identity(),
            screen_to_world: Matrix4::identity(),
            derp_rot: 0.0,
        };

        Ok(s)
    }

    pub fn calculate_view_transform(&mut self, ctx: &Context, origin: Point2, scale: f32) {
        let window_size = graphics::get_size(ctx);

        let viewport_transform = Matrix4::new_translation(&Vector3::new(
            window_size.0 as f32 * 0.5,
            window_size.1 as f32 * 0.5,
            0.0,
        )) * Matrix4::new_nonuniform_scaling(&Vector3::new(
            window_size.1 as f32 * 0.5,
            window_size.1 as f32 * 0.5,
            1.0,
        ));

        self.world_to_screen = viewport_transform
            * Matrix4::new_nonuniform_scaling(&Vector3::new(scale, -scale, 1.0))
            * Matrix4::new_translation(&Vector3::new(-origin.x, -origin.y, 0.0));

        self.screen_to_world = self.world_to_screen.try_inverse().unwrap();
    }

    /// Apply the calculated view transform to the current graphics context
    pub fn apply_view_transform(&self, ctx: &mut Context) {
        graphics::set_transform(ctx, self.world_to_screen);
        graphics::apply_transformations(ctx).unwrap();
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dt = 1.0 / 60.0;
        self.calculate_view_transform(&ctx, Point2::origin(), 1.0);

        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.a += self.direction;
            if self.a > 250 || self.a <= 0 {
                self.direction *= -1;

                println!("Delta frame time: {:?} ", timer::get_delta(ctx));
                println!("Average FPS: {}", timer::get_fps(ctx));
            }
        }

        self.derp_rot += dt;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.apply_view_transform(ctx);

        let c = self.a as u8;
        graphics::set_color(ctx, Color::from((c, c, c, 255)))?;
        graphics::clear(ctx);

        let dest_point = graphics::Point2::new(0.0, 0.0);
        let half_w = 0.5 / self.image.width() as f32;
        let half_h = 0.5 / self.image.height() as f32;
        graphics::draw_ex(
            ctx,
            &self.image,
            graphics::DrawParam {
                dest: dest_point - Vector2::new(0.5, 0.5),
                scale: graphics::Point2::new(half_w * 2.0, half_h * -2.0),
                offset: Point2::new(0.5, 0.5),
                rotation: self.derp_rot,
                ..Default::default()
            },
        )?;

        /*graphics::draw(ctx, &self.text, dest_point, 0.0)?;
        let dest_point = graphics::Point2::new(100.0, 50.0);
        graphics::draw(ctx, &self.bmptext, dest_point, 0.0)?;

        let dest_point2 = graphics::Point2::new(0.0, 256.0);
        graphics::set_color(ctx, Color::from((0, 0, 0, 255)))?;
        graphics::rectangle(
            ctx,
            graphics::DrawMode::Fill,
            graphics::Rect::new(0.0, 256.0, 500.0, 32.0),
        )?;
        graphics::set_color(ctx, Color::from((255, 255, 255, 255)))?;
        graphics::draw(ctx, &self.pixel_sized_text, dest_point2, 0.0)?;*/

        graphics::present(ctx);

        timer::yield_now();
        Ok(())
    }
}

pub fn main() {
    let c = conf::Conf::new();
    println!("Starting with default config: {:#?}", c);
    let ctx = &mut Context::load_from_conf("imageview", "ggez", c).unwrap();

    // We add the CARGO_MANIFEST_DIR/resources do the filesystems paths so
    // we we look in the cargo project for files.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        ctx.filesystem.mount(&path, true);
    }

    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}
