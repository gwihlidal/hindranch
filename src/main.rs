#![allow(dead_code)]

extern crate ggez;
extern crate rand;
extern crate toml;
#[macro_use]
extern crate serde_derive;

use ggez::audio;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, MouseButton};
use ggez::graphics;
#[allow(unused_imports)]
use ggez::graphics::{spritebatch::SpriteBatch, Color, DrawParam, Image, Rect, Scale};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::timer;
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path::{self, Path};
use std::rc::Rc;

mod ai;
mod characters;
mod consts;
mod enemy;
mod music;
mod player;
mod settings;
mod sounds;
mod tile_util;
mod types;
mod voice;
mod weapon;

mod dead;
mod intro;
mod menu;
mod outro;
mod prepare;
mod round;

use self::ai::*;
use self::characters::*;
use self::consts::*;
use self::enemy::*;
use self::music::*;
use self::player::*;
use self::settings::*;
use self::sounds::*;
use self::tile_util::*;
use self::types::*;
use self::weapon::*;

use self::dead::*;
use self::intro::*;
use self::menu::*;
use self::outro::*;
use self::prepare::*;
use self::round::*;

use na::Isometry2;
use ncollide2d::shape::{Ball, Cuboid, ShapeHandle};
use ncollide2d::world::CollisionGroups;
use nphysics2d::algebra::Force2;
use nphysics2d::force_generator::{ForceGeneratorHandle, Spring};
use nphysics2d::object::{BodyHandle, Material};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::world::World;

pub const DESIRED_FPS: u32 = 60;

enum Phase {
    Dead(DeadPhase),
    Intro(IntroPhase),
    Menu(MenuPhase),
    Outro(OutroPhase),
    Prepare(PreparePhase),
    Round(RoundPhase),
}

pub struct WorldData {
    world_to_screen: Matrix4,
    screen_to_world: Matrix4,
    map: tiled::Map,
    map_tile_image: graphics::Image,
    map_spritebatch: graphics::spritebatch::SpriteBatch,
    bullets: Vec<Bullet>,
    bullet_batch: SingleImageSpriteBatch,
    wall_pieces: Vec<WallPiece>,
    world: World<f32>,
    engine_data: audio::SoundData,
    font: graphics::Font,
    text: graphics::Text,
    sounds: Sounds,
    characters: Characters,
    player: Player,
    player_weapon: Weapon,
    splash: graphics::Image,
    dozer_image: Rc<graphics::Image>,
    voice_queue: voice::VoiceQueue,
    music_track: Option<music::MusicTrack>,
    enemies: Vec<Box<dyn Enemy>>,
    camera_pos: Point2,
    strategic_view: bool,
    character_spritebatch: graphics::spritebatch::SpriteBatch,
}

impl WorldData {
    pub fn new(settings: settings::Settings, ctx: &mut Context) -> Self {
        let map = tiled::parse_file(&Path::new("resources/map.tmx")).unwrap();
        let map_tile_image =
            Image::new(ctx, &map.tilesets[0].images[0].source).expect("opening the tileset image");
        let map_spritebatch = graphics::spritebatch::SpriteBatch::new(map_tile_image.clone());

        let mut world = World::new();
        world.set_timestep(1.0 / 60.0);

        let characters = Characters::load(ctx);

        let dozer_image = Rc::new(graphics::Image::new(ctx, "/dozer_lores.png").unwrap());

        let engine_sound = audio::SoundData::new(ctx, "/sound/bulldozer3.ogg").unwrap();

        let splash = graphics::Image::new(ctx, "/splash/hindranch_0.png").unwrap();

        let character_spritebatch =
            graphics::spritebatch::SpriteBatch::new(characters.image.clone());

        let mut voice_queue = voice::VoiceQueue::new();
        if settings.voice {
            voice_queue.enqueue("shout", ctx);
            voice_queue.enqueue("defiance", ctx);
        }

        let mut music_track = music::MusicTrack::new("cantina", ctx);
        if settings.music {
            music_track.play();
        }

        let health = 100.0;
        let player = Player::new(
            &mut world,
            "woman_green",
            health,
            Point2::new(0.5, 0.5),
            &characters,
        );

        let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf").unwrap();
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        WorldData {
            world_to_screen: Matrix4::identity(),
            screen_to_world: Matrix4::identity(),
            map,
            map_tile_image,
            map_spritebatch,
            bullets: Vec::new(),
            bullet_batch: SingleImageSpriteBatch::new(ctx, "/bullet.png"),
            wall_pieces: Vec::new(),
            world,
            font,
            text,
            engine_data: engine_sound.clone(),
            sounds: Sounds::load(ctx),
            characters,
            player,
            player_weapon: Weapon::from_config(WeaponConfig::from_toml("resources/shotgun.toml")),
            splash,
            dozer_image,
            voice_queue,
            music_track: Some(music_track),
            enemies: Vec::new(),
            camera_pos: Point2::origin(),
            strategic_view: false,
            character_spritebatch,
        }
    }
}

impl From<&PlayerInput> for Movement {
    fn from(i: &PlayerInput) -> Self {
        Self {
            forward: (if i.up { 1.0 } else { 0.0 }) + (if i.down { -1.0 } else { 0.0 }),
            right: (if i.right { 1.0 } else { 0.0 }) + (if i.left { -1.0 } else { 0.0 }),
        }
    }
}

#[allow(dead_code)]
struct WallPiece {
    tile_snip: Rect,
    rb: BodyHandle,
    spring: ForceGeneratorHandle,
    hp: f32,
}

struct SingleImageSpriteBatch {
    batch: graphics::spritebatch::SpriteBatch,
    image_dims: (u32, u32),
}

impl SingleImageSpriteBatch {
    fn new(ctx: &mut Context, path: &str) -> Self {
        let image = graphics::Image::new(ctx, path).expect(&format!("opening image: {}", path));
        let image_dims = (image.width() as u32, image.height() as u32);
        let batch = graphics::spritebatch::SpriteBatch::new(image);

        Self { batch, image_dims }
    }

    fn add(&mut self, pos: Point2, scl: f32, rot: f32) {
        let min_extent = self.image_dims.0.min(self.image_dims.1);
        let half_w = 0.5 * scl / min_extent as f32;
        let half_h = 0.5 * scl / min_extent as f32;
        self.batch.add(
            graphics::DrawParam::new()
                .dest(pos - Vector2::new(0.5, 0.5))
                .scale(Vector2::new(half_w * 2.0, half_h * -2.0))
                .offset(Point2::new(0.5, 0.5))
                .rotation(rot),
        );
    }

    fn draw_and_clear(&mut self, ctx: &mut Context) {
        graphics::draw(ctx, &self.batch, graphics::DrawParam::new()).unwrap();
        self.batch.clear();
    }
}

struct MainState {
    world_data: WorldData,
    settings: settings::Settings,
    phase: Phase,
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

impl MainState {
    fn new(settings: settings::Settings, ctx: &mut Context) -> GameResult<MainState> {
        let mut s = MainState {
            world_data: WorldData::new(settings.clone(), ctx),
            settings: settings.clone(),
            phase: Phase::Round(RoundPhase::new(ctx)),
        };

        s.spawn_wall_pieces();

        if settings.enemies {
            s.spawn_bulldozers(ctx, 8);
        }

        Ok(s)
    }

    fn spawn_bulldozers(&mut self, ctx: &mut Context, count: usize) {
        let a_off = rand::random::<f32>() * std::f32::consts::PI;

        // Stratified circular positioning
        for i in 0..count {
            let amin = i as f32 / count as f32;
            let amax = (i + 1) as f32 / count as f32;
            let a =
                a_off + (amin + (amax - amin) * rand::random::<f32>()) * std::f32::consts::PI * 2.0;

            const SPAWN_DIST: f32 = DOZER_OUTER_RADIUS;

            let dozer_0 = spawn_dozer(
                ctx,
                &mut self.world_data.world,
                self.world_data.engine_data.clone(),
                self.world_data.dozer_image.clone(),
                Point2::new(a.cos() * SPAWN_DIST, a.sin() * SPAWN_DIST),
                std::f32::consts::PI + a,
            );
            self.world_data.enemies.push(dozer_0);
        }
    }

    /// Apply the calculated view transform to the current graphics context
    pub fn apply_view_transform(ctx: &mut Context, world_to_screen: Matrix4) {
        graphics::set_transform(ctx, world_to_screen);
        graphics::apply_transformations(ctx).unwrap();
    }

    fn draw_single_image(
        ctx: &mut Context,
        image: &graphics::Image,
        color: Color,
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
                .color(color)
                .rotation(rotation),
        )
        .unwrap();
    }

    fn spawn_wall_pieces(&mut self) {
        let view = TileMapLayerView::new(get_map_layer(&self.world_data.map, "Walls"));

        for MapTile { tile_id, pos } in view.iter() {
            let src = tile_id_to_src_rect(
                tile_id,
                &self.world_data.map,
                &self.world_data.map_tile_image,
            );

            let rb = {
                let rad = 0.5 - COLLIDER_MARGIN;

                // Sim as balls for less coupling between elements
                //let geom = ShapeHandle::new(Ball::new(rad));
                let geom = ShapeHandle::new(Cuboid::new(Vector2::new(rad, rad)));

                let inertia = geom.inertia(10.0);
                let center_of_mass = geom.center_of_mass();

                let pos = Isometry2::new(pos.coords, na::zero());
                let rb = self
                    .world_data
                    .world
                    .add_rigid_body(pos, inertia, center_of_mass);

                let collider_handle = self.world_data.world.add_collider(
                    COLLIDER_MARGIN,
                    geom.clone(),
                    rb,
                    Isometry2::identity(),
                    Material::new(0.3, 0.0),
                );

                let mut col_group = CollisionGroups::new();
                col_group.set_membership(&[COLLISION_GROUP_WORLD]);
                self.world_data
                    .world
                    .collision_world_mut()
                    .set_collision_groups(collider_handle, col_group);

                rb
            };

            let spring = self.world_data.world.add_force_generator(Spring::new(
                BodyHandle::ground(),
                rb,
                pos,
                Point2::origin(),
                0.0,
                100.0,
            ));

            self.world_data.wall_pieces.push(WallPiece {
                tile_snip: src,
                rb,
                spring,
                hp: 1.0,
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

            let color = graphics::Color {
                r: 1.0,
                g: wall_piece.hp,
                b: wall_piece.hp,
                a: 1.0,
            };

            sprite_batch.add(
                graphics::DrawParam::new()
                    .src(wall_piece.tile_snip)
                    .dest(pos - Vector2::new(0.5, 0.5))
                    .scale(Vector2::new(scale, -scale))
                    .rotation(rot)
                    .offset(Point2::new(0.5, 0.5))
                    .color(color),
            );
        }
    }

    fn wall_velocity_to_damage(vel: &Vector2) -> f32 {
        (0.1 * (vel.norm() - 4.0)).max(0.0)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, DESIRED_FPS) {
            match self.phase {
                Phase::Dead(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
                Phase::Intro(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
                Phase::Menu(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
                Phase::Outro(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
                Phase::Prepare(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
                Phase::Round(ref mut phase) => {
                    phase.update(&self.settings, &mut self.world_data, ctx)
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.phase {
            Phase::Dead(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
            Phase::Intro(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
            Phase::Menu(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
            Phase::Outro(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
            Phase::Prepare(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
            Phase::Round(ref mut phase) => phase.draw(&self.settings, &mut self.world_data, ctx),
        }

        graphics::present(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, xrel: f32, yrel: f32) {
        match self.phase {
            Phase::Dead(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
            Phase::Intro(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
            Phase::Menu(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
            Phase::Outro(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
            Phase::Prepare(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
            Phase::Round(ref mut phase) => {
                phase.mouse_motion_event(&mut self.world_data, ctx, x, y, xrel, yrel)
            }
        }
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        match self.phase {
            Phase::Dead(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Intro(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Menu(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Outro(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Prepare(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Round(ref mut phase) => {
                phase.mouse_button_down_event(&mut self.world_data, ctx, button, x, y)
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        match self.phase {
            Phase::Dead(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Intro(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Menu(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Outro(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Prepare(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
            Phase::Round(ref mut phase) => {
                phase.mouse_button_up_event(&mut self.world_data, ctx, button, x, y)
            }
        }
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

        match self.phase {
            Phase::Dead(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
            Phase::Intro(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
            Phase::Menu(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
            Phase::Outro(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
            Phase::Prepare(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
            Phase::Round(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, true)
            }
        }
    }

    fn key_up_event(&mut self, ctx: &mut Context, key_code: KeyCode, _key_mod: KeyMods) {
        match self.phase {
            Phase::Dead(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
            Phase::Intro(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
            Phase::Menu(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
            Phase::Outro(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
            Phase::Prepare(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
            Phase::Round(ref mut phase) => {
                phase.handle_key(&self.settings, &mut self.world_data, ctx, key_code, false)
            }
        }
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
