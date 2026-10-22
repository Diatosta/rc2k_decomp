//! `race::panel_values` (0x45AB5A): both view blocks' values into the panel fields at
//! 0x6D13FE .. 0x6D1472: +0x718 rounded (`fistp`), +0xAB0, +0x1D4, the distance from
//! checkpoint `dword_60102C` to the block's +0x98 (`g_checkpointDistances`, halved, then
//! twice that over 65536.0 and over `g_metresInMile`, a float), +0x88 x 10, +0xA0C,
//! -(+0xA10), +0xA14; for the first view also +0xA18, +0xA1C, +0xA20, +0xA24.
//!
//! In:  nothing
//! Out: esi the second view's block, ebx `dword_60102C`, eax its +0x88 x 10 (a `call`
//!      follows); `dword_62400C` (a scratch dword) the second view's halved distance
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB36), each frame drawn. The
//! fields' meaning (speed, gear, distance in miles, time: a reading) is not settled. The
//! checkpoint indices are not checked (the original reads the table unchecked). The Rust
//! version is `panel_values_plain` (`set_up_panel` keeps the exact one). Shadow mode: by
//! state, the two distances within `FLOAT`. Self-test, at the test precisions: the game's
//! view blocks with patterns, checkpoint distances 0 .. 7 set, both orders of distance;
//! all put back (skipped without view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
    floats_entry,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::panel_values", rva(0x45AB5A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DISTANCES: usize = rva(0x9F78D8); // g_checkpointDistances
const START: usize = rva(0x60102C); // dword_60102C

fn regions() -> Vec<Region> {
    vec![
        (game_addr(PANEL_START_RVA) as u32, PANEL_LEN),
        at(SCRATCH_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { panel_values_plain() })
}

/// The distances shown, within `FLOAT`.
fn float_places() -> [(Region, Tolerance); 2] {
    PANEL_FLOATS.map(|a| (at(a), FLOAT))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
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
    let blocks: Vec<u32> = VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(START), (game_addr(DISTANCES) as u32, 0x20)]);
    kept.extend(blocks.iter().map(|&b| (b, 0xFE0)));
    let saved = capture(&kept);
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for (seed, start, checkpoints) in [
            (1u32, 0u32, [3u32, 5]),
            (0x5A5A_1234, 4, [2, 7]),
            (0xDEAD_BEEF, 7, [7, 0]),
        ] {
            check_state_floats(
                &mut t,
                "race::panel_values",
                &format!(
                    "control word {word:#x}, seed {seed:#x} start {start} checkpoints {checkpoints:?}"
                ),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                    std::ptr::write_bytes(game_addr(PANEL_START_RVA) as *mut u8, 0xAA, PANEL_LEN);
                    game_write(START, start);
                    for k in 0..8u32 {
                        game_write(DISTANCES + 4 * k as usize, k * 1_000_003);
                    }
                    for (n, &b) in blocks.iter().enumerate() {
                        let s = seed.wrapping_mul(n as u32 + 3);
                        for off in [
                            0x88u32, 0x1D4, 0xAB0, 0xA0C, 0xA10, 0xA14, 0xA18, 0xA1C, 0xA20, 0xA24,
                        ] {
                            w(b + off, s.rotate_left(off));
                        }
                        // A float near a few hundred, by the seed.
                        w(b + 0x718, f32::to_bits((s % 50_000) as f32 / 97.0 - 100.0));
                        w(b + 0x98, checkpoints[n]);
                    }
                },
                &regions,
                floats,
                &rust,
            );
        }
    });
    restore(&kept, &saved);
    t.finish()
}
