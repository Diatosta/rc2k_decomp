//! `race::free_game_map` (0x41B907): the map's two lists freed, its pointers 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/game_map.rs`. Shadow mode runs the original alone (it frees). Self-test:
//! blocks of the test's in the two lists, freed by each.

use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva, shadow};
use crate::race::game_map::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::free_game_map", rva(0x41B907), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { free_game_map() } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !lists_empty() {
        return t.skip("the map's lists hold blocks");
    }
    let vars: [Region; 3] = [
        (crate::framework::game_addr(VARS.0) as u32, VARS.1),
        (crate::framework::game_addr(LIST_RVA) as u32, 0x40),
        (crate::framework::game_addr(KEY_LIST_RVA) as u32, 0x40),
    ];
    let saved = capture(&vars);
    let fill = || unsafe {
        for (r, bytes) in [
            (LIST_RVA, 0x100u32),
            (LIST_RVA, 0x2000),
            (KEY_LIST_RVA, 0x40),
        ] {
            let at =
                crate::memory::allocate::allocate(crate::framework::game_addr(r) as *mut _, bytes)
                    .map(|p| p as u32)
                    .unwrap_or(0);
            game_write(
                if r == KEY_LIST_RVA {
                    KEY_PIXELS_RVA
                } else {
                    TILES_RVA
                },
                at,
            );
        }
        game_write(TILE_PIXELS_RVA, 0x1234u32);
    };
    fill();
    let before = Regs::SENTINEL;
    let mut ro = before;
    unsafe { call_original(&mut ro) };
    let theirs = (capture(&vars), ro.preserved_except(&before, 0));
    fill();
    let mine = unsafe { free_game_map() }.is_ok();
    let ours = capture(&vars);
    t.check(mine && theirs.0 == ours && theirs.1, || {
        format!(
            "freed: rust ok {mine}, same {}, original kept the registers {}",
            theirs.0 == ours,
            theirs.1
        )
    });
    restore(&vars, &saved);
    t.finish()
}
