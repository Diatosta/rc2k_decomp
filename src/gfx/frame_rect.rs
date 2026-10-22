//! `gfx::frame_rect` (0x4066DF): rectangle ebx's outline, moved by (ecx, edx), in colour eax (see `gfx/rect.rs`).
//!
//! In:  ebx = the rectangle, ecx / edx = the move, eax = the colour, esi = the palette
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `sub_415EA6` and three others. Shadow mode: by state (the surface's pixels it may touch are
//! not compared: the work variables are). Self-test: a test surface at 16 and 32 bits,
//! clips and rectangles moved by (2, 1).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::rect::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::frame_rect", rva(0x4066DF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { frame_rect(r.ebx, r.ecx as i32, r.edx as i32, r.eax, r.esi) };
    Ok(Left::nothing())
}

fn work() -> Vec<Region> {
    [0x4F504Cusize, 0x4F5050, 0x4F5054, 0x4F5058]
        .map(|a| at(rva(a)))
        .to_vec()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &work(), &rust);
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
    let stage = RectStage::new();
    let regions = stage.regions();
    let mut rect = vec![0i32; 4];
    let p = rect.as_mut_ptr() as u32;
    for (bits, clip, x, y, w, h) in CASES {
        check_state(
            &mut t,
            &format!("bits {bits} clip {clip:?} at ({x}, {y}) {w} x {h}"),
            call_original,
            Regs {
                eax: 7,
                ebx: p,
                ecx: 2,
                edx: 1,
                esi: stage.palette.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &|| {
                stage.prepare(bits, clip);
                unsafe { (p as *mut [i32; 4]).write_unaligned([x, y, w, h]) };
            },
            &regions,
            &rust,
        );
    }
    drop(rect);
    stage.end();
    t.finish()
}
