//! `race::panel_players` (0x45ACDB): the panel's car fields by players: unless
//! `dword_62389C` is set, `dword_6D141A` and `dword_6D144A` -1; with more than one player
//! also `dword_6D1426` and `dword_6D142E` -1, and `dword_6D141A` / `dword_6D144A` the two
//! view blocks' +0xFA4.
//!
//! In:  nothing
//! Out: eax the second block's +0xFA4 when written, else kept (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB40), the panel's set-up each
//! frame. The fields (unaligned, in a block at 0x6D14xx) are a reading. Shadow mode: by
//! state. Self-test: the game's view blocks' +0xFA4 and the variables set, each case of
//! the two tests; all put back (skipped without view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::panel_players", rva(0x45ACDB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const REPLAYING: usize = rva(0x62389C); // dword_62389C
const PLAYERS: usize = rva(0x63DB94); // dword_63DB94

fn regions() -> Vec<Region> {
    PANEL_FIELDS.iter().map(|&a| at(a)).collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_players() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(REPLAYING), at(PLAYERS)]);
    kept.extend(blocks.iter().map(|&b| (b + 0xFA4, 4)));
    let saved = capture(&kept);
    for (replaying, players) in [(0u32, 1u32), (0, 2), (1, 2), (0, 0), (0xFFFF_FFFF, 1)] {
        check_state(
            &mut t,
            &format!("replaying {replaying:#x} players {players}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for (k, &a) in PANEL_FIELDS.iter().enumerate() {
                    game_write(a, 0x5A5A_0000 + k as u32);
                }
                game_write(REPLAYING, replaying);
                game_write(PLAYERS, players);
                ((blocks[0] + 0xFA4) as *mut u32).write_unaligned(0x1234);
                ((blocks[1] + 0xFA4) as *mut u32).write_unaligned(0x5678);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
