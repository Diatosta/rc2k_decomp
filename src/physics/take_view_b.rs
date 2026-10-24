//! `physics::take_view_b` (0x438024): `physics::take_view_a` for the second view: +4 ..
//! +0xC plus `dword_637638` .. into `flt_608BB0` ..; +0x10 .. +0x24 into `flt_608B8C`
//! .., `flt_608BD4` ..; +0x28 .. +0x38 into `dword_608C0C`, `608C10`, `608C14`,
//! `608C08`, `608C18`; the heading into `dword_608B68`; the next two halved (`sar`)
//! unless the mode before (`unk_71A660`) is 1, 2 or 3, into `dword_608B70`,
//! `dword_608B6C`.
//!
//! In:  esi the view's record
//! Out: eax, ebx the two; the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::rd;
use crate::physics::flow::{g, p, x87_run};

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The eax and ebx it leaves are read by none of its callers.
pub unsafe fn take_view_b(s: u32) {
    x87_run!(
        "fld dword ptr [{s} + 0x4]",
        "fadd dword ptr [{rc} + 0x237638]",
        "fstp dword ptr [{rc} + 0x208BB0]",
        "fld dword ptr [{s} + 0x8]",
        "fadd dword ptr [{rc} + 0x23763C]",
        "fstp dword ptr [{rc} + 0x208BB4]",
        "fld dword ptr [{s} + 0xC]",
        "fadd dword ptr [{rc} + 0x237640]",
        "fstp dword ptr [{rc} + 0x208BB8]";
        s = s, rc = crate::race::store::asm_base()
    );
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B68, g(0x71A998));
    let (mut eax, mut ebx) = (g(0x71A99C), g(0x71A9A0));
    if !matches!(g(0x71A660), 1..=3) {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    p(0x608B70, eax);
    p(0x608B6C, ebx);
}

/// Where the record's +0x10 .. +0x38 go.
const COPIES: [usize; 11] = [
    0x608B8C, 0x608B90, 0x608B94, 0x608BD4, 0x608BD8, 0x608BDC, 0x608C0C, 0x608C10, 0x608C14,
    0x608C08, 0x608C18,
];
