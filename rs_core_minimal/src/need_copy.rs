use bitflags::bitflags;
use std::fs;
use std::io::Read;
use std::path::Path;

bitflags! {
    pub struct CompareMode: u32 {
        const SIZE = 1;
        const CONTENT = 1 << 1;
        const MTIME = 1 << 2;
    }
}

pub fn is_need_copy<P: AsRef<Path>>(src: P, dst: P, mode: CompareMode) -> Option<bool> {
    let dst_meta = match fs::metadata(&dst) {
        Ok(m) => m,
        Err(_) => return Some(true),
    };

    let src_meta = fs::metadata(&src).ok()?;

    if mode.contains(CompareMode::SIZE) {
        if src_meta.len() != dst_meta.len() {
            return Some(true);
        }
    }

    if mode.contains(CompareMode::MTIME) {
        let src_time = src_meta.modified().ok()?;
        let dst_time = dst_meta.modified().ok()?;
        if src_time > dst_time {
            return Some(true);
        }
    }

    if mode.contains(CompareMode::CONTENT) {
        let mut fa = fs::File::open(&src).ok()?;
        let mut fb = fs::File::open(&dst).ok()?;

        let mut ba = Vec::new();
        let mut bb = Vec::new();

        fa.read_to_end(&mut ba).ok()?;
        fb.read_to_end(&mut bb).ok()?;

        if ba != bb {
            return Some(true);
        }
    }

    return Some(false);
}
