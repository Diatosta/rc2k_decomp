//! `race::draw_skidmarks_if_on` (0x4585E3): `race::draw_skidmarks` unless `dword_6416B8`
//! is set (see `race/skidmarks.rs`).
//!
//! In:  nothing
//! Out: as `draw_skidmarks` leaves them, or all kept
//!
//! Caller: `sub_4557AB` (0x455853), each view drawn. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose call goes to a stand-in, the switch on and off.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_write, hook_asm, rva, shadow};
use crate::race::skidmarks::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::draw_skidmarks_if_on",
    rva(0x4585E3),
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
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_skidmarks_if_on(r, &mut |e| draw_skidmarks(e, &mut skid_part)) } {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLS: AtomicU32 = AtomicU32::new(0);

fn stand_in(_: u32, _: &mut Regs) {
    CALLS.fetch_add(1, Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4585E3,
            len: 0x10,
            entry: &[0x83, 0x3D, 0xB8, 0x16, 0x64, 0x00, 0x00, 0x74],
            calls: &[0xA],
            jumps_out: &[],
        },
        &|c| (c == rva(0x458C03)).then(|| crate::install::stand_ins::stand_in(1)),
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let kept = [at(rva(0x6416B8))];
    let saved = capture(&kept);
    for on in [0u32, 1] {
        unsafe { game_write(rva(0x6416B8), on) };
        CALLS.store(0, Ordering::SeqCst);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = CALLS.swap(0, Ordering::SeqCst);
        let _ = unsafe {
            draw_skidmarks_if_on(&Regs::SENTINEL, &mut |e| {
                CALLS.fetch_add(1, Ordering::SeqCst);
                Ok(*e)
            })
        };
        let mine = CALLS.swap(0, Ordering::SeqCst);
        t.check(theirs == mine, || {
            format!("switch {on}: original {theirs} calls, rust {mine}")
        });
    }
    restore(&kept, &saved);
    t.finish()
}
