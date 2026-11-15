//! The replay check: a replay is the cars' starting state and each tick's input words, played
//! back through the physics, so a race recorded and the same race played back must go through
//! the same states. Every 1000 ticks of car 0's clock `rc2k.log` gets one line (the mode, the
//! clock, and for each car a hash of the replay-kept state, its speed and gear); the first line
//! that differs between two runs is where the physics went apart. A replay also keeps the cars'
//! whole state at each frame (100 steps): `compare_frame`, just before it is loaded over the
//! cars, compares it with the state our physics reached, so any replay, one recorded by RAL.EXE
//! too, checks the physics. The car fields the viewer sets itself (`VIEWERS`) are left out. Logged:
//! the first differing dwords (at most `DETAILS` lines a run) and a count every `SUMMARY_EVERY`
//! frames.

use crate::drive::replay_state::{
    FRAME_HEAD_AT, GEAR_CHANGE_PENDING_AT, LAST_PACE_CALL_AT, OPTION_BITS_AT,
};
use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{PLAYING, RECORDING};
use rc2k_core::drive::replay_frames::{StartPiece, start_layout};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

const EVERY: u32 = 1000;
const DETAILS: u32 = 40;
const SUMMARY_EVERY: u32 = 20;
/// A car's fields the viewer or the sound sets, not the physics, left out (file offsets): +0x0
/// and +0x4 (every car's; the first car's are the frame's head: the camera bytes
/// `drive::replay_cameras` reads; a camera edited in the viewer rewrites them), +0x9C the last
/// pace checkpoint called (`race::replay_viewer` holds the revs, which keeps
/// `physics::pace_notes_step` off; the recording's are the live race's), +0x220 the option
/// bits (`drive::replay_cameras` writes the viewer's) and +0x23C the gear-change flag the
/// sound listener clears (`drive::set_listener`, run by the frame, not the step: seen
/// recorded 1, played 0).
const VIEWERS: [usize; 5] = [
    FRAME_HEAD_AT,
    FRAME_HEAD_AT + 4,
    LAST_PACE_CALL_AT,
    OPTION_BITS_AT,
    GEAR_CHANGE_PENDING_AT,
];

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
pub fn note(race: &Race) -> Result<(), BadInput> {
    let mode = match race.course.numbers.get().replay_mode.get() {
        RECORDING => "recording",
        PLAYING => "playing",
        _ => return Ok(()),
    };
    let clock = race.car_blocks().cars[0].step_clock.get();
    #[cfg(feature = "remote")]
    if mode == "playing" {
        crate::remote::replay_tick(clock);
    }
    if !clock.is_multiple_of(EVERY) && clock != 1 {
        return Ok(());
    }
    let mut line = format!("replay check: {mode}, clock {clock}:");
    // The cars' state as the replay file keeps it, field by field at the file's offsets, so
    // the hash does not depend on how the live record lies in memory.
    let kept = crate::drive::replay_state::gather(&race.car_blocks().cars, race.shared.work.get());
    let cars = race.shared.view.get().race_cars.get().min(4) as usize;
    for (car, (block, kept_car)) in race
        .car_blocks()
        .cars
        .iter()
        .zip(&kept.cars)
        .take(cars)
        .enumerate()
    {
        let h = hash(kept_car.iter().copied());
        let speed = block.local_velocity[2].get(); // the car's speed
        let gear = block.gear.get();
        line += &format!(" car {car} {h:08x} speed {speed:.3} gear {gear};");
    }
    crate::logfile::program_log::log(&line);
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
pub fn compare_frame(race: &Race) -> Result<(), BadInput> {
    let staging = race.replay.staging();
    let clock = race.car_blocks().cars[0].step_clock.get();
    if staging.start_word(2)? != clock {
        SKIPPED.fetch_add(1, Relaxed);
        return Ok(());
    }
    let live = crate::drive::replay_state::gather(&race.car_blocks().cars, race.shared.work.get());
    let mut differing = 0;
    for (piece, offset, n) in start_layout() {
        let ours: &[u32] = match piece {
            StartPiece::Car(k) => &live.cars[k as usize],
            StartPiece::RaceSlots => &live.race_slots,
            StartPiece::Extra => &live.extra,
        };
        for (k, &now) in ours.iter().enumerate().take(n) {
            let kept = staging.start_word(offset + k)?;
            if now == kept || matches!(piece, StartPiece::Car(_)) && VIEWERS.contains(&(4 * k)) {
                continue;
            }
            differing += 1;
            if DETAILED.fetch_add(1, Relaxed) < DETAILS {
                crate::logfile::program_log::log(&format!(
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
        crate::logfile::program_log::log(&format!(
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
