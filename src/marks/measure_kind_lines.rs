//! `marks::measure_kind_lines` (0x4D1225): faces esi and edi, block ebp: the lines of the
//! block's 0x11 kinds (+0x130, sums +0x1B8, counts +0x240).
//!
//! In:  esi, edi the two faces, ebp the block
//! Out: eax 0x11, ebx and ecx the last differences
//!
//! See `marks/lines.rs`. One caller, `marks::measure_lines`. Self-test: the original and
//! the Rust version on a scene of the test's (faces, points, blocks by lot), put back
//! afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::measure_kind_lines", rva(0x4D1225), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions(r: &Regs) -> Vec<Region> {
    vec![
        (r.ebp, BLOCK_LEN as usize),
        (game_addr(DIFFERENCE_RVA) as u32, 12),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { measure_kind_lines(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let differences = [(game_addr(DIFFERENCE_RVA) as u32, 12)];
    let saved = capture(&differences);
    for seed in 1..=4u32 {
        for face in [0, 3, 0x11] {
            let r = Regs {
                esi: scene.face(face),
                edi: scene.face(TEST_FACES + face),
                ebp: scene.block(seed % MOST_BLOCKS),
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("seed {seed}, face {face:#x}"),
                call_original,
                r,
                &|| scene.fill(seed, 8),
                &regions(&r),
                &rust,
            );
        }
    }
    restore(&differences, &saved);
    drop(scene);
    t.finish()
}
