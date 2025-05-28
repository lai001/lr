use rs_artifact::EEndianType;

pub type DataLengthType = u32;
pub const ENDIAN_TYPE_UNIT_SIZE: u8 = 1;

pub struct Header {
    pub endian_type: EEndianType,
    pub data_length: DataLengthType,
}

pub struct Message {
    pub header: Header,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(header: Header) -> Message {
        let data = Vec::with_capacity(header.data_length as usize);
        Message { header, data }
    }

    pub fn is_full(&self) -> bool {
        self.data.capacity() == self.data.len()
    }

    pub fn fill_data(&mut self, data: &[u8]) -> usize {
        let need = self.data.capacity() - self.data.len();
        let read = need.min(data.len());
        self.data.append(&mut data[0..read].to_vec());
        read
    }
}

pub trait Encoder {
    fn encode(&self, data: &[u8]) -> crate::error::Result<Vec<u8>>;
}

pub trait Decoder {
    fn decode(&mut self, data: Vec<u8>) -> crate::error::Result<()>;
    fn get_messages(&self) -> &[Message];
    fn take_messages(&mut self) -> Vec<Message>;
}
