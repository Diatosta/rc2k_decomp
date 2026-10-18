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
//! Self-test: `physics/camera_view.rs` (`view_test`), as `take_view_a`'s.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{rd, view_regions, view_test};
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::take_view_b", rva(0x438024), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
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
    let to = [
        0x608B8Cusize,
        0x608B90,
        0x608B94,
        0x608BD4,
        0x608BD8,
        0x608BDC,
        0x608C0C,
        0x608C10,
        0x608C14,
        0x608C08,
        0x608C18,
    ];
    for (k, x) in to.iter().enumerate() {
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { take_view_b(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &view_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    view_test(&HOOK, call_original, &rust)
}
