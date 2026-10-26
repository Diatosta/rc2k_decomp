//! `race::draw_distance_settings` (0x43932A): the draw distance's level (`dword_60AC84`,
//! held to 0 ..= 4 by sign) picks the values of six tables: `dword_60A87C` (the grid's
//! reach, at least 1 by sign), `dword_60930C`, `dword_609358`, `dword_609308`,
//! `dword_609320`, `dword_60AC80`.
//!
//! In:  nothing
//! Out: ebx the level, eax the last value (a `lea` follows)
//!
//! See `race/frame_draw.rs`. Caller: `race::fog_table`. Shadow mode: by state. Self-test:
//! the game's tables, levels -1 .. 5 and 0x80000000; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::draw_distance_settings",
    rva(0x43932A),
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

pub fn distance_regions() -> Vec<Region> {
    [
        0x60A87Cusize,
        0x60930C,
        0x609358,
        0x609308,
        0x609320,
        0x60AC80,
    ]
    .iter()
    .map(|&a| at(rva(a)))
    .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { draw_distance_settings() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &distance_regions(), &rust);
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
    let regions = distance_regions();
    let mut kept = regions.clone();
    kept.push(at(rva(0x60AC84)));
    let saved = capture(&kept);
    for level in [u32::MAX, 0, 1, 2, 3, 4, 5, 0x8000_0000] {
        check_state(
            &mut t,
            &format!("level {level:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x60AC84), level);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
