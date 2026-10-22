//! `marks::map_shapes` (0x4BE752): the pixel table at esi cleared and every shape at ebp
//! mapped into it from the corner table edi, rounding toward zero.
//!
//! In:  ebp the shapes, esi the table (0x8000 dwords), edi the corner table
//! Out: eax the count, ecx 0, esi and edi as the last shape leaves them
//!
//! See `marks/pixels.rs`. Callers: `marks::make_masks` (the two shapes). Refuses a count of
//! 0 or less or past 0x80, and rows or spans outside the table, before writing.
//! Self-test: the original and the Rust version on shapes and corner tables by lot with a
//! table of the test's, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::masks::TestShapes;
use crate::marks::pixels::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::map_shapes", rva(0x4BE752), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { map_shapes(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = shapes_regions();
    regions.push((r.esi, 0x2_0000));
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
    let mut regions = shapes_regions();
    let saved = capture(&regions);
    let table = vec![0xAAu8; 0x2_0000];
    let tp = table.as_ptr() as u32;
    let mut corners = [0u32; 6 * 4];
    let cp = corners.as_mut_ptr() as u32;
    regions.push((tp, table.len()));
    for (count, seed) in [(1u32, 1u32), (2, 2), (3, 3)] {
        let shapes = TestShapes::new(count, seed, false);
        check_state(
            &mut t,
            &format!("{count} shapes, seed {seed}"),
            call_original,
            Regs {
                ebp: shapes.shapes(),
                esi: tp,
                edi: cp,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                std::ptr::write_bytes(tp as *mut u8, 0xAA, table.len());
                let mut s = seed;
                for k in 0..24u32 {
                    s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    ((cp + 4 * k) as *mut u32)
                        .write_unaligned((((s >> 8) % 0x8000) as f32 / 128.0).to_bits());
                }
                game_write(crate::marks::setup::SEED_RVA, seed * 0x2468_ACE1);
            },
            &regions,
            &rust,
        );
    }
    let shapes = TestShapes::new(0, 1, false);
    t.check(
        unsafe {
            map_shapes(&Regs {
                ebp: shapes.shapes(),
                esi: tp,
                edi: cp,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "no shapes: the Rust version went on".into(),
    );
    regions.pop();
    restore(&regions, &saved);
    t.finish()
}
