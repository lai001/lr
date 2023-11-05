use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::{Arc, RwLock},
};

pub struct Logger {
    world_file: Arc<RwLock<BufWriter<File>>>,
}

impl Logger {
    pub fn new() -> Logger {
        std::fs::create_dir_all("./log").unwrap();
        let world_file = Arc::new(std::sync::RwLock::new(std::io::BufWriter::new(
            std::fs::File::create(format!(
                "./log/{}.log",
                chrono::Local::now().format("%Y_%m_%d-%H_%M_%S")
            ))
            .unwrap(),
        )));
        let log_env = env_logger::Env::default()
            .default_filter_or("rs_computer_graphics,rs_dotnet,rs_media,rs_metis");
        env_logger::Builder::from_env(log_env)
            .format({
                let world_file = world_file.clone();
                move |buf, record| {
                    let mut writer = world_file.write().unwrap();
                    let level = record.level();
                    let level_style = buf.default_level_style(level);
                    let current_thread = std::thread::current();
                    let thread_name =
                        format!("Thread: {}", current_thread.name().unwrap_or("Unknown"));
                    let content = format!(
                        "[{}][{}] {}:{} {} {}",
                        level_style.value(level),
                        thread_name,
                        record.file().unwrap_or("Unknown"),
                        record.line().unwrap_or(0),
                        buf.timestamp_millis(),
                        record.args()
                    );
                    let _ = writer.write_fmt(format_args!("{}\n", content));
                    writeln!(buf, "{}", content)
                }
            })
            .init();
        Logger { world_file }
    }

    pub fn flush(&self) {
        let mut writer = self.world_file.write().unwrap();
        let _ = writer.flush();
    }
}
