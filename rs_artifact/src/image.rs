use crate::error::Result;
use crate::{asset::Asset, resource_type::EResourceType};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

#[derive(Copy, PartialEq, Eq, Debug, Clone, Hash, Serialize, Deserialize)]
pub enum ColorType {
    L8,
    La8,
    Rgb8,
    Rgba8,
    L16,
    La16,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
}

impl ColorType {
    pub fn from_external_format(color_type: image::ColorType) -> ColorType {
        match color_type {
            image::ColorType::L8 => ColorType::L8,
            image::ColorType::La8 => ColorType::La8,
            image::ColorType::Rgb8 => ColorType::Rgb8,
            image::ColorType::Rgba8 => ColorType::Rgba8,
            image::ColorType::L16 => ColorType::L16,
            image::ColorType::La16 => ColorType::La16,
            image::ColorType::Rgb16 => ColorType::Rgb16,
            image::ColorType::Rgba16 => ColorType::Rgba16,
            image::ColorType::Rgb32F => ColorType::Rgb32F,
            image::ColorType::Rgba32F => ColorType::Rgba32F,
            _ => todo!(),
        }
    }

    pub fn to_external_format(&self) -> image::ColorType {
        match self {
            ColorType::L8 => image::ColorType::L8,
            ColorType::La8 => image::ColorType::La8,
            ColorType::Rgb8 => image::ColorType::Rgb8,
            ColorType::Rgba8 => image::ColorType::Rgba8,
            ColorType::L16 => image::ColorType::L16,
            ColorType::La16 => image::ColorType::La16,
            ColorType::Rgb16 => image::ColorType::Rgb16,
            ColorType::Rgba16 => image::ColorType::Rgba16,
            ColorType::Rgb32F => image::ColorType::Rgb32F,
            ColorType::Rgba32F => image::ColorType::Rgba32F,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Image {
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
        let mut reader = image::ImageReader::new(std::io::Cursor::new(&self.data));
        reader.set_format(self.image_format.to_external_format());
        reader.decode()
    }

    pub fn from_file(path: &str) -> Result<Image> {
        let file = std::fs::File::open(path).map_err(|err| {
            crate::error::Error::IO(err, Some(format!("Can not open file {}", path)))
        })?;
        let buf_reader = std::io::BufReader::new(file);
        let data = buf_reader.buffer().to_vec();
        let image_reader = image::ImageReader::new(buf_reader);
        let image_reader = image_reader
            .with_guessed_format()
            .map_err(|err| crate::error::Error::IO(err, Some(format!("Unknow format"))))?;
        let format = image_reader
            .format()
            .ok_or(crate::error::Error::File(Some("Unknow format".to_string())))?;
        let image = Image {
            image_format: ImageFormat::from_external_format(format),
            data,
            url: url::Url::from_str("").unwrap(),
        };
        Ok(image)
    }
}
