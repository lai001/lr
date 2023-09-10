use std::sync::{Mutex, OnceLock};
use std::{collections::HashMap, sync::Arc};

pub struct TimeTrace {
    label: String,
    start: std::time::Instant,
    end: Option<std::time::Instant>,
}

impl TimeTrace {
    pub fn begin(label: String) -> TimeTrace {
        TimeTrace {
            label,
            start: std::time::Instant::now(),
            end: None,
        }
    }

    pub fn end(&mut self) {
        self.end = Some(std::time::Instant::now());
    }

    pub fn dump_end(&mut self) {
        self.end = Some(std::time::Instant::now());
        let duration = self.get_duration_seconds();
        println!("{}", format!("[{}] {}", self.label, duration));
    }

    pub fn get_duration(&mut self) -> std::time::Duration {
        let duration = self.end.unwrap() - self.start;
        duration
    }

    pub fn get_duration_seconds(&mut self) -> f32 {
        let duration = self.end.unwrap() - self.start;
        duration.as_secs_f32()
    }

    pub fn get_duration_millis(&mut self) -> u128 {
        let duration = self.end.unwrap() - self.start;
        duration.as_millis()
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }
}

fn get_global_profiler() -> &'static Mutex<Profiler> {
    static GLOBAL_PROFILER: OnceLock<Mutex<Profiler>> = OnceLock::new();
    GLOBAL_PROFILER.get_or_init(|| Mutex::new(Profiler::new()))
}

pub struct Profiler {
    traces: HashMap<String, Vec<Arc<Mutex<TimeTrace>>>>,
}

impl Profiler {
    pub fn default() -> &'static Mutex<Profiler> {
        get_global_profiler()
    }

    fn new() -> Profiler {
        Profiler {
            traces: HashMap::new(),
        }
    }

    pub fn trace(&mut self, label: String) -> Arc<Mutex<TimeTrace>> {
        let time_trace = TimeTrace::begin(label.clone());
        let time_trace = Arc::new(Mutex::new(time_trace));
        match self.traces.get_mut(&label) {
            Some(value) => {
                value.push(time_trace.clone());
            }
            None => {
                self.traces.insert(label, vec![time_trace.clone()]);
            }
        }
        time_trace
    }

    #[track_caller]
    pub fn auto_trace(&mut self) -> Arc<Mutex<TimeTrace>> {
        let caller_location = std::panic::Location::caller();
        let label = format!(
            "{}:{}",
            caller_location.file().to_string(),
            caller_location.line()
        );
        let time_trace = TimeTrace::begin(label.clone());
        let time_trace = Arc::new(Mutex::new(time_trace));
        match self.traces.get_mut(&label) {
            Some(value) => {
                value.push(time_trace.clone());
            }
            None => {
                self.traces.insert(label, vec![time_trace.clone()]);
            }
        }
        time_trace
    }
}
