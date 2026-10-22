//! `marks::draw_mask_shape` (0x4BE6FB): shape esi's two entries filled into the mask with
//! its number.
//!
//! In:  esi the shape, ebp the shapes
//! Out: esi 2 esi + 1, edi the shapes' entries
//!
//! See `marks/masks.rs`. One caller, `marks::draw_mask`. Refuses a span outside the mask,
//! before writing. Self-test: the original and the Rust version on shapes and a mask of
//! the test's, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::masks::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::draw_mask_shape", rva(0x4BE6FB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { draw_mask_shape(r) }?))
}

fn regions() -> Vec<Region> {
    let mut r = mask_regions();
    r.push((unsafe { game_read::<u32>(MASK_RVA) }, MASK_LEN as usize));
    r
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
    let globals = mask_regions();
    let saved = capture(&globals);
    for seed in 1..=4u32 {
        let shapes = TestShapes::new(3, seed, false);
        unsafe { game_write(MASK_RVA, shapes.mask.as_ptr() as u32) };
        let mut regions = mask_regions();
        regions.extend(shapes.regions());
        for shape in 0..3u32 {
            check_state(
                &mut t,
                &format!("seed {seed}, shape {shape}"),
                call_original,
                Regs {
                    esi: shape,
                    ebp: shapes.shapes(),
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(shapes.mask.as_ptr() as *mut u8, 0xAA, MASK_LEN as usize);
                    game_write(MASK_RVA, shapes.mask.as_ptr() as u32);
                },
                &regions,
                &rust,
            );
        }
    }
    let shapes = TestShapes::new(1, 9, true);
    unsafe { game_write(MASK_RVA, shapes.mask.as_ptr() as u32) };
    t.check(
        unsafe {
            draw_mask_shape(&Regs {
                esi: 0,
                ebp: shapes.shapes(),
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "a span past the mask: the Rust version went on".into(),
    );
    restore(&globals, &saved);
    t.finish()
}
