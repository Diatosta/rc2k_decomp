//! `race::countdown_lights` (0x45B008): one view's start lights, from the time since the
//! countdown began (+0x14 - `dword_6D1814`, held to 1500).
//!
//! In:  edi 0 or 0x10 (the view's fields at 0x6D1482 + edi: lit, fade, size, shift), esi
//!      the view block, ebp where the "on a whole step" flag goes
//! Out: as `countdown_lights` in `race/frame_draw.rs` gives (eax, ebx, ecx, edx by path)
//!
//! Steps: fade, shift and the flag 0 (size: `sub [..], 0`, unchanged); nothing more when
//! +0x1C is set or the time is negative; `dword_6D1810` = hundreds (at most 5), lit = the
//! hundreds (-1 from 6); the flag 1 on a whole hundred (with lit not -1), else the rest
//! x 2 taken from 0x100 for fade and size; with five lit, fade = 0x80 x (held 0 .. 0x100)
//! / 0x30 and shift = half what that took away. Callers: `race::both_countdown_lights`
//! (twice; the next is a `mov` / `retn`). A reading: the five lights before the start.
//! Shadow mode: by state. Self-test: a test block and flag cell, times around each step,
//! both edi; the game's fields put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::countdown_lights", rva(0x45B008), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { countdown_lights(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = light_regions();
    regions.push((r.ebp, 4));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut block = vec![0u32; 0x10];
    let mut flag = Box::new(0u32);
    let b = block.as_mut_ptr() as u32;
    let f = &mut *flag as *mut u32 as u32;
    let mut regions = light_regions();
    regions.push((f, 4));
    let mut kept = regions.clone();
    kept.push(at(LIGHTS_START));
    let saved = capture(&kept);
    let times = [
        0u32,
        1,
        99,
        100,
        150,
        400,
        499,
        500,
        501,
        550,
        599,
        600,
        1499,
        1500,
        1600,
        0x7FFF_FFFF,
        0xFFFF_FFFF,
    ];
    for edi in [0u32, 0x10] {
        for (n, &time) in times.iter().enumerate() {
            let held = u32::from(n == 3);
            check_state(
                &mut t,
                &format!("edi {edi:#x} time {time} held {held}"),
                call_original,
                Regs {
                    edi,
                    esi: b,
                    ebp: f,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    for &(a, len) in &regions {
                        std::ptr::write_bytes(a as *mut u8, 0xAA, len);
                    }
                    game_write(LIGHTS_START, 1000u32);
                    ((b + 0x14) as *mut u32).write(time.wrapping_add(1000));
                    ((b + 0x1C) as *mut u32).write(held);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&kept, &saved);
    drop((block, flag));
    t.finish()
}
