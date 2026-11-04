//! `sound::mark_car_voices` (0x40D75A): each car's sound object told whether the car
//! sounds and whose voice it has.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Only while `dword_514F8C` is 1: for each of the `dword_514F7C` objects in use
//! (`g_raceSoundObjects[n]`), +8 = 0; when its car (`g_carBlocks[n]`) has +0x25C not 0,
//! +8 = 1 and +0 = 0, and unless +0x25C is 1, +0 = !n (a negative number: a car of the
//! computer's, by the callers' reading). One caller, `sound::race_turn`.
//! Refuses (before writing) a count of 0 (the original's loop would not end) or over 8,
//! and a null object or car.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::race::work_store::RACE_WORK;
use crate::sound::store::{RACE_SOUNDS, SoundObject};

// ---- Rust version ----

/// The (object, car) pairs in use, or a refusal.
fn pairs() -> Result<Vec<(&'static SoundObject, u32)>, BadInput> {
    let count = RACE_SOUNDS.count.get();
    if count == 0 || count > 8 {
        return Err(BadInput("a sound object count of 0 or over 8"));
    }
    let pairs = (0..count as usize)
        .map(|n| Ok((SoundObject::number(n as u32)?, RACE_WORK.car_block(n))))
        .collect::<Result<Vec<_>, BadInput>>()?;
    if pairs.iter().any(|&(_, c)| c == 0) {
        return Err(BadInput("a null car"));
    }
    Ok(pairs)
}

/// Converted: `rc2k_core::sound::cars::voice_mark`.
pub fn mark_car_voices() -> Result<(), Refused> {
    if RACE_SOUNDS.mark_car_voices.get() != 1 {
        return Ok(());
    }
    for (n, (o, car)) in pairs()?.into_iter().enumerate() {
        let kind = crate::race::trap::car_at(car)?.driver_kind.get();
        let (sounds, voice) = rc2k_core::sound::cars::voice_mark(n as u32, kind);
        o.in_use.set(sounds);
        if let Some(voice) = voice {
            o.sound_offset.set(voice);
        }
    }
    Ok(())
}
