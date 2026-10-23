//! `input::set_effect` (0x40961F): a request for one of a device's six force-feedback
//! effects: with the record's effects on (+0x28 1), and for a "set" (ebx 3) only while the
//! race runs (`dword_518090` 1), `ebx | (ecx & 0xFFF) << 8 | (edx & 0xFFF) << 20` into
//! the effect's +0x2C (effect eax at +0x3C + eax * 0xA4), where the input thread takes it
//! (`input::device_effects`).
//!
//! In:  eax = the effect (0 .. 5), ebx = what to do (2 start, 3 set, 4 stop; not masked),
//!      ecx, edx = two numbers (their low 12 bits), esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. 21 callers (`start_device_effects`, `stop_device_effects`,
//! `update_car_effects`). Refuses (before anything) an effect past 5 (the original writes
//! wherever eax * 0xA4 lands). Shadow mode runs the original alone (the input thread takes
//! the requests). Self-test: a copy whose `dword_518090` is the test's, on records of the
//! test's, against the Rust version: effects 0 .. 5, the four kinds and others, the
//! switches both ways, numbers past 12 bits.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::input::devices::{at, put};
use crate::input::effects::{EFFECT_LEN, EFFECTS, HAS_EFFECTS, RACE_ON_RVA, REQUEST};

/// Effect `effect` (0 .. 5) of the device at `record` told `what` (2 start, 3 set, 4
/// stop) with the numbers `a` and `b`; "set" only while the race runs (`dword_518090`).
/// Converted: `rc2k_core::input::effects::wanted` and `request`.
pub fn set_effect(record: u32, effect: u32, what: u32, a: u32, b: u32) -> Result<(), Refused> {
    use rc2k_core::input::effects::{request, wanted};
    let race_on: u32 = unsafe { crate::framework::game_read(RACE_ON_RVA) };
    if wanted(at(record, HAS_EFFECTS) == 1, race_on == 1, effect, what)? {
        put(
            record,
            EFFECTS + EFFECT_LEN * effect + REQUEST,
            request(what, a, b),
        );
    }
    Ok(())
}
