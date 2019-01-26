extern crate ggez;
extern crate rand;

use ggez::conf::WindowSetup;
use ggez::event;
use ggez::graphics;
#[allow(unused_imports)]
use ggez::graphics::{Color, Rect, Scale};
use ggez::timer;
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path::{self, Path};

mod voice;

#[allow(dead_code)]
type Point2 = na::Point2<f32>;
type Vector2 = na::Vector2<f32>;
type Vector3 = na::Vector3<f32>;
type Matrix4 = na::Matrix4<f32>;

use na::Isometry2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::algebra::Force2;
//use nphysics2d::joint::{CartesianConstraint, PrismaticConstraint, RevoluteConstraint};
use nphysics2d::object::{BodyHandle, Material, RigidBody};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;

const COLLIDER_MARGIN: f32 = 0.01;

struct Positional {
    position: Point2,
    rotation: f32,
}

impl Positional {
    fn set_from_physics(&mut self, rigid_body: &RigidBody<f32>) {
        let pos = rigid_body.position();
        self.position = pos.translation.vector.into();
        self.rotation = pos.rotation.angle();
    }
}

impl Default for Positional {
    fn default() -> Self {
        Self {
            position: Point2::origin(),
            rotation: 0.0,
        }
    }
}

struct MainState {
    a: i32,
    direction: i32,
    dragon: graphics::Image,
    dozer: graphics::Image,
    dozer_rb: BodyHandle,
    dozer_pos: Positional,
    //text: graphics::Text,
    //bmptext: graphics::Text,
    //pixel_sized_text: graphics::Text,
    voice_queue: voice::VoiceQueue,

    world_to_screen: Matrix4,
    screen_to_world: Matrix4,

    map: tiled::Map,
    map_tile_image: graphics::Image,
    world: World<f32>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let map = tiled::parse_file(&Path::new("resources/map.tmx")).unwrap();
        println!("{:?}", map);

        let map_tile_image = graphics::Image::new(ctx, &map.tilesets[0].images[0].source)
            .expect("opening the tileset image");

        let mut world = World::new();
        world.set_timestep(1.0 / 60.0);

        let dozer_rb;
        {
            let rad = 0.2;

            let geom = ShapeHandle::new(Cuboid::new(Vector2::repeat(rad)));
            let inertia = geom.inertia(1.0);
            let center_of_mass = geom.center_of_mass();

            let pos = Isometry2::new(Vector2::new(0.5, -0.8), na::zero());
            let rb = world.add_rigid_body(pos, inertia, center_of_mass);

            world.add_collider(
                COLLIDER_MARGIN,
                geom.clone(),
                rb,
                Isometry2::identity(),
                Material::new(0.3, 0.5),
            );

            dozer_rb = rb;
        }

        {
            let rad = 0.2;

            let geom = ShapeHandle::new(Cuboid::new(Vector2::repeat(rad)));
            let inertia = geom.inertia(1000.0);
            let center_of_mass = geom.center_of_mass();

            let pos = Isometry2::new(Vector2::new(0.68, 0.8), na::zero());
            let rb = world.add_rigid_body(pos, inertia, center_of_mass);

            world.add_collider(
                COLLIDER_MARGIN,
                geom.clone(),
                rb,
                Isometry2::identity(),
                Material::new(0.3, 0.5),
            );
        }

        let dragon = graphics::Image::new(ctx, "/dragon1.png").unwrap();
        let dozer = graphics::Image::new(ctx, "/dozer.png").unwrap();

        /*let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48).unwrap();
        let text = graphics::Text::new(ctx, "Hello world!", &font).unwrap();
        let bmpfont =
            graphics::Font::new_bitmap(ctx, "/arial.png", "ABCDEFGHIJKLMNOPQRSTUVWXYZ").unwrap();
        let bmptext = graphics::Text::new(ctx, "ZYXWVYTSRQPONMLKJIHGFEDCBA", &bmpfont).unwrap();*/

        /*let pixel_font = graphics::Font::new_px(ctx, "/DejaVuSerif.ttf", 32).unwrap();
        let pixel_sized_text =
            graphics::Text::new(ctx, "This text is 32 pixels high", &pixel_font).unwrap();*/

        let mut voice_queue = voice::VoiceQueue::new();
        voice_queue.enqueue("shout", ctx);
        voice_queue.enqueue("defiance", ctx);

        let s = MainState {
            a: 0,
            direction: 1,
            dragon,
            dozer,
            dozer_rb,
            dozer_pos: Positional::default(),
            //text,
            //bmptext,
            //pixel_sized_text,
            voice_queue,

            world_to_screen: Matrix4::identity(),
            screen_to_world: Matrix4::identity(),
            world,
            map,
            map_tile_image,
        };

        Ok(s)
    }

    pub fn calculate_view_transform(&mut self, ctx: &Context, origin: Point2, scale: f32) {
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

    // Inspired by https://github.com/FloVanGH/pg-engine/blob/master/src/drawing.rs
    // TODO: add batching
    fn draw_map_layer(&self, ctx: &mut Context, layer_name: &str) {
        let map = &self.map;
        let layer_idx = map
            .layers
            .iter()
            .position(|layer| layer.name == layer_name)
            .unwrap();

        let scale = 0.001;

        let tile_width = map.tile_width;
        let tile_height = map.tile_height;

        let image = &self.map_tile_image;

        let start_column = 10;
        let start_row = 30;
        let end_column = 40; //end_column;
        let end_row = 60; //end_row;

        /*if end_column > map.column_count() {
            end_column = map.column_count();
        }

        if end_row > map.row_count() {
            end_row = map.row_count();
        }*/

        let layer = &map.layers[layer_idx];
        let tile_column_count = (image.width() as usize) / (tile_width as usize);

        let tile_w = tile_width as f32 / image.width() as f32;
        let tile_h = tile_height as f32 / image.height() as f32;

        for r in start_row..end_row {
            for c in start_column..end_column {
                let tile = layer.tiles[99 - r][c];
                if 0 == tile {
                    continue;
                }
                let tile = tile - 1; // HACK: assumes one tileset with first id=1

                let x = (c - start_column) * map.tile_width as usize; // + offset_x as f32;
                let y = (r - start_row) * map.tile_height as usize; // + offset_y as f32;

                let tile_c = (tile as usize % tile_column_count) as f32;
                let tile_r = (tile as usize / tile_column_count) as f32;

                let draw_param = graphics::DrawParam::new()
                    .src(Rect::new(tile_w * tile_c, tile_h * tile_r, tile_w, tile_h))
                    .dest(Point2::new(x as f32 * scale, y as f32 * scale))
                    .scale(Vector2::new(scale, -scale));

                graphics::draw(ctx, image, draw_param).unwrap();
            }
        }
    }

    fn draw_single_image(
        ctx: &mut Context,
        image: &graphics::Image,
        pos: Point2,
        scale: f32,
        rotation: f32,
    ) {
        let min_extent = image.width().min(image.height());
        let half_w = 0.5 * scale / min_extent as f32;
        let half_h = 0.5 * scale / min_extent as f32;
        graphics::draw(
            ctx,
            image,
            graphics::DrawParam::new()
                .dest(pos - Vector2::new(0.5, 0.5))
                .scale(Vector2::new(half_w * 2.0, half_h * -2.0))
                .offset(Point2::new(0.5, 0.5))
                .rotation(rotation),
        )
        .unwrap();
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.calculate_view_transform(&ctx, Point2::origin(), 1.0);

        const DESIRED_FPS: u32 = 60;
        //let dt = 1.0 / (DESIRED_FPS as f32);

        self.voice_queue.process();

        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.a += self.direction;
            if self.a > 250 || self.a <= 0 {
                self.direction *= -1;

                println!("Delta frame time: {:?} ", timer::delta(ctx));
                println!("Average FPS: {}", timer::fps(ctx));
            }

            {
                let rigid_body = self.world.rigid_body_mut(self.dozer_rb).unwrap();
                self.dozer_pos.set_from_physics(rigid_body);
                rigid_body.apply_force(&Force2::linear(Vector2::new(0.0, 0.1)));
            }

            self.world.step();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.apply_view_transform(ctx);

        //let c = self.a as u8;
        //graphics::set_color(ctx, Color::from((c, c, c, 255)))?;
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        self.draw_map_layer(ctx, "Background");
        self.draw_map_layer(ctx, "Walls");

        Self::draw_single_image(ctx, &self.dragon, Point2::new(0.0, 0.0), 1.0, 0.0);
        Self::draw_single_image(
            ctx,
            &self.dozer,
            self.dozer_pos.position,
            0.5,
            self.dozer_pos.rotation,
        );

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

        graphics::present(ctx)?;

        timer::yield_now();
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("hindranch", "ggez")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup {
            title: "Hindranch v 3.74b".to_owned(),
            srgb: true,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
