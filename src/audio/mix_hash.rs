//! With `RC2K_MIX_HASH` (a check, remote builds only): what the device mixer outputs during a
//! race, hashed per sound turn and logged as "mix hash N: ...".
//!
//! The device mixer normally runs in SDL's audio callback, paced by the device's clock, so the
//! samples it has made at a given command are not the same in two runs. While a race is loaded
//! and the check is on, the callback therefore plays silence and the audio thread mixes instead:
//! once after each audio turn that took a turn's end command, a fixed `TURN_FRAMES` frames. The
//! buffers' positions then advance only by those mixes, so the hash depends on the commands
//! alone.

use crate::debug::check_hash::CheckHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock};

/// Frames mixed per sound turn: 20 ms of the device's 44100 Hz.
const TURN_FRAMES: usize = 882;

/// A race is loaded (set by the command queue, which knows it).
static RACING: AtomicBool = AtomicBool::new(false);

/// The audio thread took a turn's end command and has not mixed for it yet.
static TURN_TAKEN: AtomicBool = AtomicBool::new(false);

/// The turns of this race hashed so far.
static TURNS: Mutex<u32> = Mutex::new(0);

fn asked() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("RC2K_MIX_HASH").is_some())
}

/// Whether the audio thread mixes for the check (and the device callback plays silence).
pub fn deterministic() -> bool {
    asked() && RACING.load(Relaxed)
}

/// Records whether a race is loaded; a race starts a fresh count.
pub fn set_racing(racing: bool) {
    if RACING.swap(racing, Relaxed) != racing {
        TURN_TAKEN.store(false, Relaxed);
        if let Ok(mut turns) = TURNS.lock() {
            *turns = 0;
        }
    }
}

/// The audio thread has taken a turn's end command.
pub fn turn_end_taken() {
    TURN_TAKEN.store(true, Relaxed);
}

/// After the audio thread's turn: if it took a turn's end, mixes `TURN_FRAMES` frames and logs
/// their hash.
pub fn audio_turn_done() {
    if !deterministic() || !TURN_TAKEN.swap(false, Relaxed) {
        return;
    }
    let pcm = crate::platform::sound::mixed_frames(TURN_FRAMES);
    let mut hasher = CheckHasher::default();
    pcm.hash(&mut hasher);
    let Ok(mut turns) = TURNS.lock() else {
        return;
    };
    crate::logfile::program_log::log(&format!("mix hash {turns}: {:016x}", hasher.finish()));
    *turns += 1;
}
