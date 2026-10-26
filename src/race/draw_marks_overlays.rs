//! `race::draw_marks_overlays` (0x45632B): `marks::draw_overlays`, then `marks::draw_shadow`
//! (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `draw_shadow` leaves them (its caller returns)
//!
//! It is its two calls: private copy (0xB, both relocated), manual patch. Refuses (before
//! anything) the first shadow's refusals (`marks::draw_shadow` would refuse after the
//! overlays were drawn). Caller: `sub_455AD3`. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins, against the Rust order.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::{draw_marks_overlays, overlays_part};
use crate::race::level_init::note;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_marks_overlays", rva(0x45632B), thunk, &ORIGINAL)
        .private_copy(0xB)
        .relocate_calls(&[0x0, 0x5])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_marks_overlays(r, &mut overlays_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_marks_overlays(r, &mut overlays_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45632B,
            len: 0xB,
            entry: &[0xE8, 0xAA, 0x11, 0x07, 0x00, 0xE8, 0x33, 0x1E],
            calls: &[0x0, 0x5],
            jumps_out: &[],
        },
        &|c| {
            [(0x4C74DAusize, 1u32), (0x4C8168, 2)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let line = compare_plain(
        "the two calls",
        &|| {},
        &[],
        copy.addr(),
        Regs::SENTINEL,
        &mut |r| unsafe {
            draw_marks_overlays(r, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        },
    );
    match line {
        // The Rust version's refusal (no marks settings and so on) is no mismatch here.
        Some(l) if l.contains("refused") => return t.skip("the marks' shadow would refuse"),
        line => t.check(line.is_none(), || line.unwrap_or_default()),
    }
    t.finish()
}
