//! `marks::set_quad_uvs` (0x4CFD6A): writing to the depth buffer off, and the four texture
//! places of the 0x20-byte set at `dword_920738` + 0xC60 put into the four vertices'
//! texture places at `flt_9605B8` (0x20 bytes apart: +0x18 of Direct3D vertices from
//! 0x9605A0, a reading).
//!
//! In:  nothing
//! Out: eax 0xC60, ebx 0xE, ebp and edi the last pair; the rest kept
//!
//! One caller, `sub_4CFB49` (the marks' frame). Self-test: the original and the Rust
//! version recorded on the layer (nothing drawn), the set filled by a pattern; put back
//! afterwards.

use crate::d3d::recorder::record;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::set_quad_uvs", rva(0x4CFD6A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SETS_RVA: usize = rva(0x920738); // dword_920738: sets of four texture places
const SET: u32 = 0xC60;
const VERTICES_RVA: usize = rva(0x9605B8); // flt_9605B8: the first vertex's u, v
const VERTEX: usize = 0x20;

/// The Rust version (see the header).
pub unsafe fn set_quad_uvs(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        crate::scene::depth_test_off::depth_test_off(&mut g);
        g.eax = SET;
        for n in 0..4 {
            let from = SETS_RVA + SET as usize + 8 * n;
            g.ebp = game_read(from);
            g.edi = game_read(from + 4);
            game_write(VERTICES_RVA + VERTEX * n, [g.ebp, g.edi]);
        }
    }
    g
}

fn regions() -> Vec<Region> {
    (0..4)
        .map(|n| (game_addr(VERTICES_RVA + VERTEX * n) as u32, 8))
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { set_quad_uvs(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    let regions = regions();
    let set: Region = (game_addr(SETS_RVA + SET as usize) as u32, 0x20);
    let mut inputs = regions.clone();
    inputs.push(set);
    let saved = capture(&inputs);
    for fill in [0x11u32, 0x3F80_0001] {
        let ((), calls) = record(true, || {
            check_state(
                &mut t,
                &format!("pattern {fill:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&inputs, &saved);
                    for n in 0..8u32 {
                        game_write(
                            SETS_RVA + SET as usize + 4 * n as usize,
                            fill.wrapping_mul(n + 3),
                        );
                    }
                },
                &regions,
                &rust,
            )
        });
        // The original and the Rust version, each one render state.
        t.check(calls.len() == 2 && calls[0] == calls[1], || {
            format!("pattern {fill:#x}: calls {calls:x?}")
        });
    }
    restore(&inputs, &saved);
    t.finish()
}
