//! `drive::countdown_cue` (0x45D1A5): `drive::countdown_sound` for the current car when it
//! has a sound slot (`dword_641680[g_currentCar]` not -1), ebx the sound.
//!
//! In:  ebx the sound
//! Out: eax the slot; the rest kept; no flags read
//!
//! Caller: `drive::countdown_beep`.
//! Refuses a current car past 7. Shadow mode runs the original alone (a sound command).
//! Self-test: none (a sound command).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::g;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::countdown_cue", rva(0x45D1A5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: eax.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub fn countdown_cue(sound: u32) -> Result<u32, Refused> {
    let (slot, listed) = rc2k_core::physics::listed_cars::listed_entry(g(0x71B3F8), |car| {
        g(0x641680 + 4 * car as usize)
    })?;
    if listed {
        crate::drive::countdown_sound::countdown_sound(slot, sound);
    }
    Ok(slot)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match countdown_cue(r.ebx) {
        Ok(eax) => r.eax = eax,
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let eax = countdown_cue(r.ebx)?;
        r.eax = eax
    }
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("a sound command")
}
