//! `gfx::screenshot_if_key` (0x43B573): a screenshot while key 0x52 is down.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `input/keys.rs`. One caller, `input::end_frame`. `gfx::take_screenshot` is not
//! ported (a call into the game, section 15). Shadow mode runs the original alone (it
//! may take a screenshot). Self-test: a copy whose `take_screenshot` is a stand-in
//! counting, the key up and down.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::input::keys::*;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::screenshot_if_key", rva(0x43B573), thunk, &ORIGINAL).self_test(self_test);

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
    unsafe { screenshot_if_key(&mut game_screenshot) };
}

/// The stand-in's count of screenshots.
static SHOTS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn shot_stand_in(_: u32, _: &mut Regs) {
    SHOTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    use std::sync::atomic::Ordering;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(shot_stand_in);
    let copy = match crate::race::mud::copy(
        0x43B573,
        0x13,
        &[0x60, 0xF6, 0x05, 0xAE, 0x6B, 0x4F, 0x00, 0xFF],
        &[0xB],
        &[(0x407897, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(shot_stand_in);
    let regions = regions();
    let saved = capture(&regions);
    for (screenshots, key) in [(0u32, 0u8), (0, 1), (1, 0), (1, 1)] {
        let prepare = || unsafe {
            set_state(0x77, 0x100);
            crate::framework::game_write(SCREENSHOTS_RVA, screenshots);
            crate::framework::game_write(NOW_RVA + 0x52, key);
        };
        prepare();
        SHOTS.store(0, Ordering::SeqCst);
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (
            capture(&regions),
            SHOTS.swap(0, Ordering::SeqCst),
            ro.preserved_except(&Regs::SENTINEL, 0),
        );
        prepare();
        unsafe {
            screenshot_if_key(&mut || {
                shot_stand_in(0, &mut { Regs::SENTINEL });
            })
        };
        let mine = (capture(&regions), SHOTS.swap(0, Ordering::SeqCst), true);
        t.check(theirs == mine, || {
            format!("screenshots {screenshots}, key {key}: original {} shots, rust {}; same state {}; registers kept {}", theirs.1, mine.1, theirs.0 == mine.0, theirs.2)
        });
    }
    restore(&regions, &saved);
    t.finish()
}
