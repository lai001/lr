use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone, Default)]
pub struct LoggerConfiguration {
    pub is_write_to_file: bool,
    pub is_flush_before_drop: bool,
}

pub struct Logger {
    world_file: Arc<RwLock<Option<BufWriter<File>>>>,
    cfg: LoggerConfiguration,
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

        let mut builder = env_logger::Builder::new();
        builder.write_style(env_logger::WriteStyle::Auto);
        builder.filter_level(log::LevelFilter::Trace);

        // let log_env = env_logger::Env::default();
        // let mut builder = env_logger::Builder::from_env(log_env);

        builder
            .format({
                let world_file = world_file.clone();
                move |buf, record| {
                    if !record.target().starts_with("rs_") {
                        return Err(std::io::ErrorKind::Other.into());
                    }
                    let level = record.level();
                    let level_style = buf.default_level_style(level);
                    let current_thread = std::thread::current();
                    let thread_name = format!("{}", current_thread.name().unwrap_or("Unknown"));
                    let content = format!(
                        "{} [{}] [{}] {}:{} {}",
                        buf.timestamp_millis(),
                        level_style.value(level),
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
                    writeln!(
                        buf,
                        "{} [{}] [{}] {}:{} {}",
                        buf.timestamp_millis(),
                        level_style.value(level),
                        thread_name,
                        record.file().unwrap_or("Unknown"),
                        record.line().unwrap_or(0),
                        record.args()
                    )
                }
            })
            .init();
        Logger { world_file, cfg }
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
}

impl Drop for Logger {
    fn drop(&mut self) {
        if self.cfg.is_flush_before_drop {
            self.flush();
        }
    }
}
