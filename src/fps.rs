#[allow(clippy::upper_case_acronyms)]
pub struct FPS {
    previous_time: std::time::Instant,
    frames: usize,
    fps: f64,
}

impl FPS {
    pub fn new() -> Self {
        Self {
            previous_time: std::time::Instant::now(),
            frames: 0,
            fps: f64::NAN,
        }
    }

    /// Update and return current fps
    pub fn update_and_get(&mut self) -> f64 {
        self.frames += 1;

        let current = std::time::Instant::now();
        let time_diff = (current - self.previous_time).as_secs_f64();

        self.fps = self.frames as f64 / time_diff;

        if time_diff > 0.2 {
            self.previous_time = current;
            self.frames = 0;
        }

        self.fps
    }

    /// Get the cached fps
    pub fn get_last_processed(&self) -> f64 {
        self.fps
    }
}

impl Default for FPS {
    fn default() -> Self {
        Self::new()
    }
}
