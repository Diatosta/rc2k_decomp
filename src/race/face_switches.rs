//! `race::face_switches` (0x45322C): `dword_6240E0` / `dword_6240E4` 1 when the face's
//! first / second texture is animated (`dword_63DA2C` / `dword_63DA38`), else 0 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Caller: `sub_452C6F`. Shadow mode: by state. Self-test: the four combinations.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::course_faces::face_switches;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_switches", rva(0x45322C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { face_switches() };
    Ok(Left::nothing())
}

fn regions() -> [Region; 2] {
    [at(rva(0x6240E0)), at(rva(0x6240E4))]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let kept = [regions[0], regions[1], at(rva(0x63DA2C)), at(rva(0x63DA38))];
    let saved = capture(&kept);
    for (a, b) in [(0u32, 0u32), (5, 0), (0, 0x80), (1, 1)] {
        check_state(
            &mut t,
            &format!("animations {a} {b}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(rva(0x63DA2C), a);
                game_write(rva(0x63DA38), b);
                game_write(rva(0x6240E0), 0x55u32);
                game_write(rva(0x6240E4), 0x66u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
