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
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), as `take_view_a`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs};
use crate::physics::camera_view::rd;
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::camera_view::{view_place, view_turns};

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn take_view_b(entry: &Regs) -> Left {
    let s = entry.esi;
    x87_run!(
        "fld dword ptr [{s} + 0x4]",
        "fadd dword ptr [{b} + 0x237638]",
        "fstp dword ptr [{b} + 0x208BB0]",
        "fld dword ptr [{s} + 0x8]",
        "fadd dword ptr [{b} + 0x23763C]",
        "fstp dword ptr [{b} + 0x208BB4]",
        "fld dword ptr [{s} + 0xC]",
        "fadd dword ptr [{b} + 0x237640]",
        "fstp dword ptr [{b} + 0x208BB8]";
        b = base(), s = s
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
    Left::nothing().with(EAX, eax).with(EBX, ebx)
}

/// The converted version (`rc2k_core::physics::camera_view`, plain floats): the hook's.
// Converted: `rc2k_core::physics::camera_view::{view_place, view_turns}`.
pub unsafe fn take_view_b_plain(entry: &Regs) -> Left {
    let s = entry.esi;
    let f = |x: u32| f32::from_bits(rd(x));
    let place = view_place(
        [f(s + 4), f(s + 8), f(s + 0xC)],
        [0x637638, 0x63763C, 0x637640].map(|x| f32::from_bits(g(x))),
    );
    for (k, v) in place.iter().enumerate() {
        p(0x608BB0 + 4 * k, v.to_bits());
    }
    for (k, x) in COPIES.iter().enumerate() {
        p(*x, rd(s + 0x10 + 4 * k as u32));
    }
    p(0x608B68, g(0x71A998));
    let halve = !matches!(g(0x71A660), 1..=3);
    let [eax, ebx] = view_turns([g(0x71A99C), g(0x71A9A0)], halve);
    p(0x608B70, eax);
    p(0x608B6C, ebx);
    Left::nothing().with(EAX, eax).with(EBX, ebx)
}

/// Where the record's +0x10 .. +0x38 go.
const COPIES: [usize; 11] = [
    0x608B8C, 0x608B90, 0x608B94, 0x608BD4, 0x608BD8, 0x608BDC, 0x608C0C, 0x608C10, 0x608C14,
    0x608C08, 0x608C18,
];

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { take_view_b_plain(r) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
