use crate::error::Result;
use crate::library_reload::LibraryReload;
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct HotReload {
    library_reload: Arc<Mutex<LibraryReload>>,
    receiver: Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>,
    _debouncer: Debouncer<ReadDirectoryChangesWatcher>,
}

impl HotReload {
    pub fn new(watch_folder_path: &Path, lib_folder: &Path, lib_name: &str) -> Result<HotReload> {
        let library_reload = LibraryReload::new(&lib_folder, lib_name);
        library_reload.clean_cache();
        let library_reload = Arc::new(Mutex::new(library_reload));
        let (sender, receiver) = std::sync::mpsc::channel();
        log::trace!("Watch {:?}", watch_folder_path);
        let mut debouncer = new_debouncer(std::time::Duration::from_millis(200), sender)
            .map_err(|err| crate::error::Error::Debouncer(err))?;

        debouncer
            .watcher()
            .watch(
                &Path::new(watch_folder_path),
                notify::RecursiveMode::Recursive,
            )
            .map_err(|err| crate::error::Error::Debouncer(err))?;
        let reload = HotReload {
            library_reload,
            receiver,
            _debouncer: debouncer,
        };
        Ok(reload)
    }

    pub fn get_library_reload(&self) -> Arc<Mutex<LibraryReload>> {
        self.library_reload.clone()
    }

    pub fn is_need_reload(&self) -> bool {
        let library_reload = self.library_reload.lock().unwrap();
        for events in self.receiver.try_iter() {
            let Ok(events) = events else {
                continue;
            };
            let file_path = library_reload.get_original_lib_file_path();
            for event in events {
                if file_path == event.path {
                    return true;
                }
            }
        }
        false
    }

    pub fn reload(&mut self) -> Result<()> {
        let mut library_reload = self.library_reload.lock().unwrap();
        library_reload.reload()
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
            "abc",
        );

        let hot_reload = HotReload::new(work_dir, work_dir, "abc").unwrap();
        {
            let binding = hot_reload.get_library_reload();
            let lib = binding.lock().unwrap();
            assert_eq!(lib.is_loaded(), false);
            let add_func = lib.load_symbol::<fn(usize, usize) -> usize>("add");
            assert!(add_func.is_err());
        }

        crate::library_reload::test::compile_test_lib(
            work_dir,
            r"#[no_mangle]
pub fn add(left: usize, right: usize) -> usize {
    left + right + 1
}",
            "abc",
        );

        {
            let binding = hot_reload.get_library_reload();
            let mut lib = binding.lock().unwrap();
            lib.reload().unwrap();
            let add_func = lib.load_symbol::<fn(usize, usize) -> usize>("add").unwrap();
            assert_eq!(add_func(1, 1), 3);
        }
        assert_eq!(hot_reload.is_need_reload(), false);
    }
}
