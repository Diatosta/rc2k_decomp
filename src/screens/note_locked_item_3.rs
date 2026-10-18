//! `screens::note_locked_item_3` (0x4AAAB8): the message box for item 3 unless cheat 0x11 is on or `dword_75ACCC` is 1 (see `screens/locked_notes.rs`).
//!
//! In:  nothing (ecx passed on to the message box)
//! Out: eax as it leaves it; the rest kept (the box is `pusha` .. `popa`)
//!
//! In a menu loop's draw list (0x75912E). Shadow mode runs the original alone (the box
//! draws on the surface). Self-test: a copy whose box call goes to a stand-in, against the
//! Rust version with the same: the cheat each way, items 3 and others, the switch each way; all put back.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::screens::locked_notes::{box_part, note_locked_item_3};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::note_locked_item_3",
    rva(0x4AAAB8),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::note_locked_item_3") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    match unsafe { note_locked_item_3(&mut g, &mut box_part) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4AAAB8,
            len: 0x23,
            entry: &[0x0F, 0xBA, 0x25, 0x0C, 0x87, 0x75, 0x00, 0x11],
            calls: &[0x1D],
            jumps_out: &[],
        },
        &|c| (c == rva(0x415EA6)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let kept: Vec<Region> = vec![at(rva(0x75870C)), at(rva(0x755525)), at(rva(0x75ACCC))];
    let saved = capture(&kept);
    for (cheats, item, other) in [
        (0u32, 3u32, 0u32),
        (0x2_0000, 3, 0),
        (0, 3, 1),
        (0, 2, 0),
        (0xFFFD_FFFF, 3, 7),
    ] {
        let line = compare_plain(
            &format!("cheats {cheats:#x} item {item} other {other}"),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x75870C), cheats);
                game_write(rva(0x755525), item);
                game_write(rva(0x75ACCC), other);
            },
            &[],
            copy.addr(),
            Regs::SENTINEL,
            &mut |r| unsafe {
                let left = note_locked_item_3(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })?;
                left.apply(r);
                Ok(())
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    t.finish()
}
