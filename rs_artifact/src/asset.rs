use crate::error::Result;
use crate::{
    artifact::ResourceEncodeTask,
    file_header::{FileHeader, ASSET_FILE_MAGIC_NUMBERS, HEADER_LENGTH_SIZE},
    resource_type::EResourceType,
    EEndianType,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek};

pub trait Asset: for<'a> Deserialize<'a> + Serialize + Sized {
    fn get_url(&self) -> url::Url;
    fn get_resource_type(&self) -> EResourceType;
    fn build_resource_encode_task<R>(&self, reader: R) -> ResourceEncodeTask<R>
    where
        R: Seek + Read,
    {
        ResourceEncodeTask {
            url: self.get_url(),
            resource_type: self.get_resource_type(),
            reader,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AssetHeader {
    pub resource_type: EResourceType,
}

pub fn encode_asset<A>(
    resource_type: EResourceType,
    endian_type: Option<EEndianType>,
    asset: &A,
) -> Result<Vec<u8>>
where
    A: Asset,
{
    let asset_header = AssetHeader { resource_type };

    let header_data =
        match FileHeader::write_header(ASSET_FILE_MAGIC_NUMBERS, &asset_header, endian_type) {
            Ok(header_data) => header_data,
            Err(err) => return Err(err),
        };

    let payload = match bincode::serialize(asset) {
        Ok(payload) => payload,
        Err(err) => {
            return Err(crate::error::Error::Bincode(
                err,
                Some(format!("Fail to serialize.")),
            ));
        }
    };

    let mut data = vec![0; header_data.len() + payload.len()];
    data[0..header_data.len()].copy_from_slice(&header_data);
    data[header_data.len()..].copy_from_slice(&payload);
    return Ok(data);
}

pub fn decode_asset<T>(
    data: &[u8],
    endian_type: Option<EEndianType>,
    expected_resource_type: Option<EResourceType>,
) -> Result<T>
where
    T: Asset,
{
    let mut reader = std::io::Cursor::new(data);
    let result = FileHeader::check_identification(&mut reader, ASSET_FILE_MAGIC_NUMBERS);
    if let Err(err) = result {
        return Err(err);
    }

    let length = match FileHeader::get_header_encoded_data_length(&mut reader, endian_type) {
        Ok(length) => length,
        Err(err) => return Err(err),
    };

    let asset_header: AssetHeader = match FileHeader::get_header2(&mut reader, endian_type) {
        Ok(asset_header) => asset_header,
        Err(err) => return Err(err),
    };

    if let Some(expected_resource_type) = expected_resource_type {
        if asset_header.resource_type != expected_resource_type {
            return Err(crate::error::Error::ResourceTypeNotMatch);
        }
    }

    let offset = length + ASSET_FILE_MAGIC_NUMBERS.len() as u64 + HEADER_LENGTH_SIZE as u64;

    let current_position = match reader.stream_position() {
        Ok(current_position) => current_position,
        Err(err) => {
            return Err(crate::error::Error::IO(
                err,
                Some(String::from("Can not get stream position.")),
            ));
        }
    };
    if let Err(err) = reader.seek(std::io::SeekFrom::Start(offset)) {
        return Err(crate::error::Error::IO(
            err,
            Some(format!("Failed to seek {}", offset)),
        ));
    }

    let mut payload: Vec<u8> = vec![];
    if let Err(err) = reader.read_to_end(&mut payload) {
        reader
            .seek(std::io::SeekFrom::Start(current_position))
            .expect("Seek back successfully.");
        return Err(crate::error::Error::IO(
            err,
            Some(format!("Failed to read all bytes.")),
        ));
    }
    reader
        .seek(std::io::SeekFrom::Start(current_position))
        .expect("Seek back successfully.");
    match bincode::deserialize::<T>(&payload) {
        Ok(asset) => return Ok(asset),
        Err(err) => {
            return Err(crate::error::Error::Bincode(
                err,
                Some(String::from("Fail to deserialize.")),
            ));
        }
    }
}
