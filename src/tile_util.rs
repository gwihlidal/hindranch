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
}

impl<'a> TileMapLayerView<'a> {
    pub fn iter(&'a self) -> TileMapLayerViewIterator<'a> {
        TileMapLayerViewIterator {
            view: self,
            x: self.start_x as i32 - 1,
            y: self.start_y as i32,
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
        let res = loop {
            self.x += 1;
            if self.x >= self.view.end_x as i32 {
                self.x = self.view.start_x as i32;
                self.y += 1;
            }

            if self.y < self.view.end_y as i32 {
                let tile = self.view.layer.tiles[(99 - self.y) as usize][self.x as usize];
                if tile != 0 {
                    break Some((self.x as u32, self.y as u32, tile - 1));
                }
            } else {
                break None;
            }
        };

        // TODO: get actual map size
        res.map(|(x, y, tile_id)| MapTile {
            pos: {
                let tile_width = 64; // TODO
                let tile_height = 64; // TODO
                let scale = 1.0 / tile_width as f32;

                let x = (x - self.view.start_x) * tile_width; // + offset_x as f32;
                let y = (y - self.view.start_y) * tile_height; // + offset_y as f32;

                Point2::new(x as f32 * scale, y as f32 * scale)
            },
            tile_id,
        })
    }
}
