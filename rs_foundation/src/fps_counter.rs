use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct FpsCounter {
    frame_times: VecDeque<Instant>,
    fps: usize,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            frame_times: VecDeque::with_capacity(256),
            fps: 0,
        }
    }

    pub fn tick(&mut self) -> usize {
        self.fps = Self::estimate_fps_continuously(&mut self.frame_times);
        self.fps
    }

    fn estimate_fps_continuously(tick_times: &mut VecDeque<Instant>) -> usize {
        let now = Instant::now();
        tick_times.push_back(now);
        let one_sec_ago = now - Duration::from_secs(1);
        while tick_times.front().map_or(false, |t| *t < one_sec_ago) {
            tick_times.pop_front();
        }
        tick_times.len()
    }

    pub fn fps(&self) -> usize {
        self.fps
    }
}
