//! `race::load_packed_game_map` (0x41B785): the in-race map read from its packed file.
//!
//! In:  eax = the path (`files\maps\mapxx.sq`)
//! Out: every register preserved
//!
//! See `race/game_map.rs`. One caller, `race::load_game_map`. Refuses a count of 0 or
//! past 0xFFFF and a short read not of whole pixels. Shadow mode runs the original alone
//! (it allocates and reads). Self-test: stage 21's map and a missing file, read by both,
//! compared with the pointers taken out; the map freed and the game's put back after
//! each.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::game_map::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::load_packed_game_map",
    rva(0x41B785),
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
    if let Err(e) = unsafe { load_packed_game_map(r.eax) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { load_packed_game_map(r.eax) }?;
    Ok(())
}

pub fn kept() -> Vec<Region> {
    vec![
        (game_addr(VARS.0) as u32, VARS.1),
        (game_addr(LIST_RVA) as u32, 0x40),
        (game_addr(KEY_LIST_RVA) as u32, 0x40),
        (game_addr(rva(0x93E8B0)) as u32, READ_BUFFER_LEN),
        (game_addr(PACKED_PATH_RVA) as u32, 0x40),
        crate::service::harness::at(rva(0x4F52B0)),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !lists_empty() {
        return t.skip("the map's lists hold blocks");
    }
    if !format_ready() {
        return t.skip("no pixel kinds yet");
    }
    let kept = kept();
    let saved = capture(&kept);
    for path in [&b"files\\maps\\map21.sq\0"[..], b"files\\maps\\map99.sq\0"] {
        let before = Regs {
            eax: path.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        restore(&kept, &saved);
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let theirs = (snapshot(), ro.preserved_except(&before, 0));
        let _ = unsafe { free_game_map() };
        restore(&kept, &saved);
        let mine = unsafe { load_packed_game_map(before.eax) };
        let ours = snapshot();
        let _ = unsafe { free_game_map() };
        let what = String::from_utf8_lossy(&path[..path.len() - 1]).into_owned();
        t.check(mine.is_ok() && theirs.0 == ours && theirs.1, || {
            let at = theirs.0.iter().zip(&ours).position(|(a, b)| a != b);
            format!(
                "{what}: rust {mine:?}, {} and {} values, first difference at {at:?} (tiles at 6), registers kept {}",
                theirs.0.len(),
                ours.len(),
                theirs.1
            )
        });
    }
    restore(&kept, &saved);
    t.finish()
}
