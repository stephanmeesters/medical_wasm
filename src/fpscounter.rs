extern crate instant;

use instant::Instant;
use std::{collections::VecDeque, time::Duration};

pub struct FPSCounter {
    last_second_frames: VecDeque<Instant>,
}

impl FPSCounter {
    pub fn new() -> Self {
        FPSCounter {
            last_second_frames: VecDeque::with_capacity(128),
        }
    }

    pub fn tick(&mut self) -> usize {
        let now = Instant::now();
        let a_second_ago = now - Duration::from_secs(1);

        while self
            .last_second_frames
            .front()
            .map_or(false, |t| *t < a_second_ago)
        {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push_back(now);
        self.last_second_frames.len()
    }

    pub fn print(&self) -> String {
        let len = self.last_second_frames.len();
        if len >= 2 {
            return format!("{}", self.last_second_frames.len());
        }
        "?".to_owned()
    }
}
