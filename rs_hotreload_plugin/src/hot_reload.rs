use crate::error::Result;
use crate::library_reload::LibraryReload;
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct HotReload {
    library_reload: Arc<Mutex<LibraryReload>>,
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
    debouncer: Debouncer<ReadDirectoryChangesWatcher>,
}

impl HotReload {
    pub fn new(watch_folder_path: &Path, lib_folder: &Path, lib_name: &str) -> HotReload {
        let mut library_reload = LibraryReload::new(&lib_folder, lib_name);
        library_reload.clean_cache();
        let first_load_result = library_reload.reload();
        log::trace!("{:?}", first_load_result);
        let library_reload = Arc::new(Mutex::new(library_reload));
        let (sender, receiver) = std::sync::mpsc::channel();
        log::trace!("Watch {:?}", watch_folder_path);
        let mut debouncer =
            new_debouncer(std::time::Duration::from_millis(200), None, sender).unwrap();

        let _ = debouncer.watcher().watch(
            &Path::new(watch_folder_path),
            notify::RecursiveMode::Recursive,
        );
        let reload = HotReload {
            library_reload,
            receiver,
            debouncer,
        };
        reload
    }

    pub fn get_library_reload(&self) -> Arc<Mutex<LibraryReload>> {
        self.library_reload.clone()
    }

    pub fn reload_if_need(&mut self) -> bool {
        for events in self.receiver.try_iter() {
            match events {
                Ok(events) => {
                    let mut library_reload = self.library_reload.lock().unwrap();
                    let file_path = library_reload.get_original_lib_file_path();
                    for event in events {
                        if file_path == event.path {
                            let _ = library_reload.reload();
                            return true;
                        }
                    }
                }
                Err(errors) => {}
            }
        }
        return false;
    }
}

#[cfg(test)]
pub mod test {
    use super::HotReload;

    #[test]
    pub fn test_case() {
        let binding = std::env::current_exe().unwrap();
        let work_dir = std::path::Path::new(&binding).parent().unwrap();
        std::env::set_current_dir(&work_dir).unwrap();

        crate::library_reload::test::compile_test_lib(
            work_dir,
            r"#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right
}",
        );

        let mut hot_reload = HotReload::new(work_dir, work_dir, "test");
        {
            let binding = hot_reload.get_library_reload();
            let lib = binding.lock().unwrap();
            assert_eq!(lib.is_loaded(), true);
            let add_func = lib.load_symbol::<fn(usize, usize) -> usize>("add").unwrap();
            assert_eq!(add_func(1, 1), 2);
        }

        crate::library_reload::test::compile_test_lib(
            work_dir,
            r"#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right + 1
}",
        );

        {
            let binding = hot_reload.get_library_reload();
            let mut lib = binding.lock().unwrap();
            lib.reload().unwrap();
            let add_func = lib.load_symbol::<fn(usize, usize) -> usize>("add").unwrap();
            assert_eq!(add_func(1, 1), 3);
        }
        assert_eq!(hot_reload.reload_if_need(), false);
    }
}
