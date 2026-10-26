//! `race::car_file_number` (0x41F4FD): a car's file number.
//!
//! In:  eax = the car
//! Out: eax = its entry of `dword_5C5928` (the car held to 0 .. 0x21); the rest preserved
//!
//! See `race/mud.rs`. Five callers (`race::load_mud` and the car's loading). Manual patch
//! with a private copy (a `jns` in its first bytes). Self-test: cars -5 .. 0x30.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::mud::car_file_number;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::car_file_number", rva(0x41F4FD), thunk, &ORIGINAL)
    .private_copy(0x1C)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, car_file_number(r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
    for car in (-5i32..0x30).chain([i32::MIN, i32::MAX]) {
        check_state(
            &mut t,
            &format!("car {car}"),
            call_original,
            Regs {
                eax: car as u32,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
