//! `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
//! +0x720/+0x724/+0x728/+0x730 0, the five dwords +0xA0C .. +0xA1C -1; then four dwords
//! from 0x6D1838 (`dword_6D1838`, `a64`, `dword_6D1844`, `a32`) spaces and four from
//! 0x6D1850 (`dword_6D1850`, `a64_0`, `dword_6D185C`, `a32_0`) -1.
//!
//! In:  nothing
//! Out: ebx = 8, edi = the last car block; the rest preserved
//!
//! See `race/race_init.rs`. One caller, `sub_4462B9` (race init). Shadow mode: the Rust
//! version, the blocks and texts put back, the original's, compared. Self-test: the game's
//! blocks and texts filled with values of the test's, both versions, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EBX, EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_init::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::reset_car_race_state",
    rva(0x444C1D),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { reset_car_race_state(r) };
    Ok(Left::nothing().with(EBX, g.ebx).with(EDI, g.edi))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &race_state_regions(), &rust);
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
    let regions = race_state_regions();
    let saved = capture(&regions);
    for fill in [0u8, 0x5A, 0xFF] {
        check_state(
            &mut t,
            &format!("filled {fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, fill, len);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
