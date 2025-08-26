pub enum EOptions {
    NoLimit,
    FPS(f32),
}

pub struct FrameSync {
    current: std::time::Instant,
    last_elapsed: Option<std::time::Duration>,
    options: EOptions,
}

impl FrameSync {
    pub fn new(options: EOptions) -> Self {
        Self {
            current: std::time::Instant::now(),
            last_elapsed: None,
            options,
        }
    }

    pub fn tick(&mut self) -> Option<std::time::Duration> {
        let now = std::time::Instant::now();
        let current = self.current;
        self.current = now;
        let elapsed = now - current;
        match self.options {
            EOptions::NoLimit => {
                self.last_elapsed = Some(elapsed);
                return None;
            }
            EOptions::FPS(fps) => {
                let interval = std::time::Duration::from_secs_f32(1.0 / fps);
                let wait: std::time::Duration;
                if interval <= elapsed {
                    wait = std::time::Duration::from_millis(0);
                } else {
                    wait = interval - elapsed;
                }
                self.last_elapsed = Some(elapsed);
                return Some(wait);
            }
        }
    }

    pub fn get_last_elapsed(&self) -> Option<std::time::Duration> {
        self.last_elapsed
    }

    pub fn sync(&mut self) {
        if let Some(wait) = self.tick() {
            std::thread::sleep(wait);
        }
    }
}
