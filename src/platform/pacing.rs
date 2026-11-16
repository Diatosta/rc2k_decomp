//! The frames' pace: after a frame is shown the game waits until the next one is due, one
//! frame for each refresh of the display (every second or third refresh on a display faster
//! than the simulation's hundred ticks a second). The times frames are due at lie a whole
//! period apart, so a wait that ends a little late shortens the next one and the pace does not
//! drift. Where showing a frame already waits for the display's refresh there is no wait here.
//!
//! Only the frames wait: the simulation's ticks come from the game's clock
//! (`system::tick_count`), a hundred a second whatever the pace.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};

/// The refreshes a second taken for a display that names none (or an absurd number).
const USUAL_REFRESH: f64 = 60.0;

/// The most frames a second: the simulation's ticks. A frame between two ticks would show
/// nothing new.
const MOST_FRAMES: f64 = 100.0;

/// A frame's time on a display refreshing `refresh` times a second: one refresh, or the fewest
/// whole refreshes that keep the frames at `MOST_FRAMES` a second or under.
pub fn frame_period(refresh: Option<f64>) -> Duration {
    let refresh = refresh
        .filter(|rate| (24.0..=1000.0).contains(rate))
        .unwrap_or(USUAL_REFRESH);
    let refreshes = (refresh / MOST_FRAMES).ceil().max(1.0);
    Duration::from_secs_f64(refreshes / refresh)
}

/// When the next frame is due.
pub struct Pacer {
    due: Option<Instant>,
}

impl Pacer {
    pub const fn new() -> Self {
        Pacer { due: None }
    }

    /// A frame was shown and it is `now`: the wait until the next frame is due, a `period`
    /// after this one was. A frame a whole period late or more (a slow frame, a load, the first
    /// frame) starts the pace anew from `now` instead of being caught up with.
    pub fn wait(&mut self, now: Instant, period: Duration) -> Duration {
        let due = match self.due {
            Some(last) if now < last + 2 * period => last + period,
            _ => now,
        };
        self.due = Some(due);
        due.saturating_duration_since(now)
    }

    /// The pace forgotten: the next frame shown starts it anew.
    pub fn forget(&mut self) {
        self.due = None;
    }
}

static PACER: Mutex<Pacer> = Mutex::new(Pacer::new());

/// The frame's time on the window's display in nanoseconds (`frame_period`); 60 a second
/// until a screen is set.
static PERIOD_NANOS: AtomicU64 = AtomicU64::new(16_666_667);

/// Showing a frame waits for the display's refresh itself.
static SHOWING_WAITS: AtomicBool = AtomicBool::new(false);

/// The window's display as the screen was set: its refreshes a second (`None`: not known) and
/// whether showing a frame waits for the refresh.
pub fn set_display(refresh: Option<f64>, showing_waits: bool) {
    /// Whether the log has the pace (it is written again only when it changes).
    static LOGGED: AtomicBool = AtomicBool::new(false);
    let period = frame_period(refresh);
    let before = PERIOD_NANOS.swap(period.as_nanos() as u64, Relaxed);
    let waited = SHOWING_WAITS.swap(showing_waits, Relaxed);
    let changed = before != period.as_nanos() as u64 || waited != showing_waits;
    if !LOGGED.swap(true, Relaxed) || changed {
        crate::logfile::program_log::log(&format!(
            "platform: the display refreshes {} times a second: {}",
            refresh.map_or("an unknown number of".to_string(), |rate| format!(
                "{rate:.2}"
            )),
            if showing_waits {
                "showing a frame waits for it".to_string()
            } else {
                format!(
                    "a frame every {:.2} ms ({:.2} a second)",
                    period.as_secs_f64() * 1000.0,
                    1.0 / period.as_secs_f64()
                )
            }
        ));
    }
}

/// A frame was shown: waits until the next is due.
pub fn frame_shown() {
    let mut pacer = PACER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if SHOWING_WAITS.load(Relaxed) {
        pacer.forget();
        return;
    }
    let period = Duration::from_nanos(PERIOD_NANOS.load(Relaxed));
    // A scripted run at a faster clock shows its frames as much faster.
    #[cfg(feature = "remote")]
    let period = period / crate::remote::speed();
    let wait = pacer.wait(Instant::now(), period);
    drop(pacer);
    if !wait.is_zero() {
        std::thread::sleep(wait);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn millis(period: Duration) -> f64 {
        period.as_secs_f64() * 1000.0
    }

    #[test]
    fn a_frame_for_each_refresh_up_to_the_ticks() {
        for (refresh, frames) in [
            (Some(60.0), 60.0),
            (Some(59.94), 59.94),
            (Some(50.0), 50.0),
            (Some(75.0), 75.0),
            (Some(90.0), 90.0),
            (Some(100.0), 100.0),
            (Some(120.0), 60.0),
            (Some(144.0), 72.0),
            (Some(165.0), 82.5),
            (Some(240.0), 80.0),
            (Some(360.0), 90.0),
            (None, 60.0),
            (Some(0.0), 60.0),
            (Some(1.0), 60.0),
            (Some(f64::NAN), 60.0),
            (Some(1.0e9), 60.0),
        ] {
            let period = frame_period(refresh);
            assert!(
                (millis(period) - 1000.0 / frames).abs() < 1.0e-5,
                "{refresh:?}: {period:?}"
            );
        }
    }

    #[test]
    fn frames_are_due_a_period_apart() {
        let period = Duration::from_micros(16_667);
        let start = Instant::now();
        let at = |micros: u64| start + Duration::from_micros(micros);
        let mut pacer = Pacer::new();
        // The first frame starts the pace.
        assert_eq!(pacer.wait(at(0), period), Duration::ZERO);
        // 5 ms of work: the rest of the period.
        assert_eq!(pacer.wait(at(5_000), period), Duration::from_micros(11_667));
        // That wait ended 300 us late and the frame took 4 ms: the next is still due two
        // periods after the first.
        assert_eq!(
            pacer.wait(at(20_967), period),
            Duration::from_micros(12_367)
        );
        // A frame 3 ms late is not waited for, and the one after is due on the old pace.
        assert_eq!(pacer.wait(at(53_001), period), Duration::ZERO);
        assert_eq!(pacer.wait(at(60_000), period), Duration::from_micros(6_668));
        // A frame a period late or more starts the pace anew.
        assert_eq!(pacer.wait(at(200_000), period), Duration::ZERO);
        assert_eq!(
            pacer.wait(at(201_000), period),
            Duration::from_micros(15_667)
        );
    }

    #[test]
    fn slow_frames_never_wait() {
        let period = Duration::from_micros(16_667);
        let start = Instant::now();
        let mut pacer = Pacer::new();
        // 20 ms a frame: every frame is late.
        for frame in 0..200u64 {
            let now = start + Duration::from_millis(20 * frame);
            assert_eq!(pacer.wait(now, period), Duration::ZERO, "frame {frame}");
        }
    }

    #[test]
    fn a_forgotten_pace_starts_anew() {
        let period = Duration::from_millis(10);
        let start = Instant::now();
        let mut pacer = Pacer::new();
        assert_eq!(pacer.wait(start, period), Duration::ZERO);
        pacer.forget();
        assert_eq!(
            pacer.wait(start + Duration::from_millis(1), period),
            Duration::ZERO
        );
    }
}
