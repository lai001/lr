use super::{LoggerConfiguration, SlotFlags};
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, RwLock};

const TAG: &str = "RS_ENGINE\0";

pub struct Logger {
    cfg: LoggerConfiguration,
    world_file: Arc<RwLock<Option<BufWriter<File>>>>,
    white_list: MultipleThreadMutType<HashSet<String>>,
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
        let white_list = MultipleThreadMut::new(HashSet::new());

        let config = android_logger::Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .format({
                let world_file = world_file.clone();
                let slot_flags = cfg.slot_flags.clone();
                let white_list = white_list.clone();
                move |buf, record| {
                    let white_list = {
                        let list = white_list.lock().unwrap();
                        list.clone()
                    };
                    let is_in_white_list = {
                        let mut ret = false;
                        for name in white_list {
                            if record.target().starts_with(&name) {
                                ret = true;
                                break;
                            }
                        }
                        ret
                    };
                    let level = record.level();
                    if !(record.target().starts_with("rs_")
                        || is_in_white_list
                        || level <= log::Level::Warn)
                    {
                        return Err(std::fmt::Error {});
                    }
                    let current_thread = std::thread::current();
                    let thread_name = format!("{}", current_thread.name().unwrap_or("Unknown"));
                    let content = Self::make_final_output(&slot_flags, record, &thread_name);
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
        Logger {
            cfg,
            world_file,
            white_list,
        }
    }

    fn make_final_output(
        slot_flags: &SlotFlags,
        record: &log::Record<'_>,
        thread_name: &str,
    ) -> String {
        let mut final_output = "".to_string();
        if slot_flags.contains(SlotFlags::ThreadName) {
            final_output.push_str(&format!("[{}] ", thread_name));
        }
        if slot_flags.contains(SlotFlags::FileLine) {
            final_output.push_str(&format!(
                "{}:{} ",
                record.file().unwrap_or("Unknown"),
                record.line().unwrap_or(0)
            ));
        }
        final_output.push_str(&record.args().to_string());
        final_output
    }

    pub fn add_white_list(&mut self, name: String) {
        let mut list = self.white_list.lock().unwrap();
        list.insert(name);
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

    pub fn config_log_to_file(&mut self, is_enable: bool) {
        self.cfg.is_write_to_file = is_enable;

        let mut buf_writer: Option<BufWriter<File>> = None;
        if self.cfg.is_write_to_file {
            let writer = (|| {
                let _ = std::fs::create_dir_all("./log")?;
                let file = std::fs::File::create(format!(
                    "./log/{}.log",
                    chrono::Local::now().format("%Y_%m_%d-%H_%M_%S")
                ))?;
                std::io::Result::Ok(std::io::BufWriter::new(file))
            })();
            match writer {
                Ok(writer) => {
                    buf_writer = Some(writer);
                }
                Err(err) => {
                    println!("{err}");
                }
            }
        }
        let mut file = self.world_file.write().unwrap();
        *file = buf_writer;
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        if self.cfg.is_flush_before_drop {
            self.flush();
        }
    }
}
