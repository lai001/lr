use std::fmt::Display;
use std::sync::{Mutex, OnceLock};
use std::{collections::HashMap, sync::Arc};

struct STTimeTrace {
    label: String,
    begin: std::time::Instant,
    end: std::time::Instant,
}

pub struct TimeTrace {
    inner: Mutex<STTimeTrace>,
}

impl TimeTrace {
    pub fn begin(label: String) -> TimeTrace {
        let now = std::time::Instant::now();
        let inner = STTimeTrace {
            label,
            begin: now,
            end: now,
        };
        TimeTrace {
            inner: Mutex::new(inner),
        }
    }

    pub fn delta_duration(&self) -> std::time::Duration {
        let now = std::time::Instant::now();
        let mut inner = self.inner.lock().unwrap();
        inner.end = now;
        let duration = inner.end - inner.begin;
        inner.begin = now;
        duration
    }

    pub fn get_label(&self) -> String {
        let inner = self.inner.lock().unwrap();
        inner.label.clone()
    }
}

impl Display for TimeTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self.delta_duration();
        write!(f, "{}", format!("{}s", duration.as_secs_f32()))
    }
}

fn get_global_profiler() -> &'static Arc<Profiler> {
    static GLOBAL_PROFILER: OnceLock<Arc<Profiler>> = OnceLock::new();
    GLOBAL_PROFILER.get_or_init(|| Arc::new(Profiler::new()))
}

struct STProfiler {
    traces: HashMap<String, Vec<Arc<TimeTrace>>>,
}

pub struct Profiler {
    inner: Mutex<STProfiler>,
}

impl Profiler {
    pub fn default() -> Arc<Profiler> {
        get_global_profiler().clone()
    }

    fn new() -> Profiler {
        Profiler {
            inner: Mutex::new(STProfiler {
                traces: HashMap::new(),
            }),
        }
    }

    #[track_caller]
    pub fn trace(&self, label: String) -> Arc<TimeTrace> {
        let caller_location = std::panic::Location::caller();
        let time_trace = TimeTrace::begin(label);
        let time_trace = Arc::new(time_trace);
        let key = format!(
            "{}:{}",
            caller_location.file().to_string(),
            caller_location.line()
        );
        let mut inner = self.inner.lock().unwrap();
        match inner.traces.get_mut(&key) {
            Some(value) => {
                value.push(time_trace.clone());
            }
            None => {
                inner.traces.insert(key, vec![time_trace.clone()]);
            }
        }
        time_trace
    }

    #[track_caller]
    pub fn auto_trace(&self) -> Arc<TimeTrace> {
        let caller_location = std::panic::Location::caller();
        let label = format!(
            "{}:{}",
            caller_location.file().to_string(),
            caller_location.line()
        );
        let time_trace = TimeTrace::begin(label.clone());
        let time_trace = Arc::new(time_trace);
        let mut inner = self.inner.lock().unwrap();
        match inner.traces.get_mut(&label) {
            Some(value) => {
                value.push(time_trace.clone());
            }
            None => {
                inner.traces.insert(label, vec![time_trace.clone()]);
            }
        }
        time_trace
    }
}
