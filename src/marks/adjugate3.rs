//! `marks::adjugate3` (0x4BCED6): the adjugate of the 3 x 3 matrix at esi (rows 12 bytes
//! apart) into edi's: each entry a difference of two products (`fsubp`), rounded as written
//! (so an output over the input is read back as the original does).
//!
//! In:  esi the matrix, edi the result
//! Out: every register kept
//!
//! Caller: `sub_4BCB1D` (0x4BCD63). The original's instructions. Refuses a null pointer.
//! Self-test: the original and the Rust version on matrices by lot, apart and in place.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::adjugate3", rva(0x4BCED6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub unsafe fn adjugate3(s: u32, d: u32) -> Result<(), Refused> {
    if s == 0 || d == 0 {
        return Err(BadInput("a null matrix").into());
    }
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0x10]",
            "fmul dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x14]",
            "fmul dword ptr [{s} + 0x1C]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x0]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x20]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x4]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x14]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x10]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x8]",
            "fld dword ptr [{s} + 0x14]",
            "fmul dword ptr [{s} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fmul dword ptr [{s} + 0x20]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0xC]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x20]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0x18]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x10]",
            "fld dword ptr [{s} + 0x8]",
            "fmul dword ptr [{s} + 0xC]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x14]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x14]",
            "fld dword ptr [{s} + 0xC]",
            "fmul dword ptr [{s} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fmul dword ptr [{s} + 0x18]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x18]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0x18]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x1C]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x1C]",
            "fld dword ptr [{s} + 0x0]",
            "fmul dword ptr [{s} + 0x10]",
            "fld dword ptr [{s} + 0x4]",
            "fmul dword ptr [{s} + 0xC]",
            ".byte 0xDE, 0xE9",
            "fstp dword ptr [{d} + 0x20]",
            s = in(reg) s,
            d = in(reg) d,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { adjugate3(r.esi, r.edi) }?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.edi, 36)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut m = [0u32; 18];
    let base = m.as_mut_ptr() as u32;
    for case in 0..8u32 {
        let out = if case % 2 == 0 { base + 36 } else { base };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                esi: base,
                edi: out,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                for n in 0..18u32 {
                    let x = (n.wrapping_mul(0x9E37_79B9) ^ case.wrapping_mul(0x27D4_EB2F)) as i32
                        as f32
                        / 5e8;
                    ((base + 4 * n) as *mut u32).write(x.to_bits());
                }
            },
            &[(base, 72)],
            &rust,
        );
    }
    t.finish()
}
