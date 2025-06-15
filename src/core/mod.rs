mod texture;

pub use texture::{Texture, TextureFormat};

use std::time::Instant;

pub struct Timer {
    #[allow(dead_code)]
    start: Instant,
    last_update: Instant,
}

impl Timer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_update: now,
        }
    }

    pub fn delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;
        delta
    }

    #[allow(dead_code)]
    pub fn elapsed(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        println!("[LOG] {}", format!($($arg)*));
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*));
    };
}
