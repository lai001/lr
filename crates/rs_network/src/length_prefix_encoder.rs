use crate::codec::{DataLengthType, Encoder};
use rs_artifact::EEndianType;

pub struct LengthPrefixEncoder {
    endian_type: EEndianType,
}

impl LengthPrefixEncoder {
    pub fn new(endian_type: EEndianType) -> LengthPrefixEncoder {
        LengthPrefixEncoder { endian_type }
    }
}

impl Encoder for LengthPrefixEncoder {
    fn encode(&self, data: &[u8]) -> crate::error::Result<Vec<u8>> {
        let type_value: [u8; 1] = [match self.endian_type {
            EEndianType::Big => 0,
            EEndianType::Little => 1,
            EEndianType::Native => 2,
        }];
        let data_length: DataLengthType = data.len() as DataLengthType;
        let data_length_bytes = match self.endian_type {
            EEndianType::Big => data_length.to_be_bytes(),
            EEndianType::Little => data_length.to_le_bytes(),
            EEndianType::Native => data_length.to_ne_bytes(),
        };
        let mut datas = Vec::new();
        datas.resize(type_value.len() + data_length_bytes.len() + data.len(), 0);
        datas[0..1].copy_from_slice(&type_value);
        datas[1..1 + size_of::<DataLengthType>()].copy_from_slice(&data_length_bytes);
        datas[1 + size_of::<DataLengthType>()..].copy_from_slice(data);
        Ok(datas)
    }
}
