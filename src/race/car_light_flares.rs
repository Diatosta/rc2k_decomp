//! `race::car_light_flares` (0x457036): a car's lights as flares (see `race/frame_draw.rs`;
//! `sub_457241` a pair of lights, `sub_4572B0` one placed in the car's frame,
//! `sub_457332` added to the flare tables and the depth list, `sub_45721A`,
//! `sub_457229`, `sub_41F4E1` covered here: their only callers are in this tree).
//!
//! In:  esi = the car's block
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! With `dword_621870` 8 one flare at `dword_641744` .. in white (0xF0F0F0). Else, with
//! `dword_621480`, for a known kind (+0x24 below 0x22 with `dword_5C5C58`) only below 100.0
//! of +0x310 (its +0x298 to `dword_641754`), and for any other kind always: the kind's
//! light record (`g_lightRecords`, by `dword_5C59B0` of the kind held to 0 .. 0x21) gives
//! up to nine pairs at fixed offsets, each light lit by the block's byte at its place, in
//! three colours (0x606060, 0xC00000, 0x800000). The original's second kind test and its
//! log line ("ERROR - light attempted on car type") are never reached (`notes/bugs.md`).
//! Refuses (before anything) a depth list without room for 24 flares. Shadow mode: by
//! state. Self-test: a test block (frame, place, kind, the lights' bytes), the switches,
//! the flare and depth counts; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

pub fn flare_regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x63F18C, 0x2404),
        at(DEPTH_COUNT_RVA),
        g(0x9D3730, 0x8000),
        at(FACE_KEY_RVA),
        g(0x63501C, 0xC),
        at(rva(0x635038)),
        at(rva(0x635054)),
        at(rva(0x6214FC)),
        at(rva(0x6365C8)),
        at(rva(0x62DAF4)),
        g(0x641750, 0x10),
        at(rva(0x62400C)),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_light_flares(r)? }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

pub fn flare_inputs() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        at(rva(0x621870)),
        at(rva(0x621480)),
        at(rva(0x62DAD0)),
        g(0x641744, 0xC),
        at(rva(0x613FC4)),
        g(0x5EA2CC, 0x40),
    ]
}

/// A test car block: its frame and place, kind, +0x298, +0x310 and the lights' bytes.
pub fn fill_light_block(b: &mut [u32], seed: &mut u32, kind: u32, near: f32) {
    let m = crate::scene::matrix::test_floats(seed);
    b[0xB4 / 4..0xB4 / 4 + 12].copy_from_slice(&m[..12]);
    b[0x134 / 4] = 0x10_0000;
    b[0x138 / 4] = 0x2_0000;
    b[0x13C / 4] = 0x18_0000;
    b[0x24 / 4] = kind;
    b[0x298 / 4] = 0x200 | (*seed & 0x1FF);
    b[0x310 / 4] = near.to_bits();
    for (k, v) in b[0xAC0 / 4..0xAD4 / 4].iter_mut().enumerate() {
        *v = seed.rotate_left(k as u32) & 0x0101_0101;
    }
}
