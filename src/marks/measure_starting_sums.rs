//! `marks::measure_starting_sums` (0x4D0DB5): in step 14, every block measured and its sums
//! kept as the ones the shares are made of.
//!
//! In:  nothing
//! Out: eax the first block's last sum moved, ebx 0, ebp that block (the settings with
//!      none); the rest as `marks::measure_lines` leaves them
//!
//! See `marks/lines.rs`. One caller, `marks::set_up_blocks`. Refuses as `measure_lines`
//! with every block. Self-test: the original and the Rust version on scenes of the test's
//! (0, 1, 3 and 8 blocks), the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::measure_starting_sums",
    rva(0x4D0DB5),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut r = line_regions();
    r.extend([at(EVERY_RVA), at(SHARES_RVA)]);
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { measure_starting_sums(r) }?))
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
    let scene = TestScene::new();
    let globals = [
        at(CURRENT_RVA),
        at(ANY_RVA),
        (game_addr(DIFFERENCE_RVA) as u32, 12),
    ];
    let saved = capture(&globals);
    let regions = regions();
    for blocks in [0u32, 1, 3, 8] {
        for seed in [1u32, 2] {
            check_state(
                &mut t,
                &format!("{blocks} blocks, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| scene.fill(seed * 11 + blocks, blocks),
                &regions,
                &rust,
            );
        }
    }
    scene.fill(5, 9);
    t.check(
        unsafe { measure_starting_sums(&Regs::SENTINEL) }.is_err(),
        || "9 blocks: the Rust version went on".into(),
    );
    restore(&globals, &saved);
    drop(scene);
    t.finish()
}
