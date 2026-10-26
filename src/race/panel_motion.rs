//! `race::panel_motion` (0x45AE1D): for each view block, +0xAE8 and +0xAEC x 0.5 / 65536
//! and (-(+0x9C0)) >> 14 x pi x 0.5 / 65536 (floats, x87) into the panel (`flt_6D1432`,
//! `flt_6D1436`, `flt_6D143A`; `flt_6D145A`, `flt_6D145E`, `flt_6D1462`); then
//! `dword_62400C` = `g_sceneClock` - `dword_62290C` - 800, 0 when negative.
//!
//! In:  nothing
//! Out: esi the second block, eax that time (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB45), each frame drawn. The values
//! (+0xAE8/+0xAEC speeds, +0x9C0 a heading: a reading) are not settled. Shadow mode: by
//! state, the six floats within `FLOAT`. The Rust version is `panel_motion_plain`.
//! Self-test, at the test precisions: the game's view blocks with patterns, clocks before
//! and after the 800; all put back (skipped without view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::floats::{
    Floats, at_test_precisions, check_state_floats, dispatch_state_floats, floats_entry,
};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::panel_motion", rva(0x45AE1D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn view_blocks() -> Vec<u32> {
    VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::panel::{motion, heading_turn, heading, shown_for}`.
    Ok(unsafe { panel_motion_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = motion_float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &panel_regions(), &floats, &rust);
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
    let views = view_blocks();
    if views.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = panel_regions();
    let mut kept = regions.clone();
    kept.extend([at(SCENE_CLOCK), at(SHOWN_SINCE)]);
    for &b in &views {
        kept.extend([(b + 0xAE8, 8), (b + 0x9C0, 4)]);
    }
    let saved = capture(&kept);
    let places = motion_float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, floats| {
        for (seed, clock, since) in [
            (1u32, 5000u32, 1000u32),
            (0x8000_0001, 1000, 900),
            (0xDEAD_BEEF, 0, 0xFFFF_FF00),
            (0x1234_5678, 0x8000_0000, 0),
        ] {
            check_state_floats(
                &mut t,
                "race::panel_motion",
                &format!("control word {word:#x}, seed {seed:#x} clock {clock} since {since}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                    for &(a, len) in &regions {
                        std::ptr::write_bytes(a as *mut u8, 0xAA, len);
                    }
                    game_write(SCENE_CLOCK, clock);
                    game_write(SHOWN_SINCE, since);
                    for (n, &b) in views.iter().enumerate() {
                        let s = seed.wrapping_mul(n as u32 * 2 + 1);
                        w(b + 0xAE8, s);
                        w(b + 0xAEC, s.rotate_left(7));
                        w(b + 0x9C0, s.rotate_left(13));
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
