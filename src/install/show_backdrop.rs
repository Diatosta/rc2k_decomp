//! `install::show_backdrop` (0x41525D): three frames (`screens::run_frame`) of the
//! installer's backdrop alone (`install::backdrop_frame`), as a page opens and closes.
//!
//! In:  nothing (the callers set or clear carry before it; it does not read it)
//! Out: every register kept (`pusha` / `popa`); the flags of the last frame's run and
//!      `dec ebp` (zero set; no caller reads them)
//! Caller `install::run_page` (twice). Its calls through `part` (the frame's run is the
//! platform's). Ungated (it runs frames). Shadow mode runs the original alone. Self-test:
//! a copy whose frame runs go to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{Part, RUN_FRAME, ZF, a, at, both, copy, quiet};
use crate::race::viewer::{dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::show_backdrop", rva(0x41525D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

pub const FRAME: usize = 0x415273; // install::backdrop_frame

pub fn show_backdrop(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs { ebp: 3, ..*entry };
    for _ in 0..3 {
        r.eax = a(FRAME);
        part(at(RUN_FRAME), &mut r)?;
        r.ebp = r.ebp.wrapping_sub(1);
        r.set_zero(r.ebp == 0);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        show_backdrop(r, &mut crate::install::installer::game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        show_backdrop(r, &mut crate::install::installer::game_part)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov ebp, 3
    let copy = match copy(
        0x41525D,
        0x16,
        &[0x60, 0xBD, 0x03, 0x00, 0x00, 0x00],
        &[(0xC, 0x405A71)],
        &[0x405A71],
        &[],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    both(
        &mut t,
        "three frames",
        &copy,
        Regs::SENTINEL,
        &[],
        ZF,
        &[],
        &|| {},
        &|r, part, _| show_backdrop(r, part),
    );
    t.finish()
}
