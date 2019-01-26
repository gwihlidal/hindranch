use super::types::*;

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
