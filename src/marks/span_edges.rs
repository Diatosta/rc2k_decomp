//! `marks::span_edges` (0x4BE315): a row of spans at edi for each whole row between the
//! edge pair's two heights.
//!
//! In:  edi where the spans go; the edge pair at `dword_82FD16` ..
//! Out: eax and ebx the last check's row and end, edi past the rows
//!
//! See `marks/spans.rs`. Callers: `marks::span_triangle` (twice). Refuses rows past the
//! work block or more than 0x1000, before writing. Self-test: the original and the Rust
//! version on edge pairs by lot (with the x87 as the game has it, and rounding toward
//! zero as the triangle sets it), the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::spans::*;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::span_edges", rva(0x4BE315), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The control word each test run starts from.
static WORD: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0x37F);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { span_edges(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let edge: [u32; 6] = unsafe { crate::framework::game_read(EDGE_RVA) };
    let rows = edge_rows(&edge).unwrap_or(0);
    let mut regions = vec![(r.edi, 8 * rows as usize)];
    regions.extend(edge_regions());
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

unsafe extern "C" fn original_with_word(r: *mut Regs) {
    x87::set_control_word(WORD.load(Ordering::SeqCst));
    unsafe { call_original(r) };
}

fn rust_with_word(r: &Regs) -> Result<Left, Refused> {
    x87::set_control_word(WORD.load(Ordering::SeqCst));
    rust(r)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let came = x87::control_word();
    let work = TestWork::new();
    let regions = work.regions();
    for word in [came, (came & 0xF3FF) | 0xC00] {
        WORD.store(word, Ordering::SeqCst);
        for seed in 1..=10u32 {
            let c = test_corners(seed, 3);
            let edge = [c[1], c[3], c[0], c[2], c[4], c[5]];
            check_state(
                &mut t,
                &format!("control word {word:#x}, seed {seed}"),
                original_with_word,
                Regs {
                    edi: work.addr(),
                    ..Regs::SENTINEL
                },
                &|| {
                    work.reset();
                    unsafe { game_write(EDGE_RVA, edge) };
                },
                &regions,
                &rust_with_word,
            );
        }
    }
    x87::set_control_word(came);
    work.reset();
    unsafe {
        game_write(
            EDGE_RVA,
            [f32::NAN.to_bits(), 10.0f32.to_bits(), 0, 0, 0, 0],
        )
    };
    t.check(
        unsafe {
            span_edges(&Regs {
                edi: work.addr(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "a start of NaN: the Rust version went on".into(),
    );
    drop(work);
    t.finish()
}
