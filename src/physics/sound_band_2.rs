//! `physics::sound_band_2` (0x45D097): `g_car +0x1C4` x `flt_6D1AE0`, less 10, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CD60`.
//! Self-test: `sound_bands::test_twin`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::sound_bands::{SCRATCH, Twin, band_plain, test_twin};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_band_2", rva(0x45D097), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TWIN: Twin = (0x45D097, 0x71A19C, 0x6D1AE0, 10);

/// The converted version (plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub unsafe fn sound_band_2_plain() -> Left {
    unsafe { band_plain(TWIN) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sound_band_2_plain() })
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

fn self_test() -> (usize, usize) {
    test_twin(&HOOK, TWIN, &[0xD9, 0x05, 0x9C, 0xA1, 0x71, 0x00])
}
