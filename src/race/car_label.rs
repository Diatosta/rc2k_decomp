//! `race::car_label` (0x45B1B7): seven bytes at edi, " CAR nn" (nn eax's two digits: eax
//! / 10 and eax % 10, each + '0' as a byte), or seven spaces when ecx is -1.
//!
//! In:  edi = where, eax = the number, ecx = -1 for none
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! See `race/frame_draw.rs`. Callers: `sub_45AD36` (0x45ADB0, 0x45AE0A), the panel's
//! texts; a `mov` follows both. Shadow mode: by state (the seven bytes). Self-test: a test
//! buffer filled with 0xAA, numbers 0, 7, 42, 99, 100, 255, 0xFFFFFFFE, and ecx -1.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_label", rva(0x45B1B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { car_label(r.edi, r.eax, r.ecx) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.edi, 7)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = Box::new([0u8; 16]);
    let p = buffer.as_mut_ptr() as u32;
    let regions: [Region; 1] = [(p, 16)];
    for (eax, ecx) in [
        (0u32, 0u32),
        (7, 1),
        (42, 0),
        (99, 0x8000_0000),
        (100, 0),
        (255, 0),
        (0xFFFF_FFFE, 0xFFFF_FFFE),
        (42, u32::MAX),
    ] {
        check_state(
            &mut t,
            &format!("eax {eax:#x} ecx {ecx:#x}"),
            call_original,
            Regs {
                eax,
                ecx,
                edi: p + 4,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(p as *mut u8, 0xAA, 16) },
            &regions,
            &rust,
        );
    }
    drop(buffer);
    t.finish()
}
