//! `race::whole_to_float` (0x40D706): the dword at ebx made a float (`fild` / `fstp`) when
//! it looks like a whole number (top byte 0 or 0xFF).
//!
//! In:  ebx = the dword
//! Out: eax = the dword's top byte (`and 0xFF000000`); the rest preserved
//!
//! See `race/sound_tables.rs`. Callers: `race::sound_span`, `race::sound_row` (no flags
//! read). Shadow mode: the Rust version, the dword put back, the original's, compared.
//! Self-test: dwords round the test (0, small, negative, floats, the byte's edges).

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::sound_tables::whole_to_float;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::whole_to_float", rva(0x40D706), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { whole_to_float(r.ebx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.ebx, 4)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let cell = Cell::new(0u32);
    let at = cell.as_ptr() as u32;
    for v in [
        0u32,
        5,
        0x00FF_FFFF,
        0xFFFF_FFFB,
        0xFF00_0000,
        0x3F80_0000,
        0x8000_0000,
        0x7FFF_FFFF,
        0x0100_0000,
        0xFEFF_FFFF,
    ] {
        check_state(
            &mut t,
            &format!("{v:#x}"),
            call_original,
            Regs {
                ebx: at,
                ..Regs::SENTINEL
            },
            &|| cell.set(v),
            &[(at, 4)],
            &rust,
        );
    }
    t.finish()
}
