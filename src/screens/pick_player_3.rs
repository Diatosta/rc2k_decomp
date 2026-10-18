//! `screens::pick_player_3` (0x46AF58): player 2 the current one, its settings into the page's copy (see `screens/page_settings.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a page's unread list (0x759496). Refuses (before anything) a player without its record. Shadow mode: by state. Self-test: the player's record as it is and with test bytes;
//! all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::screens::page_settings::*;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::pick_player_3", rva(0x46AF58), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { pick_player(2) }?;
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![at(rva(0x75598C)), at(rva(0x7558C4)), g(0x7555B1, 0x2C)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let p = crate::menu::player::player_at(2);
    if p == 0 {
        return t.skip("no player record");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.push((p + 0x2C, 0x50));
    let saved = capture(&kept);
    for seed in [0u32, 1, 2] {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                if seed != 0 {
                    for k in 0..0x30u32 {
                        ((p + 0x2C + k) as *mut u8).write((k * 7 + seed) as u8);
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
