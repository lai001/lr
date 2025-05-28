use rs_artifact::EEndianType;

use crate::{
    annex_b_encoder::START_CODE_PREFIX,
    codec::{Decoder, Header, Message, ENDIAN_TYPE_UNIT_SIZE},
};

pub struct AnnexBDecoder {
    messages: Vec<Message>,
    buffer: Vec<u8>,
}

impl Decoder for AnnexBDecoder {
    fn decode(&mut self, data: Vec<u8>) -> crate::error::Result<()> {
        self.buffer.append(&mut data.to_vec());

        let mut start_code_ranges = Self::search_start_code(&self.buffer);
        start_code_ranges.insert(0, 0..0);

        for iter in start_code_ranges.windows(2) {
            let range = iter[0].end..iter[1].start;
            if range.is_empty() {
                continue;
            }
            if let Ok(endian_type) = Self::read_endian_type(&data) {
                let decoded_data = Self::remove_emulation_prevention_bytes(
                    &self.buffer[range.start + ENDIAN_TYPE_UNIT_SIZE as usize..range.end],
                );
                let message = Message {
                    header: Header {
                        endian_type,
                        data_length: decoded_data.len() as u32,
                    },
                    data: decoded_data,
                };

                self.messages.push(message);
            }
        }

        if let Some(last_range) = start_code_ranges.last() {
            self.buffer.drain(0..last_range.start);
        }
        Ok(())
    }

    fn get_messages(&self) -> &[crate::codec::Message] {
        &self.messages
    }

    fn take_messages(&mut self) -> Vec<crate::codec::Message> {
        self.messages.drain(..).collect()
    }
}

impl AnnexBDecoder {
    pub fn new() -> AnnexBDecoder {
        AnnexBDecoder {
            messages: Vec::new(),
            buffer: Vec::new(),
        }
    }

    fn read_endian_type(data: &[u8]) -> crate::error::Result<EEndianType> {
        if data.len() < ENDIAN_TYPE_UNIT_SIZE as usize {
            return Err(crate::error::Error::Other(Some(format!(
                "Data is too small"
            ))));
        } else {
            let endian_type = EEndianType::try_from_u8_value(data[0]).ok_or(
                crate::error::Error::Other(Some(format!("Not a valid endian type"))),
            )?;
            return Ok(endian_type);
        }
    }

    pub fn search_start_code(data: &[u8]) -> Vec<std::ops::Range<usize>> {
        let mut ranges = vec![];
        for (start, iter) in data.windows(START_CODE_PREFIX.len()).enumerate() {
            if iter == START_CODE_PREFIX {
                ranges.push(start..start + START_CODE_PREFIX.len());
            }
        }
        ranges
    }

    pub fn remove_emulation_prevention_bytes(data: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(data.len());
        let mut zero_count = 0;

        for &byte in data {
            if zero_count == 2 && byte == 0x03 {
                zero_count = 0;
                continue;
            }
            result.push(byte);
            zero_count = match byte {
                0x00 => zero_count + 1,
                _ => 0,
            };
        }
        result
    }
}

#[cfg(test)]
mod test {
    use crate::{
        annex_b_decoder::AnnexBDecoder,
        annex_b_encoder::AnnexBEncoder,
        codec::{Decoder, Encoder},
    };
    use rs_artifact::EEndianType;

    #[test]
    fn test_case() {
        let data: Vec<u8> = vec![0, 0, 0, 0, 0, 1];
        let encoder = AnnexBEncoder::new(EEndianType::current());
        let mut encoded_data: Vec<u8> = vec![];
        encoded_data.append(&mut encoder.encode(&data).unwrap());
        encoded_data.append(&mut encoder.encode(&data).unwrap());
        let mut decoder = AnnexBDecoder::new();
        let _ = decoder.decode(encoded_data);
        for message in decoder.messages {
            assert_eq!(message.data, data);
        }
        assert_eq!(decoder.buffer.len(), encoder.encode(&data).unwrap().len());
    }
}
