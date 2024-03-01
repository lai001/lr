use crate::error::Result;
use rs_artifact::{
    virtual_texture::image::{Image, TileIndex},
    EEndianType,
};
use rs_render::virtual_texture_source::TVirtualTextureSource;
use std::{fs::File, path::Path};

enum ESourceType {
    File(Image<File>),
}

pub struct StaticVirtualTextureSource {
    source: ESourceType,
}

impl StaticVirtualTextureSource {
    pub fn from_file<P: AsRef<Path>>(path: P, endian_type: Option<EEndianType>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(crate::error::Error::IO(
                std::io::ErrorKind::NotFound.into(),
                None,
            ));
        }
        let image = rs_artifact::virtual_texture::image::decode_from_path(path, endian_type)
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        let source = ESourceType::File(image);
        Ok(Self { source })
    }
}

impl TVirtualTextureSource for StaticVirtualTextureSource {
    fn get_tile_image(&mut self, index: &glam::UVec3) -> Option<image::DynamicImage> {
        match &mut self.source {
            ESourceType::File(image) => {
                let tile_index = TileIndex {
                    x: index.x,
                    y: index.y,
                    mipmap_level: index.z,
                };
                image.get_dynamic_image(&tile_index).ok()
            }
        }
    }

    fn get_size(&self) -> glam::UVec2 {
        match &self.source {
            ESourceType::File(image) => image.get_size(),
        }
    }
}
