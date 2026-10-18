//! `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0), texture and colour 0,
//! drawn as a fan (`scene::draw_fan`); see `race/frame_views.rs`.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `sub_4559A6`. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose draw goes to a stand-in, against the Rust version with the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::face_stage::compare_plain;
use crate::race::frame_views::draw_flat_triangle;
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_flat_triangle", rva(0x455912), thunk, &ORIGINAL).self_test(self_test);

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
    unsafe {
        draw_flat_triangle(r, &mut |_, r| {
            crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part)
        })
    };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455912,
            len: 0x94,
            entry: &[0x60, 0xD9, 0xE8, 0xD9, 0x15, 0xA0, 0x05, 0x96],
            calls: &[0x8D],
            jumps_out: &[],
        },
        &|c| (c == rva(0x44EA65)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions: Vec<Region> = vec![(game_addr(rva(0x9605A0)) as u32, 0x60), at(rva(0x6240F8))];
    let saved = capture(&regions);
    let line = compare_plain(
        "filled",
        &|| restore(&regions, &vec![0xAA; saved.len()]),
        &regions,
        copy.addr(),
        Regs::SENTINEL,
        &mut |r| {
            unsafe { draw_flat_triangle(r, &mut |p, r| note(p, r)) };
            Ok(())
        },
    );
    t.check(line.is_none(), || line.unwrap_or_default());
    restore(&regions, &saved);
    t.finish()
}
