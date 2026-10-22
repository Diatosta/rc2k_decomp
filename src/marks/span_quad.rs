//! `marks::span_quad` (0x4BE0C6): the quad at ebp (four x, y floats) spanned as two triangles.
//!
//! In:  ebp the corners
//! Out: every register kept
//!
//! See `marks/spans.rs`. Callers: `sub_4BE592`. Refuses runs past their table, and spans past the
//! work block or away, before writing. Self-test: the original and the Rust version on
//! corners by lot (two at one height now and then) in a work block of the test's, the
//! refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::spans::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::span_quad", rva(0x4BE0C6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { span_quad(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = triangle_regions();
    let cursor: u32 = unsafe { crate::framework::game_read(crate::marks::masks::CURSOR_RVA) };
    regions.push((cursor, 0x1000));
    let runs: u32 = unsafe { crate::framework::game_read(crate::marks::masks::RUNS_RVA) };
    regions.push((runs, 0x20));
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
    let work = TestWork::new();
    let regions = work.regions();
    let mut corners = [0u32; 8];
    let p = corners.as_mut_ptr() as u32;
    for seed in 1..=12u32 {
        let c = test_corners(seed, 4);
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                ebp: p,
                ..Regs::SENTINEL
            },
            &|| {
                work.reset();
                unsafe { std::ptr::copy(c.as_ptr(), p as *mut u32, c.len()) };
            },
            &regions,
            &rust,
        );
    }
    work.reset();
    unsafe { (p as *mut [u32; 2]).write([0, f32::NAN.to_bits()]) };
    t.check(
        unsafe {
            span_quad(&Regs {
                ebp: p,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "a corner at NaN: the Rust version went on".into(),
    );
    drop(work);
    t.finish()
}
