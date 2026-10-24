//! `marks::adjugate3` (0x4BCED6): the adjugate of the 3 x 3 matrix at esi (rows 12 bytes
//! apart) into edi's: each entry a difference of two products (`fsubp`), rounded as written
//! (so an output over the input is read back as the original does).
//!
//! In:  esi the matrix, edi the result
//! Out: every register kept
//!
//! Caller: `sub_4BCB1D` (0x4BCD63). The original's instructions. Refuses a null pointer.
//!
//! the x87 version stays for the Rust callers in unconverted code).

use crate::errors::{BadInput, Refused};

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
