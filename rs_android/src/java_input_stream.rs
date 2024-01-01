use jni::objects::{GlobalRef, JValueGen};
use std::io::{Read, Seek};

pub struct JavaInputStream {
    jvm: jni::JavaVM,
    input_stream: GlobalRef,
    position: i64,
    input_length: i32,
}

impl JavaInputStream {
    pub fn new(env: jni::JNIEnv, input_stream: jni::objects::JObject) -> Option<JavaInputStream> {
        let jvm = match env.get_java_vm() {
            Ok(jvm) => jvm,
            Err(err) => {
                log::warn!("{err}");
                return None;
            }
        };
        let mut env = match jvm.attach_current_thread() {
            Ok(env) => env,
            Err(err) => {
                log::warn!("{err}");
                return None;
            }
        };
        let input_stream = match env.new_global_ref(input_stream) {
            Ok(input_stream) => input_stream,
            Err(err) => {
                log::warn!("{err}");
                return None;
            }
        };
        let _ = match env.call_method(input_stream.clone(), "reset", "()V", &[]) {
            Ok(_) => {}
            Err(err) => {
                log::warn!("{err}");
                return None;
            }
        };
        let available = match env.call_method(input_stream.clone(), "available", "()I", &[]) {
            Ok(available) => available,
            Err(err) => {
                log::warn!("{err}");
                return None;
            }
        };
        let JValueGen::Int(available) = available else {
            log::warn!("Type error.");
            return None;
        };
        return Some(JavaInputStream {
            jvm: env.get_java_vm().unwrap(),
            input_stream: input_stream.clone(),
            position: 0,
            input_length: available,
        });
    }

    fn seek_start(&mut self, pos: u64) -> std::io::Result<u64> {
        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(err) => {
                log::warn!("attach_current_thread {err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };
        if pos < self.input_length as u64 {
            if pos > self.position as u64 {
                if let Ok(JValueGen::Long(_)) = env.call_method(
                    self.input_stream.clone(),
                    "skip",
                    "(J)J",
                    &[JValueGen::Long(pos as i64 - self.position)],
                ) {
                    self.position = pos as i64;
                    return Ok(self.position as u64);
                }
            } else if pos < self.position as u64 {
                if let Ok(_) = env.call_method(self.input_stream.clone(), "reset", "()V", &[]) {
                    if let Ok(JValueGen::Long(_)) = env.call_method(
                        self.input_stream.clone(),
                        "skip",
                        "(J)J",
                        &[JValueGen::Long(pos as i64)],
                    ) {
                        self.position = pos as i64;
                        return Ok(self.position as u64);
                    }
                }
            } else {
                return Ok(self.position as u64);
            }
        }
        return Err(std::io::ErrorKind::InvalidInput.into());
    }
}

impl Seek for JavaInputStream {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Start(pos) => {
                return self.seek_start(pos);
            }
            std::io::SeekFrom::End(pos) => {
                todo!()
            }
            std::io::SeekFrom::Current(pos) => {
                return self.seek_start((self.position + pos) as u64);
            }
        }
    }
}

impl Read for JavaInputStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut env = match self.jvm.attach_current_thread() {
            Ok(env) => env,
            Err(err) => {
                log::warn!("{err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };

        let available = match env.call_method(self.input_stream.clone(), "available", "()I", &[]) {
            Ok(available) => {
                if let JValueGen::Int(available) = available {
                    available
                } else {
                    return Err(std::io::ErrorKind::InvalidInput.into());
                }
            }
            Err(err) => {
                log::warn!("{err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };
        let byte_array = match env.new_byte_array(buf.len().min(available as usize) as i32) {
            Ok(byte_array) => byte_array,
            Err(err) => {
                log::warn!("{err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };

        let actual_read = match env.call_method(
            self.input_stream.clone(),
            "read",
            "([B)I",
            &[JValueGen::Object(&byte_array)],
        ) {
            Ok(actual_read) => {
                if let JValueGen::Int(actual_read) = actual_read {
                    actual_read
                } else {
                    return Err(std::io::ErrorKind::InvalidInput.into());
                }
            }
            Err(err) => {
                log::warn!("{err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };

        let rs_buf = match env.convert_byte_array(byte_array) {
            Ok(buffer) => buffer,
            Err(err) => {
                log::warn!("{err}");
                return Err(std::io::ErrorKind::InvalidInput.into());
            }
        };
        let range = 0..(rs_buf.len().min(buf.len()));
        buf[range.clone()].copy_from_slice(&rs_buf[range.clone()]);
        self.position += actual_read as i64;
        return Ok(actual_read as usize);
    }
}
