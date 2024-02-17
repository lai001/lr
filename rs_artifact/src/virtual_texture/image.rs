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
    tile_map: HashMap<TileIndex, ImageInfo>,
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
    pub fn get_dynamic_image(&mut self, tile_index: TileIndex) -> Option<image::DynamicImage> {
        match self.file_header.tile_map.get(&tile_index) {
            Some(image_info) => {
                let size = (image_info.span.end - image_info.span.start) as usize;
                let _ = match self.reader.seek(std::io::SeekFrom::Start(
                    self.body_offset + image_info.span.start,
                )) {
                    Ok(new_position) => new_position,
                    Err(_) => {
                        return None;
                    }
                };
                let mut buffer = Vec::new();
                buffer.resize(size, 0);
                match self.reader.read(&mut buffer) {
                    Ok(read) => {
                        if read != size {
                            None
                        } else {
                            new_dynamic_image(
                                buffer,
                                image_info.color_type.to_external_format(),
                                image_info.width,
                                image_info.height,
                            )
                        }
                    }
                    Err(_) => None,
                }
            }
            None => None,
        }
    }

    pub fn decode_from_reader(mut reader: R, endian_type: Option<EEndianType>) -> Result<Image<R>>
    where
        R: Seek + Read,
    {
        let is_vail =
            crate::file_header::FileHeader::check_identification(&mut reader, FILE_MAGIC_NUMBERS);
        if is_vail.is_err() {
            return Err(crate::error::Error::CheckIdentificationFail);
        }

        let header_bytes_length =
            match crate::file_header::FileHeader::get_header_encoded_data_length(
                &mut reader,
                endian_type,
            ) {
                Ok(header_bytes_length) => header_bytes_length,
                Err(err) => {
                    return Err(err);
                }
            };
        let file_header: ImageFileHeader =
            match crate::file_header::FileHeader::get_header2(&mut reader, endian_type) {
                Ok(heaedr) => heaedr,
                Err(err) => {
                    return Err(err);
                }
            };
        let body_offset: u64 = HEADER_OFFSET as u64 + header_bytes_length;

        let image = Image {
            file_header,
            body_offset,
            reader,
        };

        Ok(image)
    }

    pub fn get_tile_map(&self) -> &HashMap<TileIndex, ImageInfo> {
        &self.file_header.tile_map
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
    tiles: Vec<(TileIndex, &image::DynamicImage)>,
) -> Result<()>
where
    W: Write,
{
    let mut file_header = ImageFileHeader {
        tile_map: HashMap::new(),
    };
    let mut start: u64 = 0;
    for (tile_index, dynamic_image) in tiles.iter() {
        let tile_data = dynamic_image.as_bytes();
        let span = Span {
            start,
            end: start + (tile_data.len() as u64),
        };
        let info = ImageInfo {
            span,
            width: dynamic_image.width(),
            height: dynamic_image.height(),
            color_type: crate::image::ColorType::from_external_format(dynamic_image.color()),
        };
        file_header.tile_map.insert(tile_index.clone(), info);
        start = span.end;
    }

    let header_data = match crate::file_header::FileHeader::write_header(
        FILE_MAGIC_NUMBERS,
        &file_header,
        endian_type,
    ) {
        Ok(header_data) => header_data,
        Err(err) => {
            return Err(err);
        }
    };
    let _ = match writer.write(&header_data) {
        Ok(write_size) => write_size,
        Err(err) => {
            return Err(crate::error::Error::IO(err, None));
        }
    };

    for (_, tile_data) in tiles {
        match writer.write(tile_data.as_bytes()) {
            Ok(_) => {}
            Err(err) => {
                return Err(crate::error::Error::IO(err, None));
            }
        }
    }

    Ok(())
}

pub fn encode_to_file<P: AsRef<Path>>(
    path: P,
    endian_type: Option<EEndianType>,
    tiles: Vec<(TileIndex, &image::DynamicImage)>,
) -> Result<()> {
    let mut writer = match OpenOptions::new().write(true).create(true).open(path) {
        Ok(writer) => writer,
        Err(err) => {
            return Err(crate::error::Error::IO(err, None));
        }
    };
    encode_to_writer(&mut writer, endian_type, tiles)
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
