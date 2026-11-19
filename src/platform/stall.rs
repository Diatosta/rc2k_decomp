//! Diagnostic (to be removed): a watchdog for the game's thread on the phones. The game's
//! thread notes a heartbeat at every event pump and the step it is in; a thread of its own
//! logs when the heartbeat stops for longer than `STALL_MS`, with the step, and when it
//! starts again, with how long it stopped.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

/// A stop worth a line.
const STALL_MS: u64 = 300;

static BEAT: AtomicU64 = AtomicU64::new(0);
static STEP: Mutex<&'static str> = Mutex::new("starting");
static STARTED: AtomicBool = AtomicBool::new(false);

fn now() -> u64 {
    u64::from(crate::platform::os::ticks())
}

/// The game's thread is in `step` now.
pub fn step(step: &'static str) {
    *STEP
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = step;
}

/// The game's thread pumps its events: the heartbeat, and the watchdog started once.
pub fn beat() {
    BEAT.store(now(), Relaxed);
    step("reading events");
    if !STARTED.swap(true, Relaxed) {
        let _ = std::thread::Builder::new()
            .name("stall watchdog".into())
            .spawn(watch);
    }
}

fn watch() {
    let mut stopped_at: Option<(u64, &'static str)> = None;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let beat = BEAT.load(Relaxed);
        let now = now();
        let gap = now.saturating_sub(beat);
        match stopped_at {
            None if gap > STALL_MS => {
                let step = *STEP
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                crate::logfile::program_log::log(&format!(
                    "stall: the game's thread has not read events for {gap} ms; it is in: {step}"
                ));
                stopped_at = Some((beat, step));
            }
            Some((then, step)) if beat != then => {
                crate::logfile::program_log::log(&format!(
                    "stall: the game's thread goes on after {} ms (it was in: {step})",
                    beat.saturating_sub(then)
                ));
                stopped_at = None;
            }
            _ => {}
        }
    }
}
