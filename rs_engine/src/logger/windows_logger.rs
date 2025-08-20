use crate::logger::LoggerConfiguration;
use crate::logger::SlotFlags;
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};

pub struct Logger {
    world_file: Arc<RwLock<Option<BufWriter<File>>>>,
    cfg: LoggerConfiguration,
    white_list: MultipleThreadMutType<HashSet<String>>,
}

impl Logger {
    pub fn new(cfg: LoggerConfiguration) -> Logger {
        let mut buf_writer: Option<BufWriter<File>> = None;
        if cfg.is_write_to_file {
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

        let world_file = Arc::new(std::sync::RwLock::new(buf_writer));
        let white_list = MultipleThreadMut::new(HashSet::new());
        let mut builder = env_logger::Builder::new();
        builder.write_style(env_logger::WriteStyle::Auto);
        builder.filter_level(log::LevelFilter::Trace);

        // let log_env = env_logger::Env::default();
        // let mut builder = env_logger::Builder::from_env(log_env);

        builder
            .format({
                let world_file = world_file.clone();
                let white_list = white_list.clone();
                let slot_flags = cfg.slot_flags.clone();
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
                        return Err(std::io::ErrorKind::Other.into());
                    }
                    let level_style = buf.default_level_style(level);
                    let current_thread = std::thread::current();
                    let thread_name = current_thread.name().unwrap_or("Unknown");
                    let writer = world_file.write();
                    match writer {
                        Ok(mut writer) => {
                            if writer.is_some() {
                                let final_output = Self::make_final_output(
                                    &slot_flags,
                                    buf,
                                    record,
                                    &thread_name,
                                    level,
                                    None,
                                );
                                let _ = writer
                                    .as_mut()
                                    .unwrap()
                                    .write_fmt(format_args!("{}\n", final_output));
                                match level {
                                    log::Level::Error | log::Level::Warn => {
                                        let _ = writer.as_mut().unwrap().flush();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    let final_output = Self::make_final_output(
                        &slot_flags,
                        buf,
                        record,
                        &thread_name,
                        level,
                        Some(level_style),
                    );
                    writeln!(buf, "{}", final_output)
                }
            })
            .init();
        Logger {
            world_file,
            cfg,
            white_list,
        }
    }

    fn make_final_output(
        slot_flags: &SlotFlags,
        buf: &mut env_logger::fmt::Formatter,
        record: &log::Record<'_>,
        thread_name: &str,
        level: log::Level,
        level_style: Option<env_logger::fmt::style::Style>,
    ) -> String {
        let mut final_output = "".to_string();
        if slot_flags.contains(SlotFlags::Timestamp) {
            final_output.push_str(&format!("{} ", buf.timestamp_millis()));
        }
        if slot_flags.contains(SlotFlags::Level) {
            if let Some(level_style) = level_style {
                final_output.push_str(&format!("[{level_style}{}{level_style:#}] ", level));
            } else {
                final_output.push_str(&format!("[{}] ", level));
            }
        }
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

    pub fn flush(&self) {
        match self.world_file.write() {
            Ok(mut writer) => {
                if writer.is_some() {
                    let _ = writer.as_mut().unwrap().flush();
                }
            }
            Err(_) => {}
        }
    }

    pub fn add_white_list(&mut self, name: String) {
        let mut list = self.white_list.lock().unwrap();
        list.insert(name);
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
