//! A diagnostic (2026-10-20, no hook): the owner hears, now and then, a sound stuck over
//! everything that even a pause does not stop, with the audio thread still turning. A
//! sound only stops through the playing and stopping lists (`audio::update_voices`,
//! `audio::stop_all_voices`), so a buffer that plays while its voice is on neither list
//! can never be stopped. Once a second, after the turn's update (the voices' own buffers'
//! way only: not A3D, not the game's mixer), every voice of the pool whose buffer plays is
//! looked for on the two lists; one on neither is logged once ("audio: stray voice"), with
//! what it holds, until it is quiet again. Called by the Rust thread's turn and by
//! `audio::update_voices`' dispatch (the original thread's, with shadow mode on).

use crate::audio::voices::{
    A3D_RVA, PLAYING_RVA, SOUND_MODE_RVA, STOPPING_RVA, VOICE_COUNT_RVA, VOICES, VOICES_RVA, Voice,
    read, voice,
};
use crate::framework::log;
use crate::store::addr32 as a;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// The turns between two checks (a turn is 20 ms).
const EVERY: u32 = 50;

static TURNS: AtomicU32 = AtomicU32::new(0);
/// The strays already logged.
static LOGGED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// A list's voices, from the head at `head` (at most the pool's count).
unsafe fn list(head: u32) -> HashSet<u32> {
    let mut out = HashSet::new();
    unsafe {
        let mut at = read(head);
        while at != 0 && out.len() <= VOICES as usize && out.insert(at) {
            at = voice(at).next;
        }
    }
    out
}

/// Called after each turn's update.
pub fn check() {
    if !TURNS.fetch_add(1, Ordering::Relaxed).is_multiple_of(EVERY) {
        return;
    }
    unsafe {
        if read(a(A3D_RVA)) == 1 || read(a(SOUND_MODE_RVA)) & 0x1_0000 == 0 {
            return;
        }
        let (playing, stopping) = (list(a(PLAYING_RVA)), list(a(STOPPING_RVA)));
        let listed: HashSet<u32> = playing.union(&stopping).copied().collect();
        let count = read(a(VOICE_COUNT_RVA)).min(VOICES);
        let size = std::mem::size_of::<Voice>() as u32;
        let mut strays = Vec::new();
        for n in 0..count {
            let at = a(VOICES_RVA) + n * size;
            let v = voice(at);
            if v.flags & 1 == 0 || v.buffer == 0 || listed.contains(&at) {
                continue;
            }
            if crate::audio::buffer_play::is_playing(v).is_ok_and(|s| s & 1 != 0) {
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
}
