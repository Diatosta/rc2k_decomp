//! `marks::map_pixel` (0x4BEFD2): pixel ecx of row `dword_833DB6` placed against the line
//! pair and kept as four bytes at `[ebp + 4 ecx]`.
//!
//! In:  ecx the pixel, ebp the table's row; `dword_833DB6` the row number
//! Out: every register kept
//!
//! See `marks/pixels.rs`. One caller, `marks::map_shape_rows`. Self-test: the original and
//! the Rust version on line pairs, pixels and seeds by lot (both of the shake's branches);
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::pixels::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::map_pixel", rva(0x4BEFD2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { map_pixel(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = pixel_regions();
    regions.push((r.ebp.wrapping_add(r.ecx.wrapping_mul(4)), 4));
    regions.push((crate::framework::game_addr(WORK_RVA) as u32, 8));
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = test_inputs();
    inputs.push((crate::framework::game_addr(PIXEL_RVA) as u32 + 4, 4));
    let mut regions = pixel_regions();
    regions.push((crate::framework::game_addr(WORK_RVA) as u32, 8));
    let saved = (capture(&inputs), capture(&regions));
    let row = vec![0xAAu8; 0x400];
    let rp = row.as_ptr() as u32;
    regions.push((rp, row.len()));
    for seed in 1..=24u32 {
        let x = seed * 37 % 0x100;
        check_state(
            &mut t,
            &format!("seed {seed}, pixel {x:#x}"),
            call_original,
            Regs {
                ecx: x,
                ebp: rp,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                std::ptr::write_bytes(rp as *mut u8, 0xAA, 0x400);
                fill_lines(seed / 2 + 1);
                game_write(PIXEL_RVA + 4, seed * 5 % 0x80);
                game_write(
                    crate::marks::setup::SEED_RVA,
                    seed.wrapping_mul(0x9E37_79B9),
                );
            },
            &regions,
            &rust,
        );
    }
    regions.pop();
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
