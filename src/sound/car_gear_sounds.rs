//! `sound::car_gear_sounds` (0x40E24F): a car's three timed sounds placed behind it: the
//! slots at +0x1C4 (armed by `sound::car_engine_state` when the throttle drops at high
//! revs), +0x1DC (armed by its countdown at low throttle) and +0x1F8 (armed with a gear
//! change; its pitch +0x210). Names by those readings.
//!
//! In:  ebx the car's sound object, esi the request
//! Out: every register preserved (`pusha` / `popa`)
//! Nothing in state 3 (+0x10C). The request's place = +0x2C .. +0x34 less +0x38 ..
//! +0x40; the slots at +0x1C4 and +0x1DC (`sound::play_timed_slot`); then, the
//! request's pitch factor (+0x20) times the object's +0x210 for it and put back after,
//! the slot at +0x1F8. One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request; what the slots refuse puts back
//! what was written (sounds already sent stay sent).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};

// ---- Rust version ----

/// The gear sounds of object `o`, through `request`.
pub fn car_gear_sounds(o: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let so = unsafe { crate::sound::store::SoundObject::at(o) };
    if so.state.get() == 3 {
        return Ok(());
    }
    // Converted: `rc2k_core::sound::engine::{place_behind, gear_pitch}` (exact at every
    // precision: one rounding each).
    use rc2k_core::sound::engine::{gear_pitch, place_behind};
    let float = |at: u32| unsafe { (at as *const f32).read_unaligned() };
    let place = place_behind(
        so.place.each_ref().map(|f| f.get()),
        so.motion.each_ref().map(|f| f.get()),
    );
    for (n, v) in place.into_iter().enumerate() {
        unsafe { ((request + 0xC + 4 * n as u32) as *mut f32).write_unaligned(v) };
    }
    for slot in &so.slots[5..] {
        crate::sound::play_timed_slot::play_timed_slot(slot.addr(), request)?;
    }
    let factor = (request + 0x20) as *mut u32;
    let kept = unsafe { factor.read_unaligned() };
    let pitch = gear_pitch(float(request + 0x20), so.gear_pitch.get());
    unsafe { (factor as *mut f32).write_unaligned(pitch) };
    crate::sound::play_timed_slot::play_timed_slot(so.gear_slot.addr(), request)?;
    unsafe { factor.write_unaligned(kept) };
    Ok(())
}
