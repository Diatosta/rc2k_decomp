//! `race::car_kind_11` (0x41F485): a car's kind for its slot +11.
//!
//! In:  eax = the car
//! Out: eax = its entry of `dword_5C58A0` (the car held to 0 .. 0x21); the rest preserved
//!
//! See `race/objects.rs`. One caller, `race::set_object_kinds`. Manual patch with a
//! private copy (a `jns` in its first bytes). Self-test: cars -5 .. 0x30.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::objects::{KIND_11_TABLE_RVA, car_table};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::car_kind_11", rva(0x41F485), thunk, &ORIGINAL)
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
    Ok(Left::nothing().with(EAX, car_table(KIND_11_TABLE_RVA, r.eax)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
