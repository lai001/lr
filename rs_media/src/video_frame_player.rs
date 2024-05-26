use crate::{video_frame_extractor::VideoFrame, video_player_item::VideoPlayerItem};
use rs_foundation::TimeRange;
use std::{
    cmp::Ordering,
    time::{Duration, Instant},
};

pub struct VideoFramePlayer {
    video_player_item: VideoPlayerItem,
    current_play_time: f32,
    seek_time: f32,
    start_play_time: Option<Instant>,
    frames: Vec<VideoFrame>,
}

impl VideoFramePlayer {
    pub fn new(filepath: &str) -> VideoFramePlayer {
        let video_player_item = VideoPlayerItem::new(filepath);
        VideoFramePlayer {
            video_player_item,
            current_play_time: 0.0,
            seek_time: 0.0,
            start_play_time: None,
            frames: vec![],
        }
    }

    pub fn start(&mut self) {
        if self.start_play_time.is_none() {
            self.start_play_time = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        self.start_play_time = None;
    }

    pub fn is_playing(&self) -> bool {
        self.start_play_time.is_none() == false
    }

    pub fn tick(&mut self) {
        let Some(start_play_time) = self.start_play_time else {
            return;
        };

        self.current_play_time = (Instant::now() - start_play_time
            + Duration::from_secs_f32(self.seek_time))
        .as_secs_f32();
        self.frames.retain(|element| {
            let time_range = TimeRange {
                start: self.current_play_time - 0.2,
                end: self.current_play_time + 1.0,
            };
            time_range.is_contains(element.get_time_range_second().start)
                || time_range.is_contains(element.get_time_range_second().end)
        });

        if let (Some(first), Some(last)) = (self.frames.first(), self.frames.last()) {
            let time_range = TimeRange {
                start: first.get_time_range_second().start,
                end: last.get_time_range_second().end,
            };
            if time_range.is_contains(self.current_play_time) {
                return;
            }
        }
        match self.video_player_item.try_recv() {
            Ok(frame) => {
                self.frames.push(frame);
            }
            Err(_) => {}
        }
    }

    pub fn seek(&mut self, new_time: f32) {
        self.frames.clear();
        self.start_play_time = None;
        self.seek_time = new_time;
        self.current_play_time = new_time;
        self.video_player_item.seek(new_time);
    }

    pub fn get_current_play_time(&self) -> f32 {
        self.current_play_time
    }

    pub fn get_current_frame(&mut self) -> Option<&VideoFrame> {
        let closest = self.frames.iter().min_by(|left, right| {
            let d0 = (self.current_play_time - left.get_time_range_second().start).abs()
                + (self.current_play_time - left.get_time_range_second().end).abs();
            let d1 = (self.current_play_time - right.get_time_range_second().start).abs()
                + (self.current_play_time - right.get_time_range_second().end).abs();
            if d0 == d1 {
                Ordering::Equal
            } else if d0 < d1 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
        closest
    }

    pub fn get_duration(&self) -> f32 {
        self.video_player_item.get_duration()
    }
}
