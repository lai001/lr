#[derive(Debug, Clone, Copy)]
pub enum EWhenceType {
    AvseekSize = 65536,
    SeekSet = 0,
    SeekCur = 1,
    SeekEnd = 2,
}

impl EWhenceType {
    pub fn from_value(v: i32) -> EWhenceType {
        match v {
            65536 => Self::AvseekSize,
            0 => Self::SeekSet,
            1 => Self::SeekCur,
            2 => Self::SeekEnd,
            _ => panic!(),
        }
    }
}

impl TryFrom<i32> for EWhenceType {
    type Error = String;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            65536 => Ok(Self::AvseekSize),
            0 => Ok(Self::SeekSet),
            1 => Ok(Self::SeekCur),
            2 => Ok(Self::SeekEnd),
            _ => Err(String::from("Not support")),
        }
    }
}

pub trait StreamIO {
    fn read_packet(&mut self, buffer: &mut [u8]) -> i32;
    fn write_packet(&mut self, buffer: &mut [u8]) -> i32;
    fn seek(&mut self, offset: i64, whence: EWhenceType) -> i64;
}

pub struct MediaStream {
    pub data: Vec<u8>,
    pos: i64,
}

impl MediaStream {
    pub fn new(data: Vec<u8>) -> MediaStream {
        MediaStream { data, pos: 0 }
    }
}

impl StreamIO for MediaStream {
    fn read_packet(&mut self, buffer: &mut [u8]) -> i32 {
        let len = buffer.len();
        let avaliable_read = (self.data.len() - self.pos as usize).max(0).min(len);
        if avaliable_read == 0 {
            return 0;
        }
        let range = self.pos as usize..self.pos as usize + avaliable_read;
        buffer[0..avaliable_read].copy_from_slice(&self.data[range.clone()]);
        self.pos += avaliable_read as i64;
        avaliable_read as i32
    }

    fn write_packet(&mut self, buffer: &mut [u8]) -> i32 {
        let _ = buffer;
        unimplemented!()
    }

    fn seek(&mut self, offset: i64, whence: EWhenceType) -> i64 {
        match whence {
            EWhenceType::AvseekSize => {
                return self.data.len() as i64;
            }
            EWhenceType::SeekSet => {
                self.pos = offset;
            }
            EWhenceType::SeekCur => {
                self.pos += offset;
            }
            EWhenceType::SeekEnd => {
                self.pos = self.data.len() as i64 - offset;
            }
        }
        self.pos
    }
}
