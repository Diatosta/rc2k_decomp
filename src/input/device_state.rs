//! `input::device_state` (0x408A49): device esi's four values (see `input/axes.rs`).
//!
//! In:  esi = the device
//! Out: ecx, edx, eax, ebx = +0x2C, +0x30, +0x34, +0x38; the rest kept
//!
//! Caller: `input::first_device_state`. Shadow mode: by state. Self-test: a test device.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::axes::device_state;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::device_state", rva(0x408A49), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The four as registers.
pub fn left(v: [u32; 4]) -> Left {
    Left::nothing()
        .with(ECX, v[0])
        .with(EDX, v[1])
        .with(EAX, v[2])
        .with(EBX, v[3])
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left(unsafe { device_state(r.esi) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let device: Vec<u32> = (0..0x10u32).map(|k| k.wrapping_mul(0x1234_5679)).collect();
    check_state(
        &mut t,
        "a test device",
        call_original,
        Regs {
            esi: device.as_ptr() as u32,
            ..Regs::SENTINEL
        },
        &|| {},
        &[],
        &rust,
    );
    t.finish()
}
