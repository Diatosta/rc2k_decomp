//! `race::swap_stage_texture` (0x439B03): the stage's texture swap (see `swap_stage_texture`).
//!
//! In:  nothing
//! Out: eax, ebx, ecx as the searches leave them
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::swap_stage_texture", rva(0x439B03), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(rva(0x608DB4)) as u32, 0x400),
        (game_addr(rva(0x60A67C)) as u32, 0x100),
        at(STAGE_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { swap_stage_texture(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    for stage in [0x20u32, 0x36, 0x41, 0x42, 0x10] {
        for (old_at, list_full) in [
            (Some(5u32), false),
            (None, false),
            (Some(0xFE), true),
            (Some(0), false),
        ] {
            check_state(
                &mut t,
                &format!("stage {stage:#x}, old kind at {old_at:?}, list full {list_full}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    for n in 0..0x100u32 {
                        game_write(rva(0x608DB4) + 4 * n as usize, 0x1000 + n);
                    }
                    if let Some(at) = old_at {
                        let old = match stage {
                            0x20 => 0x2F82u32,
                            0x36 => 0x2F91,
                            0x41 => 0x2F72,
                            _ => 0x2F71,
                        };
                        game_write(rva(0x608DB4) + 4 * at as usize, old);
                    }
                    for n in 0..0x40u32 {
                        game_write(
                            rva(0x60A67C) + 4 * n as usize,
                            if list_full || n < 3 { 0x77 + n } else { 0 },
                        );
                    }
                    game_write(STAGE_RVA, stage);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
