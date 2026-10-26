//! `race::face_depth_test_off` (0x452358): with no texture animation (`dword_63DA2C` 0)
//! and `g_polygonMode` 4, `scene::depth_test_off` and `dword_63D9E0` 1 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: as the depth test's call leaves them (eax 0, ebx 0xE), or all kept
//!
//! Caller: `sub_452C6F`, `sub_45232A`. Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose depth-test call goes to a stand-in, against the Rust
//! version with the same; the two switches.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_write, hook_asm, rva, shadow};
use crate::race::course_faces::*;
use crate::race::level_init::{note, take_called};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_depth_test_off", rva(0x452358), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { face_depth_test_off(r, &mut depth_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { face_depth_test_off(r, &mut depth_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    depth_test(
        &HOOK,
        0x452358,
        0x22,
        &[0x83, 0x3D, 0x2C, 0xDA, 0x63, 0x00, 0x00, 0x75],
        &[0x12],
        0x44E3D1,
        &|r, p| unsafe { face_depth_test_off(r, p) },
    )
}

/// The shared test of the two depth-test hooks: a copy of `address` whose depth-test call
/// (to `callee`) goes to a stand-in, the switches tried each way.
pub fn depth_test(
    hook: &RawHook,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    callee: usize,
    rust: &dyn Fn(&mut Regs, crate::race::frame::Part) -> Result<(), Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len,
            entry,
            calls,
            jumps_out: &[],
        },
        &|c| (c == rva(callee)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let regions: [Region; 3] = [at(rva(0x63DA2C)), at(rva(0x63DAB4)), at(rva(0x63D9E0))];
    let saved = capture(&regions);
    for (animated, mode, off) in [(0u32, 4u32, 0u32), (1, 4, 0), (0, 3, 1), (0, 4, 1)] {
        let prepare = || unsafe {
            game_write(rva(0x63DA2C), animated);
            game_write(rva(0x63DAB4), mode);
            game_write(rva(0x63D9E0), off);
            take_called();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (take_called(), capture(&regions));
        prepare();
        let mut rr = Regs::SENTINEL;
        let done = rust(&mut rr, &mut |p, r| {
            note(p, r);
            Ok(())
        });
        let mine = (take_called(), capture(&regions));
        t.check(done.is_ok() && theirs == mine, || {
            format!(
                "animated {animated} mode {mode} off {off}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
