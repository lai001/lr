use mp4::*;
use std::{
    io::{Cursor, Read, Seek},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct CompositionInfo {
    pub rgb_rect: glam::Vec4,
    pub alpha_rect: glam::Vec4,
}

pub fn check_composition(
    input_file: impl AsRef<Path>,
) -> crate::error::Result<Option<CompositionInfo>> {
    const MDTA: FourCC = FourCC { value: *b"mdta" };
    const MDTA_VALUE: u32 = u32::from_be_bytes(MDTA.value);
    const MDTA_BOX_TYPE: BoxType = BoxType::UnknownBox(MDTA_VALUE);

    const KEYS: FourCC = FourCC { value: *b"keys" };
    const KEYS_VALUE: u32 = u32::from_be_bytes(KEYS.value);
    const KEYS_BOX_TYPE: BoxType = BoxType::UnknownBox(KEYS_VALUE);

    let file = std::fs::File::open(input_file).map_err(|err| crate::error::Error::IO(err))?;
    let mp4_reader = mp4::read_mp4(file).map_err(|err| crate::error::Error::MP4(err))?;
    let Some(meta) = mp4_reader.moov.udta.map(|x| x.meta).flatten() else {
        return Ok(None);
    };

    let mut pairs: Vec<(String, String)> = vec![];

    match meta {
        mp4::meta::MetaBox::Mdir { .. } => {
            return Ok(None);
        }
        mp4::meta::MetaBox::Unknown { data, .. } => {
            if data.len() != 2 {
                return Ok(None);
            }
            if data[0].0 != KEYS_BOX_TYPE {
                return Ok(None);
            }
            if data[1].0 != BoxType::IlstBox {
                return Ok(None);
            }
            {
                let keys_box_data = &data[0].1;
                let mut reader = Cursor::new(keys_box_data);
                let mut keys_num: u64 = 0;
                let keys_num_read: &mut [u8] = unsafe {
                    std::slice::from_raw_parts_mut(&mut keys_num as *mut _ as *mut u8, 8)
                };
                reader
                    .read_exact(keys_num_read)
                    .map_err(|err| crate::error::Error::IO(err))?;
                let keys_num = keys_num.to_be();
                reader
                    .seek(std::io::SeekFrom::Start(8))
                    .map_err(|err| crate::error::Error::IO(err))?;
                let mut keys_box_body_data = vec![];
                reader
                    .read_to_end(&mut keys_box_body_data)
                    .map_err(|err| crate::error::Error::IO(err))?;
                let mut reader = Cursor::new(&mut keys_box_body_data);
                for _ in 0..keys_num {
                    let box_header = BoxHeader::read(&mut reader)
                        .map_err(|err| crate::error::Error::MP4(err))?;
                    if box_header.name == MDTA_BOX_TYPE {
                        let mut key_data: Vec<u8> = vec![];
                        key_data.resize((box_header.size - HEADER_SIZE) as usize, 0);
                        reader
                            .read(&mut key_data)
                            .map_err(|err| crate::error::Error::IO(err))?;
                        let key = String::from_utf8(key_data)
                            .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
                        pairs.push((key, format!("")));
                    } else {
                        return Err(crate::error::Error::Other(format!("Not mdta box type")));
                    }
                }
            }

            {
                let ilst_box_data = &data[1].1;
                let mut reader = Cursor::new(ilst_box_data);
                for (_, v) in pairs.iter_mut() {
                    reader
                        .seek(std::io::SeekFrom::Current(8))
                        .map_err(|err| crate::error::Error::IO(err))?;
                    let box_header = BoxHeader::read(&mut reader)
                        .map_err(|err| crate::error::Error::MP4(err))?;

                    if box_header.name == BoxType::DataBox {
                        let mut value_data: Vec<u8> = vec![];
                        value_data.resize((box_header.size - HEADER_SIZE - 8) as usize, 0);
                        reader
                            .seek(std::io::SeekFrom::Current(8))
                            .map_err(|err| crate::error::Error::IO(err))?;
                        reader
                            .read(&mut value_data)
                            .map_err(|err| crate::error::Error::IO(err))?;
                        let value = String::from_utf8(value_data)
                            .map_err(|err| crate::error::Error::FromUtf8Error(err))?;
                        *v = value;
                    } else {
                        return Err(crate::error::Error::Other(format!("Not data box type")));
                    }
                }
            }
        }
    }
    let composition_info = pairs.iter().find(|x| x.0 == "composition_info");
    let Some(pair) = composition_info else {
        return Ok(None);
    };
    let mut value = pair.1.to_string();
    value = value
        .strip_prefix("\"")
        .ok_or(crate::error::Error::Other(format!("")))?
        .to_string();
    value = value
        .strip_suffix("\"")
        .ok_or(crate::error::Error::Other(format!("")))?
        .to_string();

    let sp = value
        .split(";")
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    if sp.len() != 2 {
        return Ok(None);
    }

    let first = sp[0]
        .split(",")
        .map(|x| x.parse::<f32>())
        .filter_map(|x| x.ok())
        .collect::<Vec<f32>>();

    let second = sp[1]
        .split(",")
        .map(|x| x.parse::<f32>())
        .filter_map(|x| x.ok())
        .collect::<Vec<f32>>();

    if first.len() != 4 && second.len() != 4 {
        return Ok(None);
    }

    let info = CompositionInfo {
        rgb_rect: glam::Vec4::from_slice(&first),
        alpha_rect: glam::Vec4::from_slice(&second),
    };

    Ok(Some(info))
}
