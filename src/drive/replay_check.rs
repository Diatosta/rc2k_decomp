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

use crate::errors::BadInput;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::race;
use crate::race::view_store::RACE_VIEW;
use rc2k_core::drive::replay_frames::{CAR_DWORDS, PLAYING, RECORDING};
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

const EVERY: u32 = 1000;
const DETAILS: u32 = 40;
const SUMMARY_EVERY: u32 = 20;
/// A car's fields the viewer or the sound sets, not the physics, left out: +0x0 and +0x4
/// (every car's; the first car's are the frame's head (the camera bytes `drive::replay_cameras`
/// reads; a camera edited in the viewer rewrites them: seen 2026-10-29)), +0x9C the last
/// pace checkpoint called (`race::replay_viewer` sets `g_holdCarRevs`, which keeps
/// `physics::pace_notes_step` off; the recording's are the live race's), +0x220 the option
/// bits (`drive::replay_cameras` writes the viewer's) and +0x23C the gear-change flag the
/// sound listener clears (`drive::set_listener`, run by the frame, not the step: seen
/// recorded 1, played 0, 2026-10-29).
const VIEWERS: [usize; 5] = [0x0, 0x4, 0x9C, 0x220, 0x23C];

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
pub fn note() -> Result<(), BadInput> {
    let mode = match race().course.numbers.replay_mode.get() {
        RECORDING => "recording",
        PLAYING => "playing",
        _ => return Ok(()),
    };
    let clock = CAR_BLOCKS.cars[0].step_clock.bits();
    #[cfg(feature = "remote")]
    if mode == "playing" {
        crate::remote::replay_tick(clock);
    }
    if !clock.is_multiple_of(EVERY) && clock != 1 {
        return Ok(());
    }
    let mut line = format!("replay check: {mode}, clock {clock}:");
    for car in 0..RACE_VIEW.race_cars.get().min(4) as usize {
        // g_carBlockStore: four blocks of 0xFE0.
        let block = &CAR_BLOCKS.cars[car];
        let bytes =
            crate::pointers::GamePtr::to(block).read_bytes(4 * CAR_DWORDS, "a car's block")?;
        let h = hash(
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|w| u32::from_le_bytes(*w)),
        );
        let speed = block.local_velocity[2].get(); // g_carSpeed's place in the block
        let gear = block.gear.get() as i32; // g_carGear's
        line += &format!(" car {car} {h:08x} speed {speed:.3} gear {gear};");
    }
    crate::framework::log(&line);
    Ok(())
}

fn piece_name(piece: StartPiece) -> String {
    match piece {
        StartPiece::Car(k) => format!("car {k}"),
        StartPiece::RaceSlots => "race slots".into(),
        StartPiece::Extra => "0x71BF84".into(),
    }
}

/// At a whole frame of a replay played, before its recorded state is loaded.
pub fn compare_frame() -> Result<(), BadInput> {
    let saved = crate::race::blocks_store::replay_state();
    let clock = CAR_BLOCKS.cars[0].step_clock.bits();
    if saved.offset(8).read::<u32>("the replay check's blocks")? != clock {
        SKIPPED.fetch_add(1, Relaxed);
        return Ok(());
    }
    let mut differing = 0;
    for (piece, offset, n) in start_layout() {
        let ours = crate::drive::save_replay_cars::piece_place(piece);
        for k in 0..n as u32 {
            let (now, kept) = (
                ours.offset(4 * k)
                    .read::<u32>("the replay check's blocks")?,
                saved
                    .offset(4 * (offset as u32 + k))
                    .read::<u32>("the replay check's blocks")?,
            );
            if now == kept
                || matches!(piece, StartPiece::Car(_)) && VIEWERS.contains(&(4 * k as usize))
            {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_is_fnv1a() {
        // FNV-1a of the four bytes 00 00 00 00.
        assert_eq!(super::hash([0u32].into_iter()), 0x4B95_F515);
    }
}
