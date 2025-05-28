use rs_artifact::EEndianType;

use crate::codec::Encoder;

pub const START_CODE_PREFIX: &[u8] = &[0x00, 0x00, 0x00, 0x01];
// const SHORT_START_CODE: &[u8] = &[0x00, 0x00, 0x01];

pub struct AnnexBEncoder {
    endian_type: EEndianType,
}

impl AnnexBEncoder {
    pub fn new(endian_type: EEndianType) -> Self {
        Self { endian_type }
    }

    pub fn add_emulation_prevention_bytes(data: &[u8], result: &mut Vec<u8>) {
        let mut zero_count = 0;

        for &byte in data {
            if zero_count >= 2 && byte <= 0x03 {
                result.push(0x03);
                zero_count = 0;
            }
            result.push(byte);
            zero_count = if byte == 0x00 { zero_count + 1 } else { 0 };
        }
    }
}

impl Encoder for AnnexBEncoder {
    fn encode(&self, data: &[u8]) -> crate::error::Result<Vec<u8>> {
        let type_value: [u8; 1] = [match self.endian_type {
            EEndianType::Big => 0,
            EEndianType::Little => 1,
            EEndianType::Native => 2,
        }];
        let mut encoded_data: Vec<u8> =
            Vec::with_capacity(START_CODE_PREFIX.len() + type_value.len() + data.len());
        encoded_data.append(&mut START_CODE_PREFIX.to_vec());
        encoded_data.append(&mut type_value.to_vec());
        AnnexBEncoder::add_emulation_prevention_bytes(&data, &mut encoded_data);
        Ok(encoded_data)
    }
}

#[cfg(test)]
mod test {
    use crate::{annex_b_decoder::AnnexBDecoder, annex_b_encoder::AnnexBEncoder};

    #[test]
    fn test_case() {
        let data: Vec<u8> = vec![0, 0, 0, 0, 0, 1];
        let mut new_data = Vec::with_capacity(data.len());
        AnnexBEncoder::add_emulation_prevention_bytes(&data, &mut new_data);
        assert_eq!(new_data, [0, 0, 3, 0, 0, 3, 0, 1]);
        let reverse_data = AnnexBDecoder::remove_emulation_prevention_bytes(&new_data);
        assert_eq!(reverse_data, data);
    }
}
