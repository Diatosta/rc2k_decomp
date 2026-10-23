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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ESI, Regs};
use crate::physics::camera_view::{rd, record, record_plain};
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::camera_view::{view_place, view_turns};

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn take_view_a(entry: &Regs) -> Left {
    let s = entry.esi;
    x87_run!(
        "fld dword ptr [{s} + 0x4]",
        "fadd dword ptr [{b} + 0x237638]",
        "fstp dword ptr [{b} + 0x208B98]",
        "fld dword ptr [{s} + 0x8]",
        "fadd dword ptr [{b} + 0x23763C]",
        "fstp dword ptr [{b} + 0x208B9C]",
        "fld dword ptr [{s} + 0xC]",
        "fadd dword ptr [{b} + 0x237640]",
        "fstp dword ptr [{b} + 0x208BA0]";
        b = base(), s = s
    );
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B5C, g(0x71A998));
    let (mut eax, mut ebx) = (g(0x71A99C), g(0x71A9A0));
    let esi = record();
    if rd(esi + 0x4C) != 0 {
        eax = ((eax as i32) >> 1) as u32;
        ebx = ((ebx as i32) >> 1) as u32;
    }
    p(0x608B64, eax);
    p(0x608B60, ebx);
    Left::nothing().with(EAX, eax).with(EBX, ebx).with(ESI, esi)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::{view_place, view_turns}`.
pub unsafe fn take_view_a_plain(entry: &Regs) -> Left {
    let s = entry.esi;
    let f = |x: u32| f32::from_bits(rd(x));
    let place = view_place(
        [f(s + 4), f(s + 8), f(s + 0xC)],
        [0x637638, 0x63763C, 0x637640].map(|x| f32::from_bits(g(x))),
    );
    for (k, v) in place.iter().enumerate() {
        p(0x608B98 + 4 * k, v.to_bits());
    }
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B5C, g(0x71A998));
    let esi = record_plain();
    let [eax, ebx] = view_turns([g(0x71A99C), g(0x71A9A0)], rd(esi + 0x4C) != 0);
    p(0x608B64, eax);
    p(0x608B60, ebx);
    Left::nothing().with(EAX, eax).with(EBX, ebx).with(ESI, esi)
}

/// Where the record's +0x10 .. +0x38 go.
const COPIES: [usize; 11] = [
    0x608B80, 0x608B84, 0x608B88, 0x608BC8, 0x608BCC, 0x608BD0, 0x608BF8, 0x608BFC, 0x608C00,
    0x608BF4, 0x608C04,
];

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { take_view_a_plain(r) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
