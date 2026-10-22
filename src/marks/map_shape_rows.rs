//! `marks::map_shape_rows` (0x4BE855): entry esi of the rows at edi mapped pixel by pixel into the table.
//!
//! In:  esi the entry, edi the entries
//! Out: every register kept
//!
//! See `marks/pixels.rs`. Callers: `marks::map_shape` (twice). Refuses rows or spans outside the 0x100 x 0x80
//! table, before writing. Self-test: the original and the Rust version on shapes, line
//! pairs and seeds by lot with a table of the test's, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::masks::TestShapes;
use crate::marks::pixels::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::map_shape_rows", rva(0x4BE855), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { map_shape_rows(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = row_regions();
    regions.push(at(crate::marks::masks::VALUE_RVA));
    let table: u32 = unsafe { crate::framework::game_read(TABLE_RVA) };
    regions.push((table, 0x2_0000));
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
    let mut inputs = test_inputs();
    inputs.push(at(crate::marks::masks::VALUE_RVA));
    let mut regions = row_regions();
    let saved = (capture(&inputs), capture(&regions));
    let table = vec![0xAAu8; 0x2_0000];
    let tp = table.as_ptr() as u32;
    regions.push((tp, table.len()));
    regions.push(at(crate::marks::masks::VALUE_RVA));
    for seed in 1..=3u32 {
        let shapes = TestShapes::new(2, seed, false);
        for e in 0..4u32 {
            check_state(
                &mut t,
                &format!("seed {seed}, entry {e}"),
                call_original,
                Regs {
                    esi: e,
                    edi: shapes.entries.as_ptr() as u32,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::write_bytes(tp as *mut u8, 0xAA, table.len());
                    game_write(TABLE_RVA, tp);
                    fill_lines(seed + e);
                    game_write(crate::marks::setup::SEED_RVA, seed * 0x1234_5679);
                },
                &regions,
                &rust,
            );
        }
    }
    let shapes = TestShapes::new(1, 9, true);
    let e = 0u32;
    t.check(
        unsafe {
            map_shape_rows(&Regs {
                esi: e,
                edi: shapes.entries.as_ptr() as u32,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "rows past the table: the Rust version went on".into(),
    );
    regions.pop();
    regions.pop();
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
