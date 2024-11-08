use std::{
    env, io,
    path::{Path, PathBuf},
};

pub mod channel;
pub mod id_generator;
pub mod new;
pub mod profiler;

#[repr(C)]
#[derive(Clone, Default, PartialEq, Eq, Hash, Debug)]
pub struct Range<T: Copy> {
    pub start: T,
    pub end: T,
}

impl<T> Range<T>
where
    T: Copy,
{
    pub fn to_std_range(&self) -> std::ops::Range<T> {
        std::ops::Range::<T> {
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: f32,
    pub end: f32,
}

impl TimeRange {
    pub fn is_contains(&self, time: f32) -> bool {
        time >= self.start && time <= self.end
    }
}

pub fn ffi_to_rs_string(c_str: *const std::ffi::c_char) -> Option<String> {
    if c_str.is_null() {
        None
    } else {
        let rs_string = unsafe { std::ffi::CStr::from_ptr(c_str).to_str().unwrap().to_owned() };
        Some(rs_string)
    }
}

pub fn math_remap_value_range(
    value: f64,
    from_range: std::ops::Range<f64>,
    to_range: std::ops::Range<f64>,
) -> f64 {
    (value - from_range.start) / (from_range.end - from_range.start)
        * (to_range.end - to_range.start)
        + to_range.start
}

pub fn cast_any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
    }
}

pub fn get_object_address<T>(object: &T) -> String {
    let raw_ptr = object as *const T;
    std::format!("{:?}", raw_ptr)
}

pub fn cast_to_raw_buffer<'a, T>(vec: &[T]) -> &'a [u8] {
    let type_szie = std::mem::size_of::<T>();
    let buffer = vec.as_ptr() as *const u8;
    let size = type_szie * vec.len();
    let buffer = unsafe { std::slice::from_raw_parts(buffer, size) };
    buffer
}

pub fn cast_to_raw_type_buffer<'a, U>(buffer: *const u8, len: usize) -> &'a [U] {
    unsafe {
        let type_szie = std::mem::size_of::<U>();
        let new_len = len / type_szie;
        if new_len * type_szie != len {
            panic!();
        }
        std::slice::from_raw_parts(buffer as *const U, new_len)
    }
}

pub fn cast_to_type_buffer<'a, U>(buffer: &'a [u8]) -> &'a [U] {
    unsafe {
        let type_szie = std::mem::size_of::<U>();
        let len = buffer.len() / type_szie;
        if len * type_szie != buffer.len() {
            panic!();
        }
        std::slice::from_raw_parts(buffer.as_ptr() as *const U, len)
    }
}

pub fn cast_to_type_vec<U>(mut buffer: Vec<u8>) -> Vec<U> {
    unsafe {
        let type_szie = std::mem::size_of::<U>();
        let len = buffer.len() / type_szie;
        if len * type_szie != buffer.len() {
            panic!();
        }
        std::vec::Vec::from_raw_parts(buffer.as_mut_ptr() as *mut U, len, len)
    }
}

pub fn alignment(n: isize, align: isize) -> isize {
    ((n) + (align) - 1) & !((align) - 1)
}

pub fn next_highest_power_of_two(v: isize) -> isize {
    let mut v = v;
    v = v - 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v = v + 1;
    v
}

pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    Ok(absolute_path)
}

pub fn search_file(filename: PathBuf, dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::<PathBuf>::new();
    if filename.is_absolute() && filename.exists() {
        paths.push(filename);
    } else {
        for dir in dirs {
            let absolute_path = dir.join(filename.clone());
            if absolute_path.exists() {
                paths.push(absolute_path);
            }
        }
    }
    paths
}

pub fn change_working_directory() -> Option<String> {
    if let (Ok(current_dir), Ok(current_exe)) = (std::env::current_dir(), std::env::current_exe()) {
        let current_exe_dir = std::path::Path::new(&current_exe)
            .parent()
            .unwrap()
            .to_str()
            .unwrap();
        let current_dir = current_dir.to_str().unwrap();
        if current_dir != current_exe_dir {
            std::env::set_current_dir(current_exe_dir).unwrap();
            return Some(current_exe_dir.to_string());
        }
    }
    None
}

pub fn get_vec_from_raw_mut<'a, T>(
    raw_source: *mut *mut T,
    num_raw_items: std::ffi::c_uint,
) -> Vec<&'a mut T> {
    let mut result = vec![];
    let slice = std::ptr::slice_from_raw_parts(raw_source, num_raw_items as usize);
    if !slice.is_null() {
        unsafe {
            match slice.as_ref() {
                Some(raw) => {
                    for itme in raw {
                        if let Some(item) = itme.as_mut() {
                            result.push(item);
                        }
                    }
                }
                None => {}
            }
        }
    }
    result
}

pub fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(";") {
            let p_str = format!("{}/{}", p, program);
            if std::fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn size_padding_of(current_size: usize, align: usize) -> usize {
    (alignment(current_size as isize, align as isize) as usize) - current_size
}

#[cfg(test)]
pub mod test {
    use crate::{alignment, math_remap_value_range, next_highest_power_of_two};

    #[test]
    pub fn next_highest_power_of_two_test() {
        assert_eq!(next_highest_power_of_two(418), 512);
    }

    #[test]
    pub fn alignment_test() {
        assert_eq!(alignment(418, 4), 420);
    }

    #[test]
    pub fn math_remap_value_range_test() {
        let mapped_value = math_remap_value_range(
            1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 50.0_f64);

        let mapped_value = math_remap_value_range(
            0.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 0.0_f64);

        let mapped_value = math_remap_value_range(
            2.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, 100.0_f64);

        let mapped_value = math_remap_value_range(
            -1.0,
            std::ops::Range::<f64> {
                start: 0.0,
                end: 2.0,
            },
            std::ops::Range::<f64> {
                start: 0.0,
                end: 100.0,
            },
        );
        assert_eq!(mapped_value, -50.0_f64);
    }
}
