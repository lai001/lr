use std::io::Write;

pub fn setup_log() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Trace);
    builder
        .format({
            move |buf, record| {
                if record.target() != "rs_shader_compiler_lsp" {
                    return Err(std::io::ErrorKind::Other.into());
                }
                let level = record.level();
                let level_style = buf.default_level_style(level);
                let current_thread = std::thread::current();
                let thread_name = format!("Thread: {}", current_thread.name().unwrap_or("Unknown"));
                writeln!(
                    buf,
                    "{} [{level_style}{}{level_style:#}] [{}] {}:{} {}",
                    buf.timestamp_millis(),
                    level,
                    thread_name,
                    record.file().unwrap_or("Unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            }
        })
        .init();
}

fn main() {
    setup_log();
    rs_shader_compiler_lsp::server::Server::new().run();
}
