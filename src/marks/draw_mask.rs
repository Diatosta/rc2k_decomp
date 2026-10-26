//! `marks::draw_mask` (0x4BE6C2): mask esi made 0xFF and each of ebp's shapes drawn into it
//! with its number.
//!
//! In:  esi the mask (0x8000 bytes), ebp the shapes
//! Out: eax the count, ecx 0, esi and edi as the last shape leaves them
//!
//! See `marks/masks.rs`. One caller, `sub_4BDE9A` (the two masks). Refuses a count of 0 or
//! less (the original's loop would run round) or a span outside the mask, before writing.
//! Self-test: the original and the Rust version on shapes and a mask of the test's (1, 2,
//! 5 shapes), the refusals; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_mask", rva(0x4BE6C2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { draw_mask(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions: Vec<Region> = mask_regions();
    regions.push((r.esi, MASK_LEN as usize));
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
    let globals = mask_regions();
    let saved = capture(&globals);
    for count in [1u32, 2, 5] {
        for seed in 1..=2u32 {
            let shapes = TestShapes::new(count, seed * 3 + count, false);
            let mut regions = mask_regions();
            regions.extend(shapes.regions());
            check_state(
                &mut t,
                &format!("{count} shapes, seed {seed}"),
                call_original,
                Regs {
                    esi: shapes.mask.as_ptr() as u32,
                    ebp: shapes.shapes(),
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(shapes.mask.as_ptr() as *mut u8, 0xAA, MASK_LEN as usize)
                },
                &regions,
                &rust,
            );
        }
    }
    for (what, shapes) in [
        ("no shapes", TestShapes::new(0, 1, false)),
        ("a span past the mask", TestShapes::new(1, 9, true)),
    ] {
        t.check(
            unsafe {
                draw_mask(&Regs {
                    esi: shapes.mask.as_ptr() as u32,
                    ebp: shapes.shapes(),
                    ..Regs::SENTINEL
                })
            }
            .is_err(),
            || format!("{what}: the Rust version went on"),
        );
    }
    restore(&globals, &saved);
    t.finish()
}
