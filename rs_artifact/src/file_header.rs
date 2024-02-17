use crate::error::Result;
use crate::EEndianType;

type HeaderLengthDataType = u64;

pub const IDENTIFICATION_OFFSET: usize = 0;
pub const IDENTIFICATION_SIZE: usize = 4;
pub const HEADER_LENGTH_OFFSET: usize = IDENTIFICATION_OFFSET + IDENTIFICATION_SIZE;
pub const HEADER_LENGTH_SIZE: usize = std::mem::size_of::<HeaderLengthDataType>();
pub const HEADER_OFFSET: usize = HEADER_LENGTH_OFFSET + HEADER_LENGTH_SIZE;

pub const ARTIFACT_FILE_MAGIC_NUMBERS: &[u8; IDENTIFICATION_SIZE] = &[b'r', b's', b'd', b'f'];
pub const ASSET_FILE_MAGIC_NUMBERS: &[u8; IDENTIFICATION_SIZE] = &[b'a', b's', b'e', b't'];

pub struct FileHeader {}

impl FileHeader {
    pub fn write_header<T>(
        magic_numbers: &[u8; IDENTIFICATION_SIZE],
        header: &T,
        endian_type: Option<EEndianType>,
    ) -> Result<Vec<u8>>
    where
        T: serde::ser::Serialize,
    {
        let endian_type = endian_type.unwrap_or(EEndianType::Native);
        let mut serialize_data = bincode::serialize(header).map_err(|err| {
            let msg = format!("Fail to serialize.");
            crate::error::Error::Bincode(err, Some(msg))
        })?;
        let header_length = serialize_data.len() as HeaderLengthDataType;
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
        Ok(data)
    }

    pub fn get_header<R, T>(reader: &mut R, header_length: HeaderLengthDataType) -> Result<T>
    where
        R: std::io::Seek + std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        let data = Self::get_header_encoded_data(reader, header_length)?;
        let file_header = bincode::deserialize(&data).map_err(|err| {
            let msg = format!("Fail to deserialize.");
            crate::error::Error::Bincode(err, Some(msg))
        })?;
        Ok(file_header)
    }

    pub fn get_header2<R, T>(reader: &mut R, endian_type: Option<EEndianType>) -> Result<T>
    where
        R: std::io::Seek + std::io::Read,
        T: serde::de::DeserializeOwned,
    {
        let header_length = Self::get_header_encoded_data_length(reader, endian_type)?;
        let data = Self::get_header_encoded_data(reader, header_length)?;
        let file_header = bincode::deserialize(&data).map_err(|err| {
            let msg = format!("Fail to deserialize.");
            crate::error::Error::Bincode(err, Some(msg))
        })?;
        Ok(file_header)
    }

    pub fn get_header_encoded_data<R>(
        reader: &mut R,
        header_length: HeaderLengthDataType,
    ) -> Result<Vec<u8>>
    where
        R: std::io::Seek + std::io::Read,
    {
        let _ = reader
            .seek(std::io::SeekFrom::Start(HEADER_OFFSET as u64))
            .map_err(|err| {
                crate::error::Error::IO(err, Some(String::from("Failed to seek `HEADER_OFFSET`.")))
            })?;
        let mut data: Vec<u8> = vec![0; header_length as usize];
        let _ = reader.read_exact(&mut data).map_err(|err| {
            let msg = String::from("Fail to read the exact number of bytes.");
            crate::error::Error::IO(err, Some(msg))
        })?;
        Ok(data)
    }

    pub fn get_header_encoded_data_length<R>(
        reader: &mut R,
        endian_type: Option<EEndianType>,
    ) -> Result<HeaderLengthDataType>
    where
        R: std::io::Seek + std::io::Read,
    {
        let endian_type = endian_type.unwrap_or(EEndianType::Native);
        let _ = reader
            .seek(std::io::SeekFrom::Start(HEADER_LENGTH_OFFSET as u64))
            .map_err(|err| {
                let msg = String::from("Failed to seek `HEADER_LENGTH_OFFSET`.");
                crate::error::Error::IO(err, Some(msg))
            })?;

        let mut data: Vec<u8> = vec![0; HEADER_LENGTH_SIZE];
        reader
            .read_exact(&mut data)
            .map_err(|err| crate::error::Error::IO(err, None))?;

        match endian_type {
            EEndianType::Big => {
                let bytes =
                    <[u8; HEADER_LENGTH_SIZE]>::try_from(data).expect("Convert successfully.");
                let length = HeaderLengthDataType::from_be_bytes(bytes);
                Ok(length)
            }
            EEndianType::Little => {
                let bytes =
                    <[u8; HEADER_LENGTH_SIZE]>::try_from(data).expect("Convert successfully.");
                let length = HeaderLengthDataType::from_le_bytes(bytes);
                Ok(length)
            }
            EEndianType::Native => {
                let bytes =
                    <[u8; HEADER_LENGTH_SIZE]>::try_from(data).expect("Convert successfully.");
                let length = HeaderLengthDataType::from_ne_bytes(bytes);
                Ok(length)
            }
        }
    }

    pub fn check_identification<R>(reader: &mut R, magic_numbers: &[u8]) -> Result<()>
    where
        R: std::io::Seek + std::io::Read,
    {
        reader
            .seek(std::io::SeekFrom::Start(IDENTIFICATION_OFFSET as u64))
            .map_err(|err| {
                let msg = String::from("Failed to seek `IDENTIFICATION_OFFSET`.");
                crate::error::Error::IO(err, Some(msg))
            })?;

        let mut data: Vec<u8> = vec![0; magic_numbers.len()];

        reader.read_exact(&mut data).map_err(|err| {
            let msg = String::from("Failed to read `IDENTIFICATION` data.");
            crate::error::Error::IO(err, Some(msg))
        })?;
        if data == magic_numbers {
            Ok(())
        } else {
            Err(crate::error::Error::CheckIdentificationFail)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{FileHeader, ARTIFACT_FILE_MAGIC_NUMBERS};
    use crate::{
        artifact::ArtifactFileHeader, resource_info::ResourceInfo, resource_type::EResourceType,
    };
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
            resource_type: EResourceType::Binary,
        };
        let fileheader = ArtifactFileHeader {
            resource_map: HashMap::from([(resource.url.clone(), resource)]),
        };
        let data = FileHeader::write_header(
            ARTIFACT_FILE_MAGIC_NUMBERS,
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
        let file: ArtifactFileHeader =
            FileHeader::get_header2(&mut reader, Some(crate::EEndianType::Little)).unwrap();
        let url = url::Url::parse("https://github.com/lai001").unwrap();
        let resource_info = file.resource_map.get(&url).unwrap();
        assert_eq!(resource_info.url, url);
    }
}
