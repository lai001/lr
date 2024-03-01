use crate::error::Result;
use crate::file_header::HEADER_OFFSET;
use crate::{file_header::IDENTIFICATION_SIZE, EEndianType};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Seek};
use std::{collections::HashMap, fs::OpenOptions, io::Write, path::Path};

pub const FILE_MAGIC_NUMBERS: &[u8; IDENTIFICATION_SIZE] = &[b'v', b'i', b'm', b'g'];

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TileIndex {
    pub x: u32,
    pub y: u32,
    pub mipmap_level: u32,
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ImageInfo {
    span: Span,
    width: u32,
    height: u32,
    color_type: crate::image::ColorType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageFileHeader {
    lod_sizes: Vec<glam::UVec2>,
    lod_images: Vec<HashMap<glam::UVec2, ImageInfo>>,
}

#[derive(Debug)]
pub struct Image<R>
where
    R: Seek + Read,
{
    file_header: ImageFileHeader,
    body_offset: u64,
    reader: R,
}

impl<R> Image<R>
where
    R: Seek + Read,
{
    pub fn get_size(&self) -> glam::UVec2 {
        self.file_header.lod_sizes.get(0).copied().unwrap()
    }

    pub fn get_dynamic_image(&mut self, tile_index: &TileIndex) -> Result<image::DynamicImage> {
        let lod = tile_index.mipmap_level;
        let tile_index = glam::uvec2(tile_index.x, tile_index.y);
        let image_infos =
            self.file_header
                .lod_images
                .get(lod as usize)
                .ok_or(crate::error::Error::NotFound(Some(format!(
                    "{:?}",
                    tile_index
                ))))?;
        let image_info = image_infos
            .get(&tile_index)
            .ok_or(crate::error::Error::NotFound(Some(format!("{:?}", lod))))?;
        let size = (image_info.span.end - image_info.span.start) as usize;
        self.reader
            .seek(std::io::SeekFrom::Start(
                self.body_offset + image_info.span.start,
            ))
            .map_err(|err| crate::error::Error::IO(err, None))?;
        let mut buffer = Vec::new();
        buffer.resize(size, 0);
        let read = self
            .reader
            .read(&mut buffer)
            .map_err(|err| crate::error::Error::IO(err, None))?;
        if read != size {
            Err(crate::error::Error::IO(
                std::io::ErrorKind::Other.into(),
                None,
            ))
        } else {
            let image = new_dynamic_image(
                buffer,
                image_info.color_type.to_external_format(),
                image_info.width,
                image_info.height,
            )
            .ok_or(crate::error::Error::DataConvertFail)?;
            Ok(image)
        }
    }

    pub fn decode_from_reader(mut reader: R, endian_type: Option<EEndianType>) -> Result<Image<R>>
    where
        R: Seek + Read,
    {
        crate::file_header::FileHeader::check_identification(&mut reader, FILE_MAGIC_NUMBERS)?;

        let header_bytes_length = crate::file_header::FileHeader::get_header_encoded_data_length(
            &mut reader,
            endian_type,
        )?;
        let file_header: ImageFileHeader =
            crate::file_header::FileHeader::get_header2(&mut reader, endian_type)?;
        let body_offset: u64 = HEADER_OFFSET as u64 + header_bytes_length;

        let image = Image {
            file_header,
            body_offset,
            reader,
        };

        Ok(image)
    }

    pub fn get_tile_map(&self) -> &[HashMap<glam::UVec2, ImageInfo>] {
        &self.file_header.lod_images
    }
}

pub fn decode_from_path<P: AsRef<Path>>(
    path: P,
    endian_type: Option<EEndianType>,
) -> Result<Image<File>> {
    let reader = match OpenOptions::new().read(true).open(path) {
        Ok(reader) => reader,
        Err(err) => {
            return Err(crate::error::Error::IO(err, None));
        }
    };
    let image = Image::<File>::decode_from_reader(reader, endian_type);
    image
}

pub fn encode_to_writer<W>(
    writer: &mut W,
    endian_type: Option<EEndianType>,
    lod_sizes: Vec<glam::UVec2>,
    lod_tiles: Vec<HashMap<glam::UVec2, image::DynamicImage>>,
) -> Result<()>
where
    W: Write,
{
    let mut file_header = ImageFileHeader {
        lod_sizes,
        lod_images: vec![],
    };
    let mut start: u64 = 0;
    let mut body: Vec<u8> = vec![];

    for tiles in lod_tiles {
        let mut infos: HashMap<glam::UVec2, ImageInfo> = HashMap::new();
        for (index, tile) in tiles {
            let data = tile.as_bytes();
            let span = Span {
                start,
                end: start + (data.len() as u64),
            };
            body.extend_from_slice(data);
            let info = ImageInfo {
                span,
                width: tile.width(),
                height: tile.height(),
                color_type: crate::image::ColorType::from_external_format(tile.color()),
            };
            start = span.end;
            infos.insert(index, info);
        }
        file_header.lod_images.push(infos);
    }

    let header_data = crate::file_header::FileHeader::write_header(
        FILE_MAGIC_NUMBERS,
        &file_header,
        endian_type,
    )?;
    writer
        .write(&header_data)
        .map_err(|err| crate::error::Error::IO(err, None))?;
    writer
        .write(&body)
        .map_err(|err| crate::error::Error::IO(err, None))?;

    Ok(())
}

pub fn encode_to_file<P: AsRef<Path>>(
    path: P,
    endian_type: Option<EEndianType>,
    lod_sizes: Vec<glam::UVec2>,
    lod_tiles: Vec<HashMap<glam::UVec2, image::DynamicImage>>,
) -> Result<()> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .map_err(|err| crate::error::Error::IO(err, None))?;

    encode_to_writer(&mut writer, endian_type, lod_sizes, lod_tiles)
}

fn new_dynamic_image(
    buffer: Vec<u8>,
    color_type: image::ColorType,
    width: u32,
    height: u32,
) -> Option<image::DynamicImage> {
    match color_type {
        image::ColorType::L8 => match image::GrayImage::from_vec(width, height, buffer) {
            Some(image) => Some(image::DynamicImage::ImageLuma8(image)),
            None => None,
        },
        image::ColorType::Rgba8 => match image::RgbaImage::from_vec(width, height, buffer) {
            Some(image) => Some(image::DynamicImage::ImageRgba8(image)),
            None => None,
        },
        image::ColorType::Rgb8 => match image::RgbImage::from_vec(width, height, buffer) {
            Some(image) => Some(image::DynamicImage::ImageRgb8(image)),
            None => None,
        },
        image::ColorType::La8 => match image::GrayAlphaImage::from_vec(width, height, buffer) {
            Some(image) => Some(image::DynamicImage::ImageLumaA8(image)),
            None => None,
        },
        image::ColorType::Rgba32F => {
            match image::Rgba32FImage::from_vec(
                width,
                height,
                rs_foundation::cast_to_type_vec(buffer),
            ) {
                Some(image) => Some(image::DynamicImage::ImageRgba32F(image)),
                None => None,
            }
        }
        image::ColorType::Rgb32F => {
            match image::Rgb32FImage::from_vec(
                width,
                height,
                rs_foundation::cast_to_type_vec(buffer),
            ) {
                Some(image) => Some(image::DynamicImage::ImageRgb32F(image)),
                None => None,
            }
        }
        image::ColorType::L16 => todo!(),
        image::ColorType::La16 => todo!(),
        image::ColorType::Rgb16 => todo!(),
        image::ColorType::Rgba16 => todo!(),
        _ => todo!(),
    }
}
