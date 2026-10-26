//! `input::start_effects` (0x409659): both device records' effects asked to start
//! (`input::start_device_effects` on `g_inputRecord1`, then `g_inputRecord2`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::begin_effects`. Its calls through `part`.
//! Shadow mode runs the original alone. Self-test: a copy whose calls go to the noting
//! stand-ins, against the Rust version (`stop_effects` shares it).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, rva};
use crate::input::effects::{Part, RECORD_1, RECORD_2, START_DEVICE, compare, effects_entry};
use crate::system::api_stand_ins::copy_with;

/// Both records through `place` (the two functions' body).
pub fn both_records(r: &mut Regs, place: u32, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    for record in [RECORD_1, RECORD_2] {
        g.esi = game_addr(rva(record)) as u32;
        part(place, &mut g)?;
    }
    Ok(())
}

/// The Rust version.
pub fn start_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    both_records(r, START_DEVICE, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    effects_entry(r, start_effects)
}

/// The test of both: a copy of the original at `address` (0x19 bytes, its calls at +7 and
/// +0x12 to `place`).
pub fn test_both(
    hook: &RawHook,
    address: usize,
    place: u32,
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    // pusha; lea esi, g_inputRecord1
    let entry = [0x60, 0x8D, 0x35, 0xF0, 0x6E, 0x4F, 0x00];
    let copy = match copy_with(address, 0x19, &entry, &[7, 0x12], &[place], &[], &[], 0) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    compare(
        &mut t,
        "both records",
        &copy,
        Regs::SENTINEL,
        &|| {},
        &[],
        &|r, part| rust(r, part),
        false,
    );
    t.finish()
}
