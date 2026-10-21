//! `marks::adjugate3` (0x4BCED6): the adjugate of the 3 x 3 matrix at esi (rows 12 bytes
//! apart) into edi's: each entry a difference of two products (`fsubp`), rounded as written
//! (so an output over the input is read back as the original does).
//!
//! In:  esi the matrix, edi the result
//! Out: every register kept
//!
//! Caller: `sub_4BCB1D` (0x4BCD63). The original's instructions. Refuses a null pointer.
//! Self-test: the original and the Rust version on matrices by lot, apart and in place.
//!
//! Converted 2026-10-21: the hook's Rust version is `adjugate3_plain`, through
//! `rc2k_core::marks::turns::adjugate` (plain floats; the x87 version stays for the Rust
//! callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
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

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::turns::adjugate`
pub unsafe fn adjugate3_plain(s: u32, d: u32) -> Result<(), Refused> {
    if s == 0 || d == 0 {
        return Err(BadInput("a null matrix").into());
    }
    let p = &mut unsafe { crate::scene::matrix::GamePlaces::new() };
    rc2k_core::marks::turns::adjugate(p, s, d);
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { adjugate3_plain(r.esi, r.edi) }?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&[(r.edi, 36)]);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[(r.edi, 36)], &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut m = [0u32; 18];
    let base = m.as_mut_ptr() as u32;
    let tolerant_places = float_places(&[(base, 72)]);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..8u32 {
            let out = if case % 2 == 0 { base + 36 } else { base };
            check_state_floats(
                &mut t,
                "marks::adjugate3",
                &format!("control word {word:#x}, case {case}"),
                call_original,
                Regs {
                    esi: base,
                    edi: out,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    for n in 0..18u32 {
                        let x = (n.wrapping_mul(0x9E37_79B9) ^ case.wrapping_mul(0x27D4_EB2F))
                            as i32 as f32
                            / 5e8;
                        ((base + 4 * n) as *mut u32).write(x.to_bits());
                    }
                },
                &[(base, 72)],
                held,
                &rust,
            );
        }
    });
    t.finish()
}
