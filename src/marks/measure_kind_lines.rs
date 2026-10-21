//! `marks::measure_kind_lines` (0x4D1225): faces esi and edi, block ebp: the lines of the
//! block's 0x11 kinds (+0x130, sums +0x1B8, counts +0x240).
//!
//! In:  esi, edi the two faces, ebp the block
//! Out: eax 0x11, ebx and ecx the last differences
//!
//! See `marks/lines.rs`. One caller, `marks::measure_lines`. Self-test: the original and
//! the Rust version on a scene of the test's (faces, points, blocks by lot), put back
//! afterwards; at the test precisions (the sums within one at extended).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{
    Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
};
use crate::service::harness::{Region, capture, restore};
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
    Ok(left_of(&unsafe { measure_kind_lines_plain(r) }))
}

/// The sums made from lengths (`fistp`): one off at a rounding edge at extended precision.
fn sums(r: &Regs) -> [(Region, Tolerance); 1] {
    [((r.ebp + 0x1b8, 4 * 0x11), ROUNDED)]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    let places = sums(r);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
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
            let places = sums(&r);
            let tolerant = Floats {
                memory: &places,
                regs: &[],
            };
            at_test_precisions(&tolerant, &mut |word, held| {
                check_state_floats(
                    &mut t,
                    "marks::measure_kind_lines",
                    &format!("control word {word:#x}, seed {seed}, face {face:#x}"),
                    call_original,
                    r,
                    &|| scene.fill(seed, 8),
                    &regions(&r),
                    held,
                    &rust,
                );
            });
        }
    }
    restore(&differences, &saved);
    drop(scene);
    t.finish()
}
