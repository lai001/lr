use crate::EEndianType;
use serde::{Deserialize, Serialize};

type HeaderLengthDataType = u64;

const IDENTIFICATION_OFFSET: usize = 0;
const IDENTIFICATION_SIZE: usize = 4;
const HEADER_LENGTH_OFFSET: usize = IDENTIFICATION_OFFSET + IDENTIFICATION_SIZE;
const HEADER_LENGTH_SIZE: usize = std::mem::size_of::<HeaderLengthDataType>();
const HEADER_OFFSET: usize = HEADER_LENGTH_OFFSET + HEADER_LENGTH_SIZE;

pub const FILE_MAGIC_NUMBERS: &[u8; IDENTIFICATION_SIZE] = &[b'r', b's', b'd', b'f'];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub enum EResourceType {
    Image,
    StaticMesh,
    Generic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceInfo {
    url: url::Url,
    offset: usize,
    length: usize,
    resource_type: EResourceType,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FileHeader {
    pub resource_map: std::collections::HashMap<url::Url, ResourceInfo>,
}

impl FileHeader {
    pub fn write_header<T>(
        magic_numbers: &[u8; IDENTIFICATION_SIZE],
        header: &T,
        endian_type: Option<EEndianType>,
    ) -> Option<Vec<u8>>
    where
        T: serde::ser::Serialize,
    {
        let endian_type = endian_type.unwrap_or(EEndianType::Native);
        if let Ok(mut serialize_data) = bincode::serialize(header) {
            let header_length: HeaderLengthDataType = serialize_data.len().try_into().unwrap();
            let mut header_length_data: Vec<u8> = vec![0; HEADER_LENGTH_SIZE];
            match endian_type {
                EEndianType::Big => {
                    header_length_data.copy_from_slice(&header_length.to_be_bytes());
                }
                EEndianType::Little => {
                    header_length_data.copy_from_slice(&header_length.to_le_bytes());
                }
                EEndianType::Native => {
                    header_length_data.copy_from_slice(&header_length.to_ne_bytes());
                }
            }
            let mut data: Vec<u8> = Vec::new();
            data.append(&mut magic_numbers.to_vec());
            data.append(&mut header_length_data);
            data.append(&mut serialize_data);
            return Some(data);
        }
        return None;
    }

    pub fn get_header<R, T>(reader: &mut R, header_length: HeaderLengthDataType) -> Option<T>
    where
        R: std::io::Seek + std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        if let Some(data) = Self::get_header_data(reader, header_length) {
            if let Ok(file_header) = bincode::deserialize(&data) {
                return Some(file_header);
            }
        }
        return None;
    }

    pub fn get_header2<R, T>(reader: &mut R, endian_type: Option<EEndianType>) -> Option<T>
    where
        R: std::io::Seek + std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        if let Some(header_length) = Self::get_header_length(reader, endian_type) {
            if let Some(data) = Self::get_header_data(reader, header_length) {
                if let Ok(file_header) = bincode::deserialize(&data) {
                    return Some(file_header);
                }
            }
        }
        return None;
    }

    pub fn get_header_data<R>(
        reader: &mut R,
        header_length: HeaderLengthDataType,
    ) -> Option<Vec<u8>>
    where
        R: std::io::Seek + std::io::Read,
    {
        let current_position = reader.stream_position();
        match current_position {
            Ok(current_position) => {
                match reader.seek(std::io::SeekFrom::Start(HEADER_OFFSET as u64)) {
                    Ok(_) => {
                        let mut data: Vec<u8> = vec![0; header_length as usize];
                        match reader.read_exact(&mut data) {
                            Ok(_) => {
                                return Some(data);
                            }
                            Err(_) => {}
                        }
                        let _ = reader.seek(std::io::SeekFrom::Start(current_position));
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn get_header_length<R>(
        reader: &mut R,
        endian_type: Option<EEndianType>,
    ) -> Option<HeaderLengthDataType>
    where
        R: std::io::Seek + std::io::Read,
    {
        let endian_type = endian_type.unwrap_or(EEndianType::Native);
        let current_position = reader.stream_position();
        match current_position {
            Ok(current_position) => {
                match reader.seek(std::io::SeekFrom::Start(HEADER_LENGTH_OFFSET as u64)) {
                    Ok(_) => {
                        let mut data: Vec<u8> = vec![0; HEADER_LENGTH_SIZE];
                        match reader.read_exact(&mut data) {
                            Ok(_) => match endian_type {
                                EEndianType::Big => {
                                    let bytes = <[u8; HEADER_LENGTH_SIZE]>::try_from(data);
                                    match bytes {
                                        Ok(bytes) => {
                                            let length = HeaderLengthDataType::from_be_bytes(bytes);
                                            return Some(length);
                                        }
                                        Err(_) => {}
                                    }
                                }
                                EEndianType::Little => {
                                    let bytes = <[u8; HEADER_LENGTH_SIZE]>::try_from(data);
                                    match bytes {
                                        Ok(bytes) => {
                                            let length = HeaderLengthDataType::from_le_bytes(bytes);
                                            return Some(length);
                                        }
                                        Err(_) => {}
                                    }
                                }
                                EEndianType::Native => {
                                    let bytes = <[u8; HEADER_LENGTH_SIZE]>::try_from(data);
                                    match bytes {
                                        Ok(bytes) => {
                                            let length = HeaderLengthDataType::from_ne_bytes(bytes);
                                            return Some(length);
                                        }
                                        Err(_) => {}
                                    }
                                }
                            },
                            Err(_) => {}
                        }
                        let _ = reader.seek(std::io::SeekFrom::Start(current_position));
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn check_identification<R>(reader: &mut R, magic_numbers: &[u8]) -> bool
    where
        R: std::io::Seek + std::io::Read,
    {
        let current_position = reader.stream_position();
        match current_position {
            Ok(current_position) => {
                match reader.seek(std::io::SeekFrom::Start(IDENTIFICATION_OFFSET as u64)) {
                    Ok(_) => {
                        let mut data: Vec<u8> = vec![0; magic_numbers.len()];
                        match reader.read_exact(&mut data) {
                            Ok(_) => {
                                if data == magic_numbers {
                                    return true;
                                }
                            }
                            Err(_) => {}
                        }
                        let _ = reader.seek(std::io::SeekFrom::Start(current_position));
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        return false;
    }
}

#[cfg(test)]
mod test {
    use super::{FileHeader, ResourceInfo, FILE_MAGIC_NUMBERS};
    use std::collections::HashMap;

    #[test]
    fn test_case() {
        let dir = std::path::Path::new(&std::env::current_dir().unwrap())
            .join("target")
            .join("debug");
        let filename = "test.rs";
        let file_path = dir.join(filename);

        let resource = ResourceInfo {
            url: url::Url::parse("https://github.com/lai001").unwrap(),
            offset: 0,
            length: 1024,
            resource_type: super::EResourceType::Generic,
        };
        let fileheader = FileHeader {
            resource_map: HashMap::from([(resource.url.clone(), resource)]),
        };
        let data = FileHeader::write_header(
            FILE_MAGIC_NUMBERS,
            &fileheader,
            Some(crate::EEndianType::Little),
        )
        .unwrap();
        let _ = std::fs::write(file_path, data);
    }

    #[test]
    fn test_case_1() {
        test_case();
        let dir = std::path::Path::new(&std::env::current_dir().unwrap())
            .join("target")
            .join("debug");
        let filename = "test.rs";
        let file_path = dir.join(filename);
        let f = std::fs::File::open(file_path).unwrap();
        let mut reader = std::io::BufReader::new(f);
        let file: FileHeader =
            FileHeader::get_header2(&mut reader, Some(crate::EEndianType::Little)).unwrap();
        let url = url::Url::parse("https://github.com/lai001").unwrap();
        let resource_info = file.resource_map.get(&url).unwrap();
        assert_eq!(resource_info.url, url);
    }
}
