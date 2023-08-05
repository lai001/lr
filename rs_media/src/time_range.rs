pub struct TimeRangeRational {
    pub start: ffmpeg_next::Rational,
    pub end: ffmpeg_next::Rational,
}

#[derive(Debug)]
pub struct TimeRange {
    pub start: f32,
    pub end: f32,
}
