use crate::error::Result;
use jni::objects::{GlobalRef, JValueGen};
use std::io::{Read, Seek};

pub struct JavaInputStream {
    jvm: jni::JavaVM,
    input_stream: GlobalRef,
    position: i64,
    input_length: i32,
}

impl JavaInputStream {
    pub fn new(env: jni::JNIEnv, input_stream: jni::objects::JObject) -> Result<JavaInputStream> {
        let jvm = env
            .get_java_vm()
            .map_err(|err| crate::error::Error::Jni(err))?;
        let mut env = jvm
            .attach_current_thread()
            .map_err(|err| crate::error::Error::Jni(err))?;
        let input_stream = env
            .new_global_ref(input_stream)
            .map_err(|err| crate::error::Error::Jni(err))?;
        env.call_method(input_stream.clone(), "reset", "()V", &[])
            .map_err(|err| crate::error::Error::Jni(err))?;
        let available = env
            .call_method(input_stream.clone(), "available", "()I", &[])
            .map_err(|err| crate::error::Error::Jni(err))?;
        let JValueGen::Int(available) = available else {
            return Err(crate::error::Error::ValueTypeNotMatch);
        };
        let jvm = env
            .get_java_vm()
            .map_err(|err| crate::error::Error::Jni(err))?;
        Ok(JavaInputStream {
            jvm,
            input_stream: input_stream.clone(),
            position: 0,
            input_length: available,
        })
    }

    fn seek_start(&mut self, pos: u64) -> std::io::Result<u64> {
        let mut env = self
            .jvm
            .attach_current_thread()
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        if pos >= self.input_length as u64 {
            return Err(std::io::ErrorKind::InvalidInput.into());
        }
        if pos > self.position as u64 {
            let value = env
                .call_method(
                    self.input_stream.clone(),
                    "skip",
                    "(J)J",
                    &[JValueGen::Long(pos as i64 - self.position)],
                )
                .map_err(|_| std::io::ErrorKind::InvalidInput)?;
            let JValueGen::Long(_) = value else {
                return Err(std::io::ErrorKind::InvalidInput.into());
            };
            self.position = pos as i64;
        } else if pos < self.position as u64 {
            env.call_method(self.input_stream.clone(), "reset", "()V", &[])
                .map_err(|_| std::io::ErrorKind::InvalidInput)?;
            let value = env
                .call_method(
                    self.input_stream.clone(),
                    "skip",
                    "(J)J",
                    &[JValueGen::Long(pos as i64)],
                )
                .map_err(|_| std::io::ErrorKind::InvalidInput)?;
            let JValueGen::Long(_) = value else {
                return Err(std::io::ErrorKind::InvalidInput.into());
            };
            self.position = pos as i64;
        }
        Ok(self.position as u64)
    }
}

impl Seek for JavaInputStream {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(pos) => self.seek_start(pos),
            std::io::SeekFrom::End(pos) => {
                todo!()
            }
            std::io::SeekFrom::Current(pos) => self.seek_start((self.position + pos) as u64),
        }
    }
}

impl Read for JavaInputStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut env = self
            .jvm
            .attach_current_thread()
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        let available = env
            .call_method(self.input_stream.clone(), "available", "()I", &[])
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        let available = if let JValueGen::Int(available) = available {
            available
        } else {
            return Err(std::io::ErrorKind::InvalidInput.into());
        };
        let byte_array = env
            .new_byte_array(buf.len().min(available as usize) as i32)
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        let actual_read = env
            .call_method(
                self.input_stream.clone(),
                "read",
                "([B)I",
                &[JValueGen::Object(&byte_array)],
            )
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        let actual_read = if let JValueGen::Int(actual_read) = actual_read {
            actual_read
        } else {
            return Err(std::io::ErrorKind::InvalidInput.into());
        };
        let rs_buf = env
            .convert_byte_array(byte_array)
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        let range = 0..(rs_buf.len().min(buf.len()));
        buf[range.clone()].copy_from_slice(&rs_buf[range.clone()]);
        self.position += actual_read as i64;
        Ok(actual_read as usize)
    }
}
