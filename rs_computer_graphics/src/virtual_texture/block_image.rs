use crate::file_manager::FileManager;
use image::{GenericImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PersistentBlockImageInfo {
    pub x: u32,
    pub y: u32,
    pub path: String,
}

pub struct BlockImageMakeResult {
    pub x: u32,
    pub y: u32,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
}

pub struct BlockImage {
    persistent_block_image_infos: Vec<PersistentBlockImageInfo>,
    cache_images: std::collections::HashMap<(u32, u32), ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

impl BlockImage {
    pub fn new(file_path: &str) -> BlockImage {
        if Self::is_generated(file_path) {
            let persistent_block_image_infos =
                Self::read_from_file(Self::get_json_file_path(file_path)).unwrap();
            BlockImage {
                persistent_block_image_infos,
                cache_images: std::collections::HashMap::new(),
            }
        } else {
            let results = Self::make(file_path, 256).unwrap();
            let persistent_block_image_infos = Self::cache_to_disk(file_path, &results);
            BlockImage {
                persistent_block_image_infos,
                cache_images: std::collections::HashMap::new(),
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

    pub fn make(file_path: &str, tile_size: u32) -> Option<Vec<BlockImageMakeResult>> {
        assert_eq!(tile_size, 256);
        match image::open(file_path) {
            Ok(image) => {
                assert_eq!(image.width() % tile_size, 0);
                assert_eq!(image.height() % tile_size, 0);
                let mut results = vec![];
                let mut image = image.into_rgba8();
                let width_pages = image.width() / tile_size;
                let height_pages = image.height() / tile_size;
                for w in 0..width_pages {
                    for h in 0..height_pages {
                        let sub_image =
                            image.sub_image(w * tile_size, h * tile_size, tile_size, tile_size);
                        let sub_image_copy = sub_image.to_image();
                        results.push(BlockImageMakeResult {
                            x: w,
                            y: h,
                            image: sub_image_copy,
                        });
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
            let name = format!("{}_{}", result.x, result.y);
            let save_path = format!("{}/{}{}.png", &dir, prefix, &name);
            infos.push(PersistentBlockImageInfo {
                x: result.x,
                y: result.y,
                path: save_path.clone(),
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

    pub fn get_image(&mut self, x: u32, y: u32) -> Option<&ImageBuffer<Rgba<u8>, Vec<u8>>> {
        let key = (x, y);
        let cache_images = &mut self.cache_images;
        let is_contains_key = cache_images.contains_key(&key);
        if is_contains_key == false {
            for block_image_info in &self.persistent_block_image_infos {
                if block_image_info.x == x && block_image_info.y == y {
                    let image = image::open(&block_image_info.path).unwrap();
                    let image = image.to_rgba8();
                    self.cache_images.insert(key, image);
                }
            }
        }
        let image: Option<&ImageBuffer<Rgba<u8>, Vec<u8>>> = self.cache_images.get(&key);
        match image {
            Some(image) => Some(image),
            None => {
                log::warn!("Invalid x: {}, y: {}", x, y);
                None
            }
        }
    }
}
