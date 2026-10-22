//! `race::map_tiles_stub` (0x45B68F): `call sub_41B931; retn`: the map's tiles blitted into
//! the map picture (the game's software blit, section 15); see `race/map_scroll.rs`.
//!
//! In:  eax = the picture's slot, ecx / edx = the first tile
//! Out: as `sub_41B931` leaves them (`pusha` .. `popa`: all kept)
//!
//! It is its call: private copy (6, the call relocated), manual patch. Caller:
//! `race::map_tiles`. Shadow mode runs the original alone. Self-test: a copy whose call goes
//! to a stand-in, against the Rust side's one call.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::race::map_scroll::tiles_part;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::map_tiles_stub", rva(0x45B68F), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x0])
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
    let _ = tiles_part(1, r);
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let _ = tiles_part(1, r);
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
            address: 0x45B68F,
            len: 0x6,
            entry: &[0xE8, 0x9D, 0x02, 0xFC, 0xFF, 0xC3],
            calls: &[0x0],
            jumps_out: &[],
        },
        &|c| (c == rva(0x41B931)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let line = compare_plain(
        "one call",
        &|| {},
        &[],
        copy.addr(),
        Regs::SENTINEL,
        &mut |r| {
            note(1, r);
            Ok(())
        },
    );
    t.check(line.is_none(), || line.unwrap_or_default());
    t.finish()
}
