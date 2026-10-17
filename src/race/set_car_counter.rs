//! `race::set_car_counter` (0x43A96C): car block esi's +0x71C eax, +0x730 0, +0x72C 0x12C.
//!
//! In:  esi = the block, eax = the value
//! Out: every register preserved
//!
//! See `race/frame.rs`. Six callers in `sub_43A649` (a `jmp` follows each). Shadow mode:
//! the Rust version, the dwords put back, the original's, compared. Self-test: a block of
//! the test's, values.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::frame::*;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::set_car_counter", rva(0x43A96C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_car_counter(r.esi, r.eax) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &counter_regions(r.esi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut block = vec![0x5A5A_5A5Au32; 0x800 / 4];
    let b = block.as_mut_ptr() as u32;
    for v in [0u32, 0xCA, 0xFFFF_FFFF] {
        check_state(
            &mut t,
            &format!("value {v:#x}"),
            call_original,
            Regs {
                esi: b,
                eax: v,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes((b + 0x700) as *mut u8, 0x5A, 0x40) },
            &[(b + 0x700, 0x40)],
            &rust,
        );
    }
    drop(block);
    t.finish()
}
