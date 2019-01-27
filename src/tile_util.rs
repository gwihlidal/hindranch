use super::types::*;
use crate::{DrawParam, Image, Rect, SpriteBatch};
use nalgebra as na;

pub struct TileMapLayerView<'a> {
    pub layer: &'a tiled::Layer,
    pub start_x: u32,
    pub end_x: u32,
    pub start_y: u32,
    pub end_y: u32,
}

pub struct TileMapLayerViewIterator<'a> {
    pub view: &'a TileMapLayerView<'a>,
    pub x: i32,
    pub y: i32,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl<'a> TileMapLayerView<'a> {
    pub fn new(layer: &'a tiled::Layer) -> Self {
        Self {
            layer,
            start_x: 0,
            end_x: layer.tiles[0].len() as u32,
            start_y: 0,
            end_y: layer.tiles.len() as u32,
        }
    }

    pub fn iter(&'a self) -> TileMapLayerViewIterator<'a> {
        TileMapLayerViewIterator {
            view: self,
            x: self.start_x as i32 - 1,
            y: self.start_y as i32,
            x_offset: self.layer.tiles[0].len() as f32 * -0.5,
            y_offset: self.layer.tiles.len() as f32 * -0.5,
        }
    }
}

#[derive(Clone)]
pub struct MapTile {
    pub tile_id: u32,
    pub pos: Point2,
}

impl<'a> Iterator for TileMapLayerViewIterator<'a> {
    type Item = MapTile;

    fn next(&mut self) -> Option<MapTile> {
        let map_size_y = self.view.layer.tiles.len();

        let res = loop {
            self.x += 1;
            if self.x >= self.view.end_x as i32 {
                self.x = self.view.start_x as i32;
                self.y += 1;
            }

            if self.y < self.view.end_y as i32 {
                let tile = self.view.layer.tiles[map_size_y - 1 - self.y as usize][self.x as usize];
                if tile != 0 {
                    break Some((self.x as f32, self.y as f32, tile - 1));
                }
            } else {
                break None;
            }
        };

        // TODO: get actual map size
        res.map(|(x, y, tile_id)| MapTile {
            pos: {
                let tile_size = 64.0; // TODO
                let scale = 1.0 / tile_size;

                let x = (x + self.x_offset) * tile_size;
                let y = (y + self.y_offset) * tile_size;

                Point2::new(x as f32 * scale, y as f32 * scale)
            },
            tile_id,
        })
    }
}

pub fn tile_id_to_src_rect(tile: u32, map: &tiled::Map, image: &Image) -> Rect {
    let tile_width = map.tile_width;
    let tile_height = map.tile_height;

    let tile_w = tile_width as f32 / image.width() as f32;
    let tile_h = tile_height as f32 / image.height() as f32;

    let tile_column_count = (image.width() as usize) / (tile_width as usize);

    let tile_c = (tile as usize % tile_column_count) as f32;
    let tile_r = (tile as usize / tile_column_count) as f32;

    Rect::new(tile_w * tile_c, tile_h * tile_r, tile_w, tile_h)
}

pub fn get_map_layer<'a>(map: &'a tiled::Map, layer_name: &str) -> &'a tiled::Layer {
    let layer_idx = map
        .layers
        .iter()
        .position(|layer| layer.name == layer_name)
        .unwrap();

    &map.layers[layer_idx]
}

// Inspired by https://github.com/FloVanGH/pg-engine/blob/master/src/drawing.rs
pub fn draw_map_layer(batch: &mut SpriteBatch, map: &tiled::Map, image: &Image, layer_name: &str) {
    //let map = &self.map;
    let layer = get_map_layer(map, layer_name);

    let tile_width = map.tile_width;
    let scale = 1.0 / tile_width as f32;

    let start_column = 0;
    let start_row = 0;
    let end_column = map.width;
    let end_row = map.height;

    // TODO: figure out the extents to draw
    let view = TileMapLayerView {
        layer,
        start_x: start_column,
        end_x: end_column,
        start_y: start_row,
        end_y: end_row,
    };

    for MapTile { tile_id, pos } in view.iter() {
        let src = tile_id_to_src_rect(tile_id, map, image);
        batch.add(
            DrawParam::new()
                .src(src)
                .dest(pos - Vector2::new(0.5, 0.5))
                .scale(Vector2::new(scale, -scale))
                .offset(Point2::new(0.5, 0.5)),
        );
    }
}

pub fn px_to_world(screen_to_world: Matrix4, x: f32, y: f32) -> Point2 {
    (screen_to_world * na::Vector4::new(x, y, 0.0, 1.0))
        .xy()
        .into()
}
