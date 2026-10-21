//! `physics::sound_band_1` (0x45D067): `flt_6D3108` x `flt_6D1AE0`, less 1, held to 0 ..= 0x40, in ebx.
//! See `physics/sound_bands.rs` (the core and the test).
//!
//! In:  nothing
//! Out: ebx the band; the rest kept
//!
//! Caller: `sub_45CEF9`.
//! Self-test: `sound_bands::test_twin`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, game_addr, hook_asm, rva};
use crate::physics::sound_bands::{SCRATCH, Twin, band_plain, test_twin};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_band_1", rva(0x45D067), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TWIN: Twin = (0x45D067, 0x6D3108, 0x6D1AE0, 1);

/// The converted version (plain floats).
// Converted: `rc2k_core::physics::sound_bands::sound_band`.
pub unsafe fn sound_band_1_plain() -> Left {
    unsafe { band_plain(TWIN) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sound_band_1_plain() })
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
    test_twin(&HOOK, TWIN, &[0xD9, 0x05, 0x08, 0x31, 0x6D, 0x00])
}
