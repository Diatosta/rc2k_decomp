//! A diagnostic (2026-10-20): the owner hears, now and then, a sound stuck over
//! everything that even a pause does not stop, with the audio thread still turning. A
//! sound only stops through the playing and stopping lists (`audio::update_voices`,
//! `audio::stop_all_voices`), so a buffer that plays while its voice is on neither list
//! can never be stopped. Once a second, after the turn's update (the voices' own buffers'
//! way only: not A3D, not the game's mixer), every voice of the pool whose buffer plays is
//! looked for on the two lists; one on neither is logged once ("audio: stray voice"), with
//! what it holds, until it is quiet again.

use crate::audio::voices::{VOICES, voice};
use crate::framework::log;
use crate::session::Dword;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// The turns between two checks (a turn is 20 ms).
const EVERY: u32 = 50;

static TURNS: AtomicU32 = AtomicU32::new(0);
/// The strays already logged.
static LOGGED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// A list's voices, from the head `head` holds (at most the pool's count).
fn list(head: &Dword) -> HashSet<u32> {
    let mut out = HashSet::new();
    let mut at = head.get();
    while at != 0 && out.len() <= VOICES as usize && out.insert(at) {
        at = voice(at).next;
    }
    out
}

/// The voices on the playing and stopping lists (the debug UI's, on the audio thread).
pub fn on_lists() -> (u32, u32) {
    (
        list(&crate::audio::owner::audio().state.playing_voices).len() as u32,
        list(&crate::audio::owner::audio().state.stopping_voices).len() as u32,
    )
}

/// Called after each turn's update.
pub fn check() {
    if !TURNS.fetch_add(1, Ordering::Relaxed).is_multiple_of(EVERY) {
        return;
    }
    if crate::audio::owner::audio().state.use_a3d.get() == 1
        || crate::audio::owner::audio().state.sound_mode.get() & 0x1_0000 == 0
    {
        return;
    }
    let (playing, stopping) = (
        list(&crate::audio::owner::audio().state.playing_voices),
        list(&crate::audio::owner::audio().state.stopping_voices),
    );
    let listed: HashSet<u32> = playing.union(&stopping).copied().collect();
    let count = crate::audio::owner::audio()
        .state
        .voice_count
        .get()
        .min(VOICES);
    let mut strays = Vec::new();
    for n in 0..count {
        let at = crate::audio::voices::voice_id(n);
        let v = voice(at);
        if v.flags & 1 == 0 || v.buffer == 0 || listed.contains(&at) {
            continue;
        }
        if crate::audio::buffer_play::is_playing(&v).is_ok_and(|s| s & 1 != 0) {
            strays.push(n);
            let mut logged = LOGGED.lock().unwrap();
            if !logged.contains(&n) {
                logged.push(n);
                log(&format!(
                    "audio: stray voice {n} (sound {:#x}, kind {}, state {}, channel {:#x}, next {:#x}, volume {:#x}, priority {}) plays on neither list; the playing list holds {}, the stopping list {}",
                    v.sound,
                    v.kind,
                    v.state,
                    v.channel,
                    v.next,
                    v.volume,
                    v.priority,
                    playing.len(),
                    stopping.len()
                ));
            }
        }
    }
    LOGGED.lock().unwrap().retain(|n| strays.contains(n));
}
