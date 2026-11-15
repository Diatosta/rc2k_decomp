//! A diagnostic for the GPU's own time a frame (`RC2K_GPU_TIME=n`): every n-th frame, the GL
//! thread waits for the GPU to finish (`glFinish`) before it shows the frame, and the time of
//! the wait is the work the GPU still had left when the CPU had issued all of it. The frame's
//! issue time is counted from its first part to that wait. A sampled frame loses the overlap
//! of CPU and GPU, so a sample is a lower bound of the GPU's time only when the wait is
//! long; with a short wait the GPU kept up with the CPU. Off (nothing read, nothing waited
//! for) unless the variable is set.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};

/// The sampled frames since the last report, the time from their first part's start to the
/// wait, and the wait itself (microseconds); the longest wait.
pub static FRAMES: AtomicU64 = AtomicU64::new(0);
pub static ISSUE_MICROS: AtomicU64 = AtomicU64::new(0);
pub static WAIT_MICROS: AtomicU64 = AtomicU64::new(0);
pub static LONGEST_WAIT_MICROS: AtomicU64 = AtomicU64::new(0);

/// Which frames are sampled, and when the current frame began.
pub struct Sampler {
    every: u32,
    frames: u32,
    began: Option<Instant>,
}

impl Sampler {
    /// A sampler of every `every`-th frame; none for 0.
    pub fn every(every: u32) -> Option<Sampler> {
        (every > 0).then_some(Sampler {
            every,
            frames: 0,
            began: None,
        })
    }

    /// The sampler `RC2K_GPU_TIME` asks for, if it does.
    pub fn from_env() -> Option<Sampler> {
        let every = std::env::var("RC2K_GPU_TIME").ok()?.trim().parse().ok()?;
        Sampler::every(every)
    }

    /// A part of the current frame is about to be drawn.
    pub fn part_begins(&mut self) {
        self.began.get_or_insert_with(Instant::now);
    }

    /// The frame is about to be shown: whether to wait for the GPU first. The next frame
    /// begins afterwards.
    pub fn frame_ends(&mut self) -> bool {
        self.frames += 1;
        self.frames.is_multiple_of(self.every)
    }

    /// The frame's wait measured: counted, and the frame closed.
    pub fn counted(&mut self, wait: Duration) {
        let issue = self
            .began
            .take()
            .map_or(Duration::ZERO, |b| b.elapsed().saturating_sub(wait));
        FRAMES.fetch_add(1, Relaxed);
        ISSUE_MICROS.fetch_add(issue.as_micros() as u64, Relaxed);
        WAIT_MICROS.fetch_add(wait.as_micros() as u64, Relaxed);
        LONGEST_WAIT_MICROS.fetch_max(wait.as_micros() as u64, Relaxed);
    }

    /// A frame that was not sampled is over.
    pub fn skipped(&mut self) {
        self.began = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_nth_frame_is_sampled_and_zero_is_off() {
        assert!(Sampler::every(0).is_none());
        let mut s = Sampler::every(3).unwrap();
        let sampled: Vec<bool> = (0..7).map(|_| s.frame_ends()).collect();
        assert_eq!(sampled, [false, false, true, false, false, true, false]);
    }
}
