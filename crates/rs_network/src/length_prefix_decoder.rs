use crate::codec::{DataLengthType, Decoder, Header, Message};
use rs_artifact::EEndianType;

pub struct LengthPrefixDecoder {
    messages: Vec<Message>,
    message: Option<Message>,
    buffer: Vec<u8>,
}

impl LengthPrefixDecoder {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            buffer: Vec::new(),
            message: None,
        }
    }

    fn trim_header(buffer: &mut Vec<u8>) {
        buffer.drain(0..1 + size_of::<DataLengthType>());
    }

    fn read_header(data: &[u8]) -> crate::error::Result<Header> {
        if data.len() < 1 + size_of::<DataLengthType>() {
            return Err(crate::error::Error::Other(Some(format!(
                "Data is too small"
            ))));
        } else {
            let endian_type = EEndianType::try_from_u8_value(data[0]).ok_or(
                crate::error::Error::Other(Some(format!("Not a valid endian type"))),
            )?;
            let data_length_bytes: [u8; size_of::<DataLengthType>()] = data
                [1..1 + size_of::<DataLengthType>()]
                .try_into()
                .map_err(|err| crate::error::Error::TryFromSliceError(err))?;
            let data_length = match endian_type {
                EEndianType::Big => DataLengthType::from_be_bytes(data_length_bytes),
                EEndianType::Little => DataLengthType::from_le_bytes(data_length_bytes),
                EEndianType::Native => DataLengthType::from_ne_bytes(data_length_bytes),
            };

            return Ok(Header {
                endian_type,
                data_length,
            });
        }
    }
}

impl Decoder for LengthPrefixDecoder {
    fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    fn take_messages(&mut self) -> Vec<Message> {
        self.messages.drain(..).collect()
    }

    fn decode(&mut self, mut data: Vec<u8>) -> crate::error::Result<()> {
        self.buffer.append(&mut data);
        loop {
            if let Some(message) = &mut self.message {
                let fill = message.fill_data(&self.buffer);
                self.buffer.drain(0..fill);
                if message.is_full() {
                    self.messages.push(self.message.take().unwrap());
                }
            } else {
                match Self::read_header(&self.buffer) {
                    Ok(header) => {
                        Self::trim_header(&mut self.buffer);
                        self.message = Some(Message::new(header));
                        continue;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            if self.buffer.is_empty() {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::LengthPrefixDecoder;
    use crate::{
        codec::{Decoder, Encoder},
        length_prefix_encoder::LengthPrefixEncoder,
    };
    use rs_artifact::EEndianType;

    #[test]
    fn test_case() {
        let encoder = LengthPrefixEncoder::new(EEndianType::current());
        let mut decoder = LengthPrefixDecoder::new();
        let test_datas: Vec<Vec<u8>> = vec![vec![0], vec![0, 1], vec![0, 1, 2]];
        let mut encoded: Vec<u8> = Vec::new();
        for test_data in &test_datas {
            encoded.append(&mut encoder.encode(test_data).unwrap());
        }

        for iter in encoded.chunks(3) {
            let _ = decoder.decode(iter.to_vec());
        }

        for (lhs, rhs) in decoder.messages.iter().zip(test_datas) {
            assert_eq!(lhs.data, rhs);
        }
    }
}
