//! `physics::take_view_a` (0x437F53): a view's figures (the record at esi) into the
//! first view: +4 .. +0xC plus `dword_637638` .. into `flt_608B98` ..; +0x10 .. +0x24
//! into `flt_608B80` .., `flt_608BC8` ..; +0x28 .. +0x38 into `dword_608BF8`,
//! `608BFC`, `608C00`, `608BF4`, `608C04`; the heading (`unk_71A998`) into
//! `dword_608B5C`; the next two (`unk_71A99C`, `unk_71A9A0`), halved (`sar`) when the
//! mode's record has +0x4C set, into `dword_608B64`, `dword_608B60`.
//!
//! In:  esi the view's record
//! Out: eax, ebx the two, esi the mode's record; the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`): the original and the
//! converted version on the game's variables (saved and put back) and a test record,
//! modes by lot, exact at double precision.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::{rd, record};
use crate::physics::flow::{g, p, x87_run};

/// The Rust version (the exact one, for `physics::update_camera`): `s` the view's record
/// (esi). The registers it leaves are read by none of its callers.
pub unsafe fn take_view_a(s: u32) {
    x87_run!(
        "fld dword ptr [{s} + 0x4]",
        "fadd dword ptr [{rc} + 0x237638]",
        "fstp dword ptr [{rc} + 0x208B98]",
        "fld dword ptr [{s} + 0x8]",
        "fadd dword ptr [{rc} + 0x23763C]",
        "fstp dword ptr [{rc} + 0x208B9C]",
        "fld dword ptr [{s} + 0xC]",
        "fadd dword ptr [{rc} + 0x237640]",
        "fstp dword ptr [{rc} + 0x208BA0]";
        s = s, rc = crate::race::store::asm_base()
    );
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B5C, g(0x71A998));
    let (mut eax, mut ebx) = (g(0x71A99C), g(0x71A9A0));
    if rd(record() + 0x4C) != 0 {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    p(0x608B64, eax);
    p(0x608B60, ebx);
}

/// Where the record's +0x10 .. +0x38 go.
const COPIES: [usize; 11] = [
    0x608B80, 0x608B84, 0x608B88, 0x608BC8, 0x608BCC, 0x608BD0, 0x608BF8, 0x608BFC, 0x608C00,
    0x608BF4, 0x608C04,
];
