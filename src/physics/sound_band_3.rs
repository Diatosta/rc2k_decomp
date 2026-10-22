//! `physics::sound_band_3` (0x45D0F1): `flt_6D3CD0` x `flt_6D1AE8`, less 10, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CE32`.
//! Self-test: `sound_bands::test_twin`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::sound_bands::{SCRATCH, Twin, band_plain, test_twin};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_band_3", rva(0x45D0F1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TWIN: Twin = (0x45D0F1, 0x6D3CD0, 0x6D1AE8, 10);

/// The converted version (plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub unsafe fn sound_band_3_plain() -> Left {
    unsafe { band_plain(TWIN) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sound_band_3_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(
        &HOOK,
        r,
        call_original,
        &[(game_addr(rva(SCRATCH)) as u32, 4)],
        &rust,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_twin(&HOOK, TWIN, &[0xD9, 0x05, 0xD0, 0x3C, 0x6D, 0x00])
}
