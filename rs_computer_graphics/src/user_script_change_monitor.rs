use crate::project::ProjectDescription;
use notify_debouncer_mini::new_debouncer;
use std::sync::{Arc, Mutex};

pub struct UserScriptChangeMonitor {
    is_source_file_changed: Arc<Mutex<bool>>,
}

impl UserScriptChangeMonitor {
    pub fn new() -> UserScriptChangeMonitor {
        let is_source_file_changed = Arc::new(Mutex::new(false));
        let is_source_file_changed_clone = Arc::clone(&is_source_file_changed);
        let user_script_path = ProjectDescription::default()
            .lock()
            .unwrap()
            .get_user_script()
            .path
            .clone();
        {
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut debouncer =
                    new_debouncer(std::time::Duration::from_millis(200), None, sender).unwrap();
                let _ = debouncer.watcher().watch(
                    std::path::Path::new(&user_script_path),
                    notify::RecursiveMode::NonRecursive,
                );
                for events in receiver {
                    log::info!("Request to rebuild script.");
                    match events {
                        Ok(_) => {
                            let mut is_source_file_changed =
                                is_source_file_changed_clone.lock().unwrap();
                            *is_source_file_changed = true;
                        }
                        Err(error) => log::error!("{:?}", error),
                    }
                }
            });
        }
        UserScriptChangeMonitor {
            is_source_file_changed,
        }
    }

    pub fn is_changed(&mut self) -> bool {
        let is_source_file_changed_clone = Arc::clone(&self.is_source_file_changed);
        let mut is_source_file_changed = is_source_file_changed_clone.lock().unwrap();
        if *is_source_file_changed == true {
            *is_source_file_changed = false;
            return true;
        }
        return false;
    }
}
