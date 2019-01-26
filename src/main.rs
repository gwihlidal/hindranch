#![allow(dead_code)]

extern crate ggez;
extern crate rand;
extern crate toml;
#[macro_use]
extern crate serde_derive;

//#[macro_use]
//extern crate state_machine;

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::graphics;
#[allow(unused_imports)]
use ggez::graphics::{Color, Rect, Scale};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::timer;
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path::{self, Path};
use std::rc::Rc;

mod enemy;
mod music;
mod settings;
mod tile_util;
mod types;
mod voice;
mod characters;

use self::tile_util::*;
use self::types::*;

use na::Isometry2;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::algebra::Force2;
//use nphysics2d::joint::{CartesianConstraint, PrismaticConstraint, RevoluteConstraint};
use nphysics2d::object::{BodyHandle, Material, RigidBody};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;

const COLLIDER_MARGIN: f32 = 0.01;

#[derive(Debug, Clone)]
pub struct Positional {
    position: Point2,
    rotation: f32,
}

impl Positional {
    pub fn set_from_physics(&mut self, rigid_body: &RigidBody<f32>) {
        let pos = rigid_body.position();
        self.position = pos.translation.vector.into();
        self.rotation = pos.rotation.angle();
    }

    // Assumes sprites face up
    pub fn forward(&self) -> Vector2 {
        Vector2::new(-self.rotation.sin(), self.rotation.cos())
    }

    pub fn right(&self) -> Vector2 {
        let forward = self.forward();
        Vector2::new(forward.y, -forward.x)
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

#[allow(dead_code)]
struct WallPiece {
    tile_snip: Rect,
    rb: BodyHandle,
    hp: f32,
}

struct MainState {
    settings: settings::Settings,

    characters: characters::Characters,

    player_input: PlayerInput,

    a: i32,
    direction: i32,
    splash: graphics::Image,

    dozer_image: Rc<graphics::Image>,

    wall_pieces: Vec<WallPiece>,

    //text: graphics::Text,
    //bmptext: graphics::Text,
    //pixel_sized_text: graphics::Text,
    voice_queue: voice::VoiceQueue,
    music_track: Option<music::MusicTrack>,

    enemies: Vec<Box<dyn enemy::Enemy>>,

    world_to_screen: Matrix4,
    screen_to_world: Matrix4,

    map: tiled::Map,
    map_tile_image: graphics::Image,
    map_spritebatch: graphics::spritebatch::SpriteBatch,
    world: World<f32>,
}

pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl PlayerInput {
    pub fn new() -> PlayerInput {
        PlayerInput {
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}

impl Default for PlayerInput {
    fn default() -> Self {
        PlayerInput::new()
    }
}

fn spawn_dozer(
    world: &mut World<f32>,
    image: Rc<graphics::Image>,
    pos: Point2,
) -> Box<dyn enemy::Enemy> {
    let size = {
        let rad = 3.0 / 2.0;
        let size = Vector2::new(image.width() as f32, image.height() as f32);
        rad * size / size.x.min(size.y)
    };

    let geom = ShapeHandle::new(Cuboid::new(size));
    let inertia = geom.inertia(1.0);
    let center_of_mass = geom.center_of_mass();

    let pos = Isometry2::new(Vector2::new(pos.x, pos.y), na::zero());
    let rb = world.add_rigid_body(pos, inertia, center_of_mass);

    world.add_collider(
        COLLIDER_MARGIN,
        geom.clone(),
        rb,
        Isometry2::identity(),
        Material::new(0.3, 0.5),
    );

    Box::new(enemy::Bulldozer::new(
        rb,
        image.clone(),
        8.0, /* health */
        Positional::default(),
    ))
}

impl MainState {
    fn new(settings: settings::Settings, ctx: &mut Context) -> GameResult<MainState> {
        let characters = characters::Characters::load(ctx);
        
        let map = tiled::parse_file(&Path::new("resources/map.tmx")).unwrap();
        //println!("{:?}", map);

        let map_tile_image = graphics::Image::new(ctx, &map.tilesets[0].images[0].source)
            .expect("opening the tileset image");

        let mut world = World::new();
        world.set_timestep(1.0 / 60.0);

        let dozer_image = Rc::new(graphics::Image::new(ctx, "/dozer.png").unwrap());

        let dozer_0 = spawn_dozer(&mut world, dozer_image.clone(), Point2::new(0.5, -0.8));
        let dozer_1 = spawn_dozer(&mut world, dozer_image.clone(), Point2::new(2.5, -2.8));
        let dozer_2 = spawn_dozer(&mut world, dozer_image.clone(), Point2::new(1.5, -1.8));

        //let _bulldozer_1 = enemy::Bulldozer::new(dozer_image.clone(), 8.0, Positional::default());
        //let _sheriff = enemy::Sheriff::new(4.0, Positional::default());

        let mut enemies: Vec<Box<dyn enemy::Enemy>> = Vec::new();
        enemies.push(dozer_0);
        enemies.push(dozer_1);
        enemies.push(dozer_2);

        let splash = graphics::Image::new(ctx, "/splash/hindranch_0.png").unwrap();

        let map_spritebatch = graphics::spritebatch::SpriteBatch::new(map_tile_image.clone());

        let mut voice_queue = voice::VoiceQueue::new();
        if settings.voice {
            voice_queue.enqueue("shout", ctx);
            voice_queue.enqueue("defiance", ctx);
        }

        let mut music_track = music::MusicTrack::new("cantina", ctx);
        if settings.music {
            music_track.play();
        }

        let mut s = MainState {
            settings,
            characters,

            player_input: Default::default(),
            a: 0,
            direction: 1,
            splash,

            dozer_image,
            wall_pieces: Vec::new(),

            //text,
            //bmptext,
            //pixel_sized_text,
            voice_queue,
            music_track: Some(music_track),

            enemies,

            world_to_screen: Matrix4::identity(),
            screen_to_world: Matrix4::identity(),
            world,
            map,
            map_tile_image,
            map_spritebatch,
        };

        s.create_wall_pieces();

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

    fn tile_id_to_src_rect(tile: u32, map: &tiled::Map, image: &graphics::Image) -> Rect {
        let tile_width = map.tile_width;
        let tile_height = map.tile_height;

        let tile_w = tile_width as f32 / image.width() as f32;
        let tile_h = tile_height as f32 / image.height() as f32;

        let tile_column_count = (image.width() as usize) / (tile_width as usize);

        let tile_c = (tile as usize % tile_column_count) as f32;
        let tile_r = (tile as usize / tile_column_count) as f32;

        Rect::new(tile_w * tile_c, tile_h * tile_r, tile_w, tile_h)
    }

    fn get_map_layer<'a>(map: &'a tiled::Map, layer_name: &str) -> &'a tiled::Layer {
        let layer_idx = map
            .layers
            .iter()
            .position(|layer| layer.name == layer_name)
            .unwrap();

        &map.layers[layer_idx]
    }

    // Inspired by https://github.com/FloVanGH/pg-engine/blob/master/src/drawing.rs
    fn draw_map_layer(
        batch: &mut graphics::spritebatch::SpriteBatch,
        map: &tiled::Map,
        image: &graphics::Image,
        layer_name: &str,
    ) {
        //let map = &self.map;
        let layer = Self::get_map_layer(map, layer_name);

        let tile_width = map.tile_width;
        let scale = 1.0 / tile_width as f32;

        let start_column = 10;
        let start_row = 30;
        let end_column = 100; //end_column;
        let end_row = 100; //end_row;

        let view = TileMapLayerView {
            layer,
            start_x: start_column,
            end_x: end_column,
            start_y: start_row,
            end_y: end_row,
        };

        for MapTile { tile_id, pos } in view.iter() {
            let src = Self::tile_id_to_src_rect(tile_id, map, image);
            batch.add(
                graphics::DrawParam::new()
                    .src(src)
                    .dest(pos - Vector2::new(0.5, 0.5))
                    .scale(Vector2::new(scale, -scale))
                    .offset(Point2::new(0.5, 0.5)),
            );
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

    fn handle_key(&mut self, keycode: KeyCode, value: bool) {
        match keycode {
            KeyCode::W | KeyCode::Up => self.player_input.up = value,
            KeyCode::A | KeyCode::Left => self.player_input.left = value,
            KeyCode::S | KeyCode::Down => self.player_input.down = value,
            KeyCode::D | KeyCode::Right => self.player_input.right = value,
            _ => (),
        }
    }

    fn create_wall_pieces(&mut self) {
        // TODO: extents
        let view = TileMapLayerView {
            layer: Self::get_map_layer(&self.map, "Walls"),
            start_x: 10,
            end_x: 100,
            start_y: 30,
            end_y: 100,
        };

        for MapTile { tile_id, pos } in view.iter() {
            let src = Self::tile_id_to_src_rect(tile_id, &self.map, &self.map_tile_image);

            let rb = {
                let rad = 0.5 - COLLIDER_MARGIN;

                let geom = ShapeHandle::new(Cuboid::new(Vector2::repeat(rad)));
                let inertia = geom.inertia(1.0);
                let center_of_mass = geom.center_of_mass();

                let pos = Isometry2::new(pos.coords, na::zero());
                let rb = self.world.add_rigid_body(pos, inertia, center_of_mass);

                self.world.add_collider(
                    COLLIDER_MARGIN,
                    geom.clone(),
                    rb,
                    Isometry2::identity(),
                    Material::new(0.3, 0.5),
                );

                rb
            };

            self.wall_pieces.push(WallPiece {
                tile_snip: src,
                rb,
                hp: 100.0,
            });
        }
    }

    fn draw_wall_pieces(
        wall_pieces: &[WallPiece],
        world: &World<f32>,
        sprite_batch: &mut graphics::spritebatch::SpriteBatch,
    ) {
        for wall_piece in wall_pieces.iter() {
            let tile_width = 64; // TODO
            let scale = 1.0 / tile_width as f32;

            let (pos, rot): (Point2, f32) = {
                let positional = world.rigid_body(wall_piece.rb).unwrap().position();
                (
                    positional.translation.vector.into(),
                    positional.rotation.angle(),
                )
            };

            sprite_batch.add(
                graphics::DrawParam::new()
                    .src(wall_piece.tile_snip)
                    .dest(pos - Vector2::new(0.5, 0.5))
                    .scale(Vector2::new(scale, -scale))
                    .rotation(rot)
                    .offset(Point2::new(0.5, 0.5)),
            );
        }
    }
}

/*
fn angle_shortest_dist(a0: f32, a1: f32) -> f32 {
    let max = f32::consts::PI * 2.0;
    let da = (a1 - a0) % max;
    2.0 * da % max - da
}

fn calculate_torque_for_aim(aim: Vector2, rotation: f32, spin: f32) -> f32 {
    let target_rot = if aim.x == 0.0 && aim.y == 0.0 {
        rotation
    } else {
        (-aim.x).atan2(aim.y)
    };

    let angle_diff = angle_shortest_dist(rotation, target_rot);

    angle_diff * 200.0 - spin * 15.0
}*/

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.calculate_view_transform(&ctx, Point2::origin(), 0.1);

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

            for (i, enemy) in &mut self.enemies.iter_mut().enumerate() {
                if i == 0 {
                    // TODO: Player controlled hack
                    enemy.update(
                        Some(enemy::Movement {
                            left: self.player_input.left,
                            right: self.player_input.right,
                            up: self.player_input.up,
                            down: self.player_input.down,
                        }),
                        &mut self.world,
                    );
                } else {
                    enemy.update(None, &mut self.world);
                }
            }

            for wall_piece in self.wall_pieces.iter() {
                if let Some(rb) = self.world.rigid_body_mut(wall_piece.rb) {
                    let mut vel = rb.velocity().clone();
                    vel.linear *= 0.1;
                    vel.angular *= 0.1;
                    rb.set_velocity(vel);
                }
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

        //graphics::set_screen_coordinates(ctx, Rect::new_i32(0, 0, 960, 540))
        //.expect("Could not set logical screen coordinates before running initial state.");

        Self::draw_single_image(ctx, &self.splash, Point2::new(0.0, 0.0), 2.0, 0.0);

        Self::draw_map_layer(
            &mut self.map_spritebatch,
            &self.map,
            &self.map_tile_image,
            "Background",
        );

        Self::draw_wall_pieces(&self.wall_pieces, &self.world, &mut self.map_spritebatch);
        graphics::draw(ctx, &self.map_spritebatch, graphics::DrawParam::new()).unwrap();
        self.map_spritebatch.clear();

        for enemy in &self.enemies {
            let positional = enemy.positional();
            Self::draw_single_image(
                ctx,
                &enemy.image(),
                positional.position,
                3.0,
                positional.rotation,
            );
        }

        graphics::present(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        key_code: KeyCode,
        _key_mod: KeyMods,
        repeat: bool,
    ) {
        if repeat {
            return;
        }

        match key_code {
            KeyCode::M => {
                if let Some(ref mut track) = self.music_track {
                    track.stop();
                    self.music_track = None;
                } else {
                    let mut music_track = music::MusicTrack::new("twisted", ctx);
                    music_track.play();
                    self.music_track = Some(music_track);
                }
            }
            _ => self.handle_key(key_code, true),
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, key_code: KeyCode, _key_mod: KeyMods) {
        self.handle_key(key_code, false);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        println!("Resized screen to {}, {}", width, height);

        //if self.window_settings.resize_projection {
        let new_rect = graphics::Rect::new(
            0.0,
            0.0,
            width as f32,  // * self.zoom,
            height as f32, // * self.zoom,
        );
        graphics::set_screen_coordinates(ctx, new_rect).unwrap();
        //}
    }
}

pub fn resolution() -> (f32, f32) {
    if cfg!(target_os = "macos") {
        (1024.0, 768.0)
    } else {
        (1280.0, 720.0)
    }
}

pub fn main() -> GameResult {
    let settings = settings::load_settings();

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (width, height) = resolution();

    let cb = ggez::ContextBuilder::new("hindranch", "ggez")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup {
            title: "Hindranch v 3.74b".to_owned(),
            srgb: true,
            ..Default::default()
        })
        .window_mode(WindowMode {
            width,
            height,
            hidpi: false,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;

    println!("Renderer: {}", graphics::renderer_info(ctx).unwrap());

    let state = &mut MainState::new(settings, ctx)?;
    event::run(ctx, event_loop, state)
}
