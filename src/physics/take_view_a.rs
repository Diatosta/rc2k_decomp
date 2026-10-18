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
//! Self-test: `physics/camera_view.rs` (`view_test`): the original and the Rust version
//! on the game's variables (saved and put back) and a test record, modes by lot, under
//! three precisions.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ESI, RawHook, Regs, hook_asm, rva};
use crate::physics::camera_view::{rd, record, view_regions, view_test};
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::take_view_a", rva(0x437F53), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
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
    let to = [
        0x608B80usize,
        0x608B84,
        0x608B88,
        0x608BC8,
        0x608BCC,
        0x608BD0,
        0x608BF8,
        0x608BFC,
        0x608C00,
        0x608BF4,
        0x608C04,
    ];
    for (k, x) in to.iter().enumerate() {
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { take_view_a(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &view_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    view_test(&HOOK, call_original, &rust)
}
