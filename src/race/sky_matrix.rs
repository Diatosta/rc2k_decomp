//! `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky (a reading: object 1,
//! drawn by `sub_455540`): turned about y by -(`dword_609334` << 8), raised by
//! `dword_609318` << 8, scaled by (`dword_6241D0` << 4) / 16, times the view's turn (see
//! `race/frame_draw.rs`, with `scene::matrix`'s Rust versions).
//!
//! In:  nothing
//! Out: eax `dword_6241D0`, ebx its x 16, ecx 0, edx 0, esi `g_turnedMatrix`, edi
//!      `g_viewTurnMatrix`, ebp `g_madeObjectMatrix` (a `lea` follows)
//!
//! Caller: `sub_455540` (0x455569). Shadow mode: by state (the matrices it builds, the
//! place and size dwords, `dword_62400C`). Self-test: the angle, height, size and view turn
//! set by pattern; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::scene::matrix::{self, matrix as whole};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::sky_matrix", rva(0x42725E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn sky_regions() -> Vec<Region> {
    let mut v: Vec<Region> = [
        matrix::TURN_Y_RVA,
        matrix::PLACED_RVA,
        matrix::TURNED_RVA,
        matrix::OBJECT_RVA,
    ]
    .map(whole)
    .to_vec();
    v.extend([
        (game_addr(matrix::TURN_Y_WORK_RVA) as u32, 8),
        (game_addr(rva(0x63501C)) as u32, 0xC),
        at(rva(0x635038)),
        at(rva(0x62400C)),
    ]);
    v
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { sky_matrix() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &sky_regions(), &rust);
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
    let regions = sky_regions();
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x609334)),
        at(rva(0x609318)),
        at(rva(0x6241D0)),
        whole(matrix::VIEW_TURN_RVA),
    ]);
    let saved = capture(&kept);
    let mut seed = 0x51C0_0001u32;
    for (angle, height, size) in [
        (0u32, 0u32, 0u32),
        (0x40, 0x10, 0x100),
        (0xFF, 0x7FFF, 0x12345),
        (0x1234, 0xFFFF_FF00, 7),
    ] {
        let view = matrix::test_floats(&mut seed);
        check_state(
            &mut t,
            &format!("angle {angle:#x} height {height:#x} size {size:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x609334), angle);
                game_write(rva(0x609318), height);
                game_write(rva(0x6241D0), size);
                game_write(matrix::VIEW_TURN_RVA, view);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
