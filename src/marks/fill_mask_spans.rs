//! `marks::fill_mask_spans` (0x4BE712): one entry's rows of spans filled into the mask with
//! the shape's number.
//!
//! In:  esi the entry, edi the entries
//! Out: every register kept
//!
//! See `marks/masks.rs`. One caller, `marks::draw_mask_shape` (twice). Refuses a span
//! outside the mask, before writing. Self-test: the original and the Rust version on
//! shapes and a mask of the test's, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::fill_mask_spans", rva(0x4BE712), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { fill_mask_spans(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(
        unsafe { crate::framework::game_read::<u32>(MASK_RVA) },
        MASK_LEN as usize,
    )];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let globals = mask_regions();
    let saved = capture(&globals);
    for seed in 1..=4u32 {
        let shapes = TestShapes::new(3, seed, false);
        for entry in 0..6u32 {
            check_state(
                &mut t,
                &format!("seed {seed}, entry {entry}"),
                call_original,
                Regs {
                    esi: entry,
                    edi: shapes.entries.as_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(shapes.mask.as_ptr() as *mut u8, 0xAA, MASK_LEN as usize);
                    game_write(MASK_RVA, shapes.mask.as_ptr() as u32);
                    game_write(VALUE_RVA, 0x100 + seed * 0x11 + entry);
                },
                &shapes.regions(),
                &rust,
            );
        }
    }
    let shapes = TestShapes::new(1, 9, true);
    unsafe { game_write(MASK_RVA, shapes.mask.as_ptr() as u32) };
    t.check(
        unsafe {
            fill_mask_spans(&Regs {
                esi: 0,
                edi: shapes.entries.as_ptr() as u32,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "a span past the mask: the Rust version went on".into(),
    );
    restore(&globals, &saved);
    t.finish()
}
