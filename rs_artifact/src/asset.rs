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
        FileHeader::write_header(ASSET_FILE_MAGIC_NUMBERS, &asset_header, endian_type)?;
    let payload = bincode::serialize(asset)
        .map_err(|err| crate::error::Error::Bincode(err, Some(format!("Fail to serialize."))))?;
    let mut data = vec![0; header_data.len() + payload.len()];
    data[0..header_data.len()].copy_from_slice(&header_data);
    data[header_data.len()..].copy_from_slice(&payload);
    Ok(data)
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
    let _ = FileHeader::check_identification(&mut reader, ASSET_FILE_MAGIC_NUMBERS)?;
    let length = FileHeader::get_header_encoded_data_length(&mut reader, endian_type)?;
    let asset_header: AssetHeader = FileHeader::get_header2(&mut reader, endian_type)?;
    if let Some(expected_resource_type) = expected_resource_type {
        if asset_header.resource_type != expected_resource_type {
            return Err(crate::error::Error::ResourceTypeNotMatch);
        }
    }
    let offset = length + ASSET_FILE_MAGIC_NUMBERS.len() as u64 + HEADER_LENGTH_SIZE as u64;
    let _ = reader
        .seek(std::io::SeekFrom::Start(offset))
        .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to seek {}", offset))));
    let mut payload: Vec<u8> = vec![];
    let _ = reader
        .read_to_end(&mut payload)
        .map_err(|err| crate::error::Error::IO(err, Some(format!("Failed to read all bytes."))))?;
    let asset = bincode::deserialize::<T>(&payload).map_err(|err| {
        crate::error::Error::Bincode(err, Some(String::from("Fail to deserialize.")))
    })?;
    Ok(asset)
}
