use rs_foundation::TimeRange;

pub struct TimeRangeRational {
    pub start: ffmpeg_next::Rational,
    pub end: ffmpeg_next::Rational,
}

impl TimeRangeRational {
    pub fn get_time_range_second(&self) -> TimeRange {
        let start = self.start.numerator() as f32 / self.start.denominator() as f32;
        let end = self.end.numerator() as f32 / self.end.denominator() as f32;
        TimeRange { start, end }
    }

    pub fn is_contains(&self, time: f32) -> bool {
        self.get_time_range_second().is_contains(time)
    }
}
