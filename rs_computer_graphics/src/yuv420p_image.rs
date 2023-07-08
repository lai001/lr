use std::{io::Read, path::Path};

pub struct YUV420pImage {
    y_buffer: Vec<u8>,
    u_buffer: Vec<u8>,
    v_buffer: Vec<u8>,
    size: glam::UVec2,
}

impl YUV420pImage {
    pub fn from_file(path: &Path, size: &glam::UVec2) -> Option<YUV420pImage> {
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut buffer = Vec::new();
            if let Ok(_) = file.read_to_end(&mut buffer) {
                return Some(Self::from_buffer(&buffer, size));
            }
        }
        return None;
    }

    pub fn from_buffer(buffer: &[u8], size: &glam::UVec2) -> YUV420pImage {
        let length = size.x * size.y;
        assert!(length > 0);
        assert_eq!(length % 4, 0);
        let y_range = 0..length as usize;
        let u_range = y_range.end..y_range.end + (length / 4) as usize;
        let v_range = u_range.end..u_range.end + (length / 4) as usize;
        YUV420pImage {
            y_buffer: (&buffer[y_range]).to_vec(),
            u_buffer: (&buffer[u_range]).to_vec(),
            v_buffer: (&buffer[v_range]).to_vec(),
            size: *size,
        }
    }

    pub fn get_size(&self) -> glam::UVec2 {
        self.size
    }

    pub fn get_y_buffer(&self) -> &[u8] {
        &self.y_buffer
    }

    pub fn get_u_buffer(&self) -> &[u8] {
        &self.u_buffer
    }

    pub fn get_v_buffer(&self) -> &[u8] {
        &self.v_buffer
    }
}
