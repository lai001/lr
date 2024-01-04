use std::str::FromStr;

use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Pnm,
    Tiff,
    Tga,
    Dds,
    Bmp,
    Ico,
    Hdr,
    OpenExr,
    Farbfeld,
    Avif,
    Qoi,
}

impl ImageFormat {
    pub fn from_external_format(format: image::ImageFormat) -> ImageFormat {
        match format {
            image::ImageFormat::Png => ImageFormat::Png,
            image::ImageFormat::Jpeg => ImageFormat::Jpeg,
            image::ImageFormat::Gif => ImageFormat::Gif,
            image::ImageFormat::WebP => ImageFormat::WebP,
            image::ImageFormat::Pnm => ImageFormat::Pnm,
            image::ImageFormat::Tiff => ImageFormat::Tiff,
            image::ImageFormat::Tga => ImageFormat::Tga,
            image::ImageFormat::Dds => ImageFormat::Dds,
            image::ImageFormat::Bmp => ImageFormat::Bmp,
            image::ImageFormat::Ico => ImageFormat::Ico,
            image::ImageFormat::Hdr => ImageFormat::Hdr,
            image::ImageFormat::OpenExr => ImageFormat::OpenExr,
            image::ImageFormat::Farbfeld => ImageFormat::Farbfeld,
            image::ImageFormat::Avif => ImageFormat::Avif,
            image::ImageFormat::Qoi => ImageFormat::Qoi,
            _ => todo!(),
        }
    }

    pub fn to_external_format(&self) -> image::ImageFormat {
        match self {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Gif => image::ImageFormat::Gif,
            ImageFormat::WebP => image::ImageFormat::WebP,
            ImageFormat::Pnm => image::ImageFormat::Pnm,
            ImageFormat::Tiff => image::ImageFormat::Tiff,
            ImageFormat::Tga => image::ImageFormat::Tga,
            ImageFormat::Dds => image::ImageFormat::Dds,
            ImageFormat::Bmp => image::ImageFormat::Bmp,
            ImageFormat::Ico => image::ImageFormat::Ico,
            ImageFormat::Hdr => image::ImageFormat::Hdr,
            ImageFormat::OpenExr => image::ImageFormat::OpenExr,
            ImageFormat::Farbfeld => image::ImageFormat::Farbfeld,
            ImageFormat::Avif => image::ImageFormat::Avif,
            ImageFormat::Qoi => image::ImageFormat::Qoi,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Image {
    pub name: String,
    pub id: uuid::Uuid,
    pub url: url::Url,
    pub image_format: ImageFormat,
    pub data: Vec<u8>,
}

impl Asset for Image {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::Image
    }
}

impl Image {
    pub fn decode(&self) -> image::ImageResult<image::DynamicImage> {
        let mut reader = image::io::Reader::new(std::io::Cursor::new(&self.data));
        reader.set_format(self.image_format.to_external_format());
        reader.decode()
    }

    pub fn from_file(name: &str, id: uuid::Uuid, path: &str) -> Option<Image> {
        if let Ok(file) = std::fs::File::open(path) {
            let buf_reader = std::io::BufReader::new(file);
            let data = buf_reader.buffer().to_vec();
            let image_reader = image::io::Reader::new(buf_reader);
            let image_reader = image_reader.with_guessed_format().unwrap();
            if let Some(format) = image_reader.format() {
                return Some(Image {
                    name: name.to_string(),
                    id,
                    image_format: ImageFormat::from_external_format(format),
                    data,
                    url: url::Url::from_str("").unwrap(),
                });
            }
        }
        return None;
    }
}
