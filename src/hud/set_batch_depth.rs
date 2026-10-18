//! `hud::set_batch_depth` (0x4B726C): the four vertices of `g_polygonVertices` given the
//! HUD's rhw (`flt_75BFEC`), depth (`flt_75BFE8`) and specular 0.
//!
//! In:  nothing
//! Out: all kept
//!
//! Caller: `sub_4B1CD0` (the HUD's start). Self-test: the original and the Rust version on
//! the vertices filled with 0xAA, values by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::hud::batch::POLYGON_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::set_batch_depth", rva(0x4B726C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DEPTH_RVA: usize = rva(0x75BFE8); // flt_75BFE8
const RHW_RVA: usize = rva(0x75BFEC); // flt_75BFEC

/// The Rust version (see the header).
pub unsafe fn set_batch_depth() {
    unsafe {
        let rhw = x87::loaded(game_read(RHW_RVA));
        let depth = x87::loaded(game_read(DEPTH_RVA));
        for n in 0..4 {
            let v = POLYGON_RVA + 0x20 * n;
            game_write(v + 0xC, rhw);
            game_write(v + 0x14, 0u32);
            game_write(v + 8, depth);
        }
    }
}

fn regions() -> Vec<Region> {
    vec![(game_addr(POLYGON_RVA) as u32, 0x80)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { set_batch_depth() };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([DEPTH_RVA, RHW_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..3u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                std::ptr::write_bytes(game_addr(POLYGON_RVA) as *mut u8, 0xAA, 0x80);
                game_write(DEPTH_RVA, (0.25f32 * case as f32).to_bits());
                game_write(RHW_RVA, (1.0f32 + case as f32).to_bits());
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
