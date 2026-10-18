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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_light_flares", rva(0x457036), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &flare_regions(), &rust);
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

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = flare_regions();
    let mut kept = regions.clone();
    kept.extend(flare_inputs());
    let saved = capture(&kept);
    let mut block = vec![0u32; 0x400];
    let mut seed = 0xF1A2_E001u32;
    for (mode, hiding, kind, near, flares) in [
        (8u32, 1u32, 1u32, 0.0f32, 0u32),
        (0, 0, 1, 0.0, 0),
        (0, 1, 1, 50.0, 0),
        (0, 1, 1, 150.0, 0),
        (0, 1, 0xD, 10.0, 0x10),
        (0, 1, 0xE, 10.0, 0x3FF),
        (0, 1, 0x40, 500.0, 0x400),
    ] {
        fill_light_block(&mut block, &mut seed, kind, near);
        let b = block.as_ptr() as u32;
        check_state(
            &mut t,
            &format!("mode {mode} hiding {hiding} kind {kind:#x} near {near} flares {flares:#x}"),
            call_original,
            Regs {
                esi: b,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x621870), mode);
                game_write(rva(0x621480), hiding);
                game_write(rva(0x62DAD0), 1u32);
                game_write(rva(0x641744), 0x100u32);
                game_write(rva(0x641748), 0x200u32);
                game_write(rva(0x64174C), 0x300u32);
                game_write(FLARE_COUNT_RVA, flares);
                game_write(DEPTH_COUNT_RVA, 0x10u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(block);
    t.finish()
}
