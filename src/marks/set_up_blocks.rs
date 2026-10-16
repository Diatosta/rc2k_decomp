//! `marks::set_up_blocks` (0x4D05BD): `init_marks`' step 14, the settings blocks' starting
//! sums measured, each block's points taken and copied, its floats set.
//!
//! In:  nothing
//! Out: as the first block's `marks::copy_block_points` leaves them, ebx its +0x4C, ebp it
//!
//! See `marks/lines.rs`. One caller, `marks::init_marks`. Refuses as
//! `marks::measure_starting_sums`, before anything is taken. Shadow mode runs the original
//! alone (memory cannot be taken twice). Manual patch with a private copy (0x136 bytes; the
//! three calls relocated): the first instruction is a call. Self-test: a redirected copy
//! whose calls go to stand-ins (the block a buffer of the test's, the other two the Rust
//! versions) on scenes of the test's, the game's variables put back afterwards.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::marks::lines::*;
use crate::race::level_init::GIVES;
use crate::race::mud::{copy, run_both};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::set_up_blocks", rva(0x4D05BD), thunk, &ORIGINAL)
    .private_copy(0x136)
    .relocate_calls(&[0, 0x9E, 0xC3])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up_blocks(r, &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call marks::measure_starting_sums
    let entry: &[u8] = &[0xE8, 0xF3, 0x07, 0x00, 0x00];
    let copy = match copy(
        0x4D05BD,
        0x136,
        entry,
        &[0, 0x9E, 0xC3],
        &[(0x4D0DB5, 1), (0x4028A7, 0), (0x4D0A82, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(give);
    let scene = TestScene::new();
    let taken = vec![0xAAu8; (12 * TEST_OBJECT_POINTS) as usize];
    let globals = [
        at(CURRENT_RVA),
        at(ANY_RVA),
        at(TAKEN_RVA),
        (game_addr(DIFFERENCE_RVA) as u32, 12),
    ];
    let saved = capture(&globals);
    let mut regions: Vec<Region> = line_regions();
    regions.extend([at(EVERY_RVA), at(SHARES_RVA), at(TAKEN_RVA)]);
    regions.push((taken.as_ptr() as u32, taken.len()));
    for blocks in [0u32, 1, 3, 8] {
        for seed in [1u32, 2] {
            run_both(
                &mut t,
                &format!("{blocks} blocks, seed {seed}"),
                &copy,
                &regions,
                &|| unsafe {
                    scene.fill(seed * 13 + blocks, blocks);
                    std::ptr::write_bytes(taken.as_ptr() as *mut u8, 0xAA, taken.len());
                    GIVES.store(taken.as_ptr() as u32, Ordering::SeqCst);
                },
                &|r| unsafe { set_up_blocks(r, &mut test_give) },
            );
        }
    }
    restore(&globals, &saved);
    drop(scene);
    t.finish()
}
