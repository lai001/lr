use super::{tile_index::TileIndex, virtual_texture_configuration::VirtualTextureConfiguration};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy)]
pub struct ArrayTile {
    pub index: u32,
    pub page_size: u32,
    pub offset_x: u32,
    pub offset_y: u32,
}

impl ArrayTile {
    pub fn put(&self, box_count: usize, box_size: u32) -> (Vec<ArrayTile>, usize) {
        debug_assert_eq!(self.page_size / box_size, 2);
        let mut result: Vec<ArrayTile> = vec![];

        for index in 0..box_count.min(4) {
            let tile = ArrayTile {
                index: self.index,
                page_size: box_size,
                offset_x: self.offset_x + (index % 2 * box_size as usize) as u32,
                offset_y: self.offset_y + (index / 2 * box_size as usize) as u32,
            };
            result.push(tile);
        }
        let remain = 4 - result.len();
        (result, remain)
    }

    pub fn max_split_cout(page_size: u32, sub_size: u32) -> usize {
        let div = page_size / sub_size;
        (div * div) as usize
    }

    pub fn split(&self, sub_size: u32) -> Vec<ArrayTile> {
        let mut result: Vec<ArrayTile> = vec![];
        let div = self.page_size / sub_size;
        // let sub_size: u32 = self.page_size / div;
        for index in 0..div * div {
            let tile = ArrayTile {
                index: self.index,
                page_size: sub_size,
                offset_x: self.offset_x + (index % div * sub_size) as u32,
                offset_y: self.offset_y + (index / div * sub_size) as u32,
            };
            result.push(tile);
        }
        result
    }
}

pub struct Packing {
    pub virtual_texture_configuration: VirtualTextureConfiguration,
}

impl Packing {
    fn page_size(size: u32, level: u8) -> u32 {
        u32::max(1, size >> level)
    }

    pub fn pack(&self, tile_index: &Vec<TileIndex>) -> HashMap<TileIndex, ArrayTile> {
        let mut tile_index_map: HashMap<u8, Vec<TileIndex>> = HashMap::new();

        for tile in tile_index {
            if tile_index_map.contains_key(&tile.mipmap_level) {
                tile_index_map
                    .get_mut(&tile.mipmap_level)
                    .unwrap()
                    .push(*tile);
            } else {
                tile_index_map.insert(tile.mipmap_level, vec![*tile]);
            }
        }

        let mut mip_levels: Vec<u8> = tile_index_map.keys().map(|x| *x).collect();
        mip_levels.sort();

        let all: Vec<ArrayTile> = (0..self
            .virtual_texture_configuration
            .physical_texture_array_size)
            .flat_map(|index| {
                let t = ArrayTile {
                    index,
                    page_size: self.virtual_texture_configuration.physical_texture_size,
                    offset_x: 0,
                    offset_y: 0,
                };
                let result = t.split(self.virtual_texture_configuration.tile_size);
                result
            })
            .collect();

        let mut all = VecDeque::from(all);

        let mut final_result: HashMap<TileIndex, ArrayTile> = HashMap::new();

        for mip_level in mip_levels {
            let page_size =
                Self::page_size(self.virtual_texture_configuration.tile_size, mip_level);
            let virtual_index_group = tile_index_map.get_mut(&mip_level).unwrap();
            virtual_index_group.sort_by(|left, right| {
                let h = self.virtual_texture_configuration.virtual_texture_size
                    / self.virtual_texture_configuration.tile_size;
                let a = left.tile_offset.y * h as u16 + left.tile_offset.x;
                let b = right.tile_offset.y * h as u16 + right.tile_offset.x;
                a.cmp(&b)
            });

            let max_split_cout =
                ArrayTile::max_split_cout(self.virtual_texture_configuration.tile_size, page_size);

            let pop_count =
                (virtual_index_group.len() as f32 / max_split_cout as f32).ceil() as usize;

            let sub_array_tile: Vec<ArrayTile> = (0..pop_count)
                .flat_map(|_| match all.pop_front() {
                    Some(array_tile) => array_tile.split(page_size),
                    None => Vec::new(),
                })
                .collect();

            for (i, virtual_index) in virtual_index_group.iter().enumerate() {
                match sub_array_tile.get(i) {
                    Some(sub_tile) => {
                        final_result.insert(*virtual_index, *sub_tile);
                    }
                    None => {
                        panic!("Increase physical map size or num.")
                    }
                }
            }
        }

        final_result
    }
}
