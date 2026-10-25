//! The replay check (owner, 2026-10-24; not the original's): a replay is the cars'
//! starting state and each tick's input words, played back through the physics, so a
//! race recorded and the same race played back must go through the same states. Every
//! 1000 ticks of car 0's clock (and at its first tick) `rc2k.log` gets one line: the mode
//! (recording or playing), the clock, and for each car a hash of the state the replay
//! keeps (the first `CAR_DWORDS` dwords of its block), its speed and its gear. Two runs
//! compare by their lines: the first that differs is where the physics went apart.
//!
//! A replay also keeps the cars' whole state at each of its frames (100 steps): playing
//! loads it over the cars at every whole frame (`drive::play_replay_frame`), so a replay
//! cannot drift. `compare_frame`, just before that load, compares the state our physics
//! reached with the recorded one, when car 0's clock in both is the same (the steps were
//! ours, no seek in between): any replay, the original game's too, checks the physics.
//! The car fields the viewer sets itself (`VIEWERS`) are left out. Logged: the first
//! differing dwords (at most `DETAILS` lines a run) and every
//! `SUMMARY_EVERY` frames compared a count.

use crate::physics::flow::{a, g};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::numbers_store::NUMBERS;
use rc2k_core::drive::replay_frames::{CAR_DWORDS, PLAYING, RECORDING};
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

const BLOCKS: usize = 0x70F3EC; // g_carBlockStore: four blocks of 0xFE0
const BLOCK: usize = 0xFE0;
const CARS: usize = 0x63DB90; // the cars' count
const EVERY: u32 = 1000;
const DETAILS: u32 = 40;
const SUMMARY_EVERY: u32 = 20;
/// A car's fields the viewer sets itself while playing, left out: +0x9C the last pace
/// checkpoint called (`race::replay_viewer` sets `g_holdCarRevs`, which keeps
/// `physics::pace_notes_step` off; the recording's are the live race's) and +0x220 the
/// option bits (`drive::replay_cameras` writes the viewer's).
const VIEWERS: [usize; 2] = [0x9C, 0x220];

static COMPARED: AtomicU32 = AtomicU32::new(0);
static EQUAL: AtomicU32 = AtomicU32::new(0);
static SKIPPED: AtomicU32 = AtomicU32::new(0);
static DETAILED: AtomicU32 = AtomicU32::new(0);

/// FNV-1a over the dwords' bytes.
fn hash(words: impl Iterator<Item = u32>) -> u32 {
    let mut h = 0x811C_9DC5u32;
    for w in words {
        for b in w.to_le_bytes() {
            h = (h ^ b as u32).wrapping_mul(0x0100_0193);
        }
    }
    h
}

/// After the cars' step of a tick.
pub fn note() {
    let mode = match NUMBERS.replay_mode.get() {
        RECORDING => "recording",
        PLAYING => "playing",
        _ => return,
    };
    let clock = CAR_BLOCKS.cars[0].step_clock.bits();
    if !clock.is_multiple_of(EVERY) && clock != 1 {
        return;
    }
    let mut line = format!("replay check: {mode}, clock {clock}:");
    for car in 0..g(CARS).min(4) as usize {
        let block = BLOCKS + BLOCK * car;
        let at = a(block);
        let h = hash(
            (0..CAR_DWORDS as u32)
                .map(|k| unsafe { ((at + 4 * k) as *const u32).read_unaligned() }),
        );
        let speed = f32::from_bits(g(block + 0x1AC)); // g_carSpeed's place in the block
        let gear = g(block + 0x1D4) as i32; // g_carGear's
        line += &format!(" car {car} {h:08x} speed {speed:.3} gear {gear};");
    }
    crate::framework::log(&line);
}

fn piece_name(piece: StartPiece) -> String {
    match piece {
        StartPiece::Car(k) => format!("car {k}"),
        StartPiece::RaceSlots => "race slots".into(),
        StartPiece::Extra => "0x71BF84".into(),
    }
}

/// At a whole frame of a replay played, before its recorded state is loaded.
pub fn compare_frame() {
    let saved = crate::race::blocks_store::block_addr(8);
    let read = |at: usize| unsafe { (at as *const u32).read_unaligned() };
    let clock = CAR_BLOCKS.cars[0].step_clock.bits();
    if read(saved + 8) != clock {
        SKIPPED.fetch_add(1, Relaxed);
        return;
    }
    let mut differing = 0;
    for (piece, offset, n) in start_layout() {
        let ours = a(crate::drive::save_replay_cars::piece_at(piece)) as usize;
        for k in 0..n {
            let (now, kept) = (read(ours + 4 * k), read(saved + 4 * (offset + k)));
            if now == kept || matches!(piece, StartPiece::Car(_)) && VIEWERS.contains(&(4 * k)) {
                continue;
            }
            differing += 1;
            if DETAILED.fetch_add(1, Relaxed) < DETAILS {
                crate::framework::log(&format!(
                    "replay check: clock {clock}: {} +{:#x}: ours {now:08x} ({}), recorded {kept:08x} ({})",
                    piece_name(piece),
                    4 * k,
                    f32::from_bits(now),
                    f32::from_bits(kept),
                ));
            }
        }
    }
    let compared = COMPARED.fetch_add(1, Relaxed) + 1;
    if differing == 0 {
        EQUAL.fetch_add(1, Relaxed);
    }
    if compared.is_multiple_of(SUMMARY_EVERY) || (differing != 0 && compared < SUMMARY_EVERY) {
        crate::framework::log(&format!(
            "replay check: clock {clock}: {compared} frames compared, {} the same as recorded, {} skipped (not reached by our steps); this one {differing} dwords apart",
            EQUAL.load(Relaxed),
            SKIPPED.load(Relaxed),
        ));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_is_fnv1a() {
        // FNV-1a of the four bytes 00 00 00 00.
        assert_eq!(super::hash([0u32].into_iter()), 0x4B95_F515);
    }
}
