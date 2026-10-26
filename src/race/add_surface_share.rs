//! `race::add_surface_share` (0x445F7B): a surface's share of the stage, added to.
//!
//! In:  al = the surface (low four bits)
//! Out: with a weight, eax & 0xF; the rest preserved
//!
//! See `race/best_line.rs`. Eight call sites in `sub_445E30` (whose Rust version keeps the
//! exact x87 `add_surface_share`: it rounds the shares to whole numbers). The Rust version
//! is `add_surface_share_plain`. Shadow mode: by state, the shares within `FLOAT`.
//! Self-test, at the test precisions: the original and the Rust version on the game's
//! variables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, at_test_precisions, check_state_floats, dispatch_state_floats, floats_entry,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::add_surface_share", rva(0x445F7B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn shares() -> Region {
    (game_addr(SHARES_RVA) as u32, 0x40)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let weighted = unsafe { game_read::<u32>(WEIGHT_RVA) } != 0;
    // Converted: `rc2k_core::race::best_line::surface_share`.
    let g = unsafe { add_surface_share_plain(r) };
    Ok(if weighted {
        Left::nothing().with(EAX, g.eax)
    } else {
        Left::nothing()
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    let floats = Floats {
        memory: &[(shares(), FLOAT)],
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[shares()], &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [shares(), at(WEIGHT_RVA), at(CHECK_COUNT_RVA)];
    let saved = capture(&regions);
    let tolerant = Floats {
        memory: &[(shares(), FLOAT)],
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for (eax, weight, count) in [
            (0x1234_5603u32, 0x80u32, 7u32),
            (0xFFFF_FF1Fu32, 0x100, 1),
            (0x0Au32, 0, 7),
            (0x0Fu32, 0x37, 0x1234),
            (5u32, 0xFFFF_FF00, 3),
        ] {
            check_state_floats(
                &mut t,
                "race::add_surface_share",
                &format!(
                    "control word {word:#x}, eax {eax:#x}, weight {weight:#x}, {count} checkpoints"
                ),
                call_original,
                Regs {
                    eax,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    for n in 0..16u32 {
                        game_write(
                            SHARES_RVA + 4 * n as usize,
                            (n as f32 * 1.5 - 4.0).to_bits(),
                        );
                    }
                    game_write(WEIGHT_RVA, weight);
                    game_write(CHECK_COUNT_RVA, count);
                },
                &regions,
                floats,
                &rust,
            );
        }
    });
    restore(&regions, &saved);
    t.finish()
}
