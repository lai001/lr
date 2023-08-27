use super::{tile_index::TileIndex, virtual_texture_configuration::VirtualTextureConfiguration};
use crate::{
    file_manager::FileManager, mipmap_generator::MipmapGenerator, util,
    virtual_texture::tile_index::TileOffset,
};
use image::{GenericImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistentBlockImageInfo {
    pub tile_index: TileIndex,
    pub path: String,
}

pub struct BlockImageMakeResult {
    pub tile_index: TileIndex,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub struct BlockImage {
    persistent_block_image_infos: Vec<PersistentBlockImageInfo>,
    cache_images: std::collections::HashMap<TileIndex, Arc<image::RgbaImage>>,
    cache_textures: std::collections::HashMap<TileIndex, Arc<wgpu::Texture>>,
    virtual_texture_configuration: VirtualTextureConfiguration,
    id: u32,
}

impl BlockImage {
    pub fn new(
        file_path: &str,
        virtual_texture_configuration: VirtualTextureConfiguration,
        id: u32,
    ) -> BlockImage {
        if Self::is_generated(file_path) {
            let persistent_block_image_infos =
                Self::read_from_file(Self::get_json_file_path(file_path)).unwrap();
            BlockImage {
                persistent_block_image_infos,
                cache_images: std::collections::HashMap::new(),
                cache_textures: std::collections::HashMap::new(),
                virtual_texture_configuration,
                id,
            }
        } else {
            let results = Self::make(file_path, id, &virtual_texture_configuration).unwrap();
            let persistent_block_image_infos = Self::cache_to_disk(file_path, &results);
            BlockImage {
                persistent_block_image_infos,
                cache_images: std::collections::HashMap::new(),
                cache_textures: std::collections::HashMap::new(),
                virtual_texture_configuration,
                id,
            }
        }
    }

    fn read_from_file<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Vec<PersistentBlockImageInfo>, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let u = serde_json::from_reader(reader)?;
        Ok(u)
    }

    fn get_save_dir(file_path: &str) -> String {
        let file_path = std::path::Path::new(file_path);
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        let dir = format!(
            "{}/{}",
            FileManager::default()
                .lock()
                .unwrap()
                .get_intermediate_dir_path(),
            file_stem
        );
        dir
    }

    fn get_json_file_path(file_path: &str) -> String {
        let dir = Self::get_save_dir(file_path);
        let file_path = std::path::Path::new(file_path);
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        let save_path = format!("{}/{}.json", dir, file_stem);
        save_path
    }

    fn is_generated(file_path: &str) -> bool {
        let path = Self::get_json_file_path(file_path);
        std::path::Path::new(&path).exists()
    }

    pub fn make(
        file_path: &str,
        id: u32,
        virtual_texture_configuration: &VirtualTextureConfiguration,
    ) -> Option<Vec<BlockImageMakeResult>> {
        let tile_size = virtual_texture_configuration.tile_size;
        match image::open(file_path) {
            Ok(image) => {
                assert_eq!(image.width() % tile_size, 0);
                assert_eq!(image.height() % tile_size, 0);
                let mut results = vec![];
                let mut image = image.into_rgba8();
                let width_pages = image.width() / tile_size;
                let height_pages = image.height() / tile_size;
                let index = id * virtual_texture_configuration.physical_texture_size / tile_size;
                let mut virtual_offset: glam::UVec2 = glam::UVec2 { x: 0, y: 0 };
                virtual_offset.x =
                    index % (virtual_texture_configuration.virtual_texture_size / tile_size);
                virtual_offset.y =
                    index / (virtual_texture_configuration.virtual_texture_size / tile_size);
                for w in 0..width_pages {
                    for h in 0..height_pages {
                        let sub_image =
                            image.sub_image(w * tile_size, h * tile_size, tile_size, tile_size);

                        let mut images: Vec<image::DynamicImage> = vec![];
                        let dynamic_image = image::DynamicImage::ImageRgba8(sub_image.to_image());
                        images.append(&mut MipmapGenerator::generate_from_image_cpu(
                            &dynamic_image,
                            None,
                        ));
                        images.insert(0, dynamic_image);

                        for (index, image) in images.iter_mut().enumerate() {
                            results.push(BlockImageMakeResult {
                                image: image.to_rgba8(),
                                tile_index: TileIndex {
                                    tile_offset: TileOffset {
                                        x: virtual_offset.x as u16 + w as u16,
                                        y: virtual_offset.y as u16 + h as u16,
                                    },
                                    mipmap_level: index as u8,
                                },
                            });
                        }
                    }
                }

                return Some(results);
            }
            Err(error) => {
                log::warn!("{:?}", error);
                None
            }
        }
    }

    pub fn cache_to_disk(
        file_path: &str,
        results: &Vec<BlockImageMakeResult>,
    ) -> Vec<PersistentBlockImageInfo> {
        let dir = Self::get_save_dir(file_path);
        let _ = std::fs::create_dir(&dir);
        let mut infos: Vec<PersistentBlockImageInfo> = vec![];
        for result in results {
            let prefix = "block_";
            let name = format!(
                "{}_{}_{}",
                result.tile_index.tile_offset.x,
                result.tile_index.tile_offset.y,
                result.tile_index.mipmap_level
            );
            let save_path = format!("{}/{}{}.png", &dir, prefix, &name);
            infos.push(PersistentBlockImageInfo {
                path: save_path.clone(),
                tile_index: result.tile_index,
            });
            match result.image.save(save_path) {
                Ok(_) => {}
                Err(error) => panic!("{}", error),
            }
        }
        let json_str = serde_json::to_string(&infos).unwrap();

        let save_path = Self::get_json_file_path(file_path);
        std::fs::write(save_path, json_str).unwrap();
        infos
    }

    pub fn get_image(&mut self, tile_index: TileIndex) -> Option<Arc<image::RgbaImage>> {
        let key = tile_index;
        let cache_images = &mut self.cache_images;
        let is_contains_key = cache_images.contains_key(&key);
        if is_contains_key == false {
            for block_image_info in &self.persistent_block_image_infos {
                if block_image_info.tile_index == tile_index {
                    let image = image::open(&block_image_info.path).unwrap();
                    let image = image.to_rgba8();
                    self.cache_images.insert(key, Arc::new(image));
                }
            }
        }
        let image: Option<Arc<image::RgbaImage>> = self.cache_images.get(&key).cloned();
        match image {
            Some(image) => Some(image),
            None => {
                // log::warn!("Invalid tile index: {:?}", tile_index);
                None
            }
        }
    }

    pub fn get_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        tile_index: TileIndex,
    ) -> Option<Arc<wgpu::Texture>> {
        let key = tile_index;
        let cache_textures = &mut self.cache_textures;
        let is_contains_key = cache_textures.contains_key(&key);
        if is_contains_key == false {
            match self.get_image(tile_index) {
                Some(cache_image) => {
                    let texture =
                        util::texture2d_from_rgba_image(device, queue, cache_image.as_ref());
                    self.cache_textures.insert(key, Arc::new(texture));
                }
                None => {}
            }
        }
        let texture = self.cache_textures.get(&key);
        match texture {
            Some(texture) => Some(texture.clone()),
            None => {
                // log::warn!("Invalid tile index: {:?}", tile_index);
                None
            }
        }
    }

    pub fn retain(&mut self, keeping_pages: &Vec<TileIndex>) {
        self.cache_images
            .retain(|&key, _| keeping_pages.contains(&key));
        self.cache_textures
            .retain(|&key, _| keeping_pages.contains(&key));
    }
}
