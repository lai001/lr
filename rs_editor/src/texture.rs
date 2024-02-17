use anyhow::{anyhow, Context, Result};
use image::GenericImage;
use md5::Digest;
use rs_artifact::{virtual_texture::image::TileIndex, EEndianType};
use rs_engine::mipmap_generator::MipmapGenerator;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureFile {
    pub name: String,
    pub url: url::Url,
    pub image_reference: Option<PathBuf>,
    pub is_virtual_texture: bool,
    pub virtual_image_reference: Option<PathBuf>,
}

impl TextureFile {
    pub fn new(name: String, url: url::Url) -> Self {
        Self {
            url,
            image_reference: None,
            name,
            is_virtual_texture: false,
            virtual_image_reference: None,
        }
    }

    pub fn is_virtual_image_cache_vaild(
        &self,
        endian_type: Option<EEndianType>,
    ) -> anyhow::Result<()> {
        if !self.is_virtual_texture {
            return Err(anyhow!("Is not a virtual texture"));
        }
        let virtual_image_reference = &self
            .virtual_image_reference
            .clone()
            .ok_or(anyhow!("Property virtual_image_reference is not set."))?;

        if !virtual_image_reference.exists() || !virtual_image_reference.is_file() {
            return Err(anyhow!(
                "{:?} is not exists or not a file.",
                virtual_image_reference
            ));
        }
        let decode_result = rs_artifact::virtual_texture::image::decode_from_path(
            virtual_image_reference,
            endian_type,
        );
        Ok(decode_result.map(|_| ())?)
    }

    pub fn create_virtual_texture_cache<P: AsRef<Path>>(
        &mut self,
        asset_folder: P,
        output: P,
        endian_type: Option<rs_artifact::EEndianType>,
        tile_size: u32,
    ) -> anyhow::Result<()> {
        // self.is_virtual_image_cache_vaild(endian_type)?;
        let image_reference = self
            .image_reference
            .clone()
            .ok_or(anyhow!("image_reference is null."))?;
        let create_result = create_virtual_texture_cache_file(
            asset_folder.as_ref().join(image_reference),
            output.as_ref().to_path_buf(),
            endian_type,
            tile_size,
        );

        if create_result.is_ok() {
            self.virtual_image_reference = Some(output.as_ref().to_path_buf());
        }
        create_result
    }

    pub fn get_pref_virtual_cache_name<P: AsRef<Path>>(&self, asset_folder: P) -> Result<String> {
        let Some(image_reference) = &self.image_reference else {
            return Err(anyhow!("image_reference is null."));
        };
        let mut hasher = md5::Md5::new();
        let data = std::fs::read(asset_folder.as_ref().join(image_reference))
            .context(format!("Failed to read from {:?}", image_reference))?;
        hasher.update(data);
        let result = hasher.finalize();
        let result = result.to_ascii_lowercase();
        let result = result
            .iter()
            .fold("".to_string(), |acc, x| format!("{acc}{:x?}", x));
        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureFolder {
    pub name: String,
    pub url: url::Url,
    pub texture_files: Vec<TextureFile>,
    pub texture_folders: Vec<TextureFolder>,
}

impl TextureFolder {
    pub fn new(name: &str, url: url::Url) -> Self {
        Self {
            name: name.to_string(),
            texture_files: Vec::new(),
            texture_folders: Vec::new(),
            url,
        }
    }
}

pub fn create_virtual_texture_cache_file<P: AsRef<Path>>(
    file_path: P,
    output: P,
    endian_type: Option<rs_artifact::EEndianType>,
    tile_size: u32,
) -> anyhow::Result<()> {
    let mut image = image::open(file_path.as_ref())
        .context(format!("Can not open file {:?}", file_path.as_ref()))?;

    if image.width() % tile_size != 0 || image.height() % tile_size != 0 {
        return Err(anyhow!("Size is not correct."));
    }
    let mut tiles: Vec<(TileIndex, image::DynamicImage)> = Vec::new();
    for x in 0..(image.width() / tile_size) {
        for y in 0..(image.height() / tile_size) {
            let sub_image = image.sub_image(x, y, tile_size, tile_size).to_image();
            let sub_image = image::DynamicImage::ImageRgba8(sub_image);
            let mut mipmap_images =
                MipmapGenerator::generate_from_image_cpu(&sub_image, None, None);
            mipmap_images.insert(0, sub_image);
            let mut level: u32 = 0;
            loop {
                if mipmap_images.is_empty() {
                    break;
                }
                let image = mipmap_images.remove(0);
                let tile_index = rs_artifact::virtual_texture::image::TileIndex {
                    x,
                    y,
                    mipmap_level: level,
                };
                tiles.push((tile_index, image));
                level += 1;
            }
        }
    }
    let tiles = tiles
        .iter()
        .map(|x| (x.0, &x.1))
        .collect::<std::vec::Vec<(TileIndex, &image::DynamicImage)>>();
    Ok(rs_artifact::virtual_texture::image::encode_to_file(
        output,
        endian_type,
        tiles,
    )?)
}
