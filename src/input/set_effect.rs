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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::input::devices::{at, put};
use crate::input::effects::{EFFECT_LEN, EFFECTS, HAS_EFFECTS, REQUEST, effects_entry};

/// The Rust version: `race_on` is `dword_518090`.
/// Converted: `rc2k_core::input::effects::wanted` and `request`.
pub fn set_effect(r: &Regs, race_on: u32) -> Result<(), Refused> {
    use rc2k_core::input::effects::{request, wanted};
    if wanted(at(r.esi, HAS_EFFECTS) == 1, race_on == 1, r.eax, r.ebx)? {
        put(
            r.esi,
            EFFECTS + EFFECT_LEN * r.eax + REQUEST,
            request(r.ebx, r.ecx, r.edx),
        );
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, |r| {
        crate::input::effects::game_part(crate::input::effects::SET_EFFECT, r)
    })
}
