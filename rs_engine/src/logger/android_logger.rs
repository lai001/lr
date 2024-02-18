use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, RwLock};

const TAG: &str = "RS_ENGINE\0";

#[derive(Debug, Clone)]
pub struct LoggerConfiguration {
    pub is_write_to_file: bool,
}

pub struct Logger {
    world_file: Arc<RwLock<Option<BufWriter<File>>>>,
}

impl Logger {
    pub fn new(cfg: LoggerConfiguration) -> Logger {
        let mut buf_writer: Option<BufWriter<File>> = None;
        if cfg.is_write_to_file {
            let writer = (|| {
                let _ = std::fs::create_dir_all("/data/local/tmp/rs")?;
                let file = std::fs::File::create(format!(
                    "/data/local/tmp/rs/{}.log",
                    chrono::Local::now().format("%Y_%m_%d-%H_%M_%S")
                ))?;
                std::io::Result::Ok(std::io::BufWriter::new(file))
            })();
            match writer {
                Ok(writer) => {
                    buf_writer = Some(writer);
                }
                Err(err) => unsafe {
                    let msg = err.to_string();
                    ndk_sys::__android_log_print(
                        ndk_sys::android_LogPriority::ANDROID_LOG_WARN.0 as std::os::raw::c_int,
                        TAG.as_ptr() as *const ::std::os::raw::c_char,
                        msg.as_ptr() as *const ::std::os::raw::c_char,
                    );
                },
            }
        }
        let world_file = Arc::new(std::sync::RwLock::new(buf_writer));

        let config = android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .format({
                let world_file = world_file.clone();
                move |buf, record| {
                    if !record.target().starts_with("rs_") {
                        return Err(std::fmt::Error {});
                    }
                    let current_thread = std::thread::current();
                    let thread_name = format!("{}", current_thread.name().unwrap_or("Unknown"));
                    let content = format!(
                        "[{}] {}:{} {}",
                        thread_name,
                        record.file().unwrap_or("Unknown"),
                        record.line().unwrap_or(0),
                        record.args()
                    );
                    let writer = world_file.write();
                    match writer {
                        Ok(mut writer) => {
                            if writer.is_some() {
                                let _ = writer
                                    .as_mut()
                                    .unwrap()
                                    .write_fmt(format_args!("{}\n", content));
                            }
                        }
                        Err(_) => {}
                    }
                    writeln!(buf, "{}", content)
                }
            });

        android_logger::init_once(config);
        Logger { world_file }
    }

    pub fn flush(&self) {
        match self.world_file.write() {
            Ok(mut writer) => {
                if writer.is_some() {
                    let _ = writer.as_mut().unwrap().flush();
                }
            }
            Err(err) => unsafe {
                let msg = err.to_string();
                ndk_sys::__android_log_print(
                    ndk_sys::android_LogPriority::ANDROID_LOG_WARN.0 as std::os::raw::c_int,
                    TAG.as_ptr() as *const ::std::os::raw::c_char,
                    msg.as_ptr() as *const ::std::os::raw::c_char,
                );
            },
        }
    }
}
