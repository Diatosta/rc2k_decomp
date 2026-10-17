//! `marks::make_corners` (0x4BEB25): the two corner tables made from shape set 0 turned at
//! 0x81 eased steps.
//!
//! In:  nothing
//! Out: eax 0x102, esi the kind's box, the rest as the last corner leaves them
//!
//! See `marks/corners.rs`. One caller, `marks::make_masks`. Refuses a kind of 0x22 or
//! more, a farthest pair past 0x3F, what `marks::turn_shape_points` refuses at any step,
//! before writing. Self-test: the original and the Rust version on sets, pairs and steps
//! by lot, the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::corners::*;
use crate::marks::shape_points::{FARTHEST_RVA, MOST_POINTS, fill_set, set_inputs, target_regions};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::make_corners", rva(0x4BEB25), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { make_corners(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = corner_regions();
    regions.extend(unsafe { target_regions() });
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The steps and the block's lowest points the test sets.
const STEPS_IN: [usize; 6] = [
    rva(0x89C9CC),
    rva(0x89C9D4),
    rva(0x884FC4),
    rva(0x89C9DC),
    rva(0x89C9E4),
    rva(0x884FC8),
];
const OFFSETS: usize = rva(0x8859F4);

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs: Vec<Region> = set_inputs();
    inputs.extend(STEPS_IN.iter().map(|&r| at(r)));
    inputs.extend(FARTHEST_RVA.iter().map(|&r| (game_addr(r) as u32, 8)));
    inputs.extend([
        (game_addr(OFFSETS) as u32, 0x18),
        at(crate::marks::kind_values::KIND_RVA),
    ]);
    let mut regions = corner_regions();
    let saved = (capture(&inputs), capture(&regions));
    let targets = vec![0xAAu8; 12 * 2 * MOST_POINTS as usize];
    let tp = targets.as_ptr() as u32;
    regions.push((tp, targets.len()));
    for seed in 1..=4u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                std::ptr::write_bytes(tp as *mut u8, 0xAA, targets.len());
                let counts = [4 + seed, 3 + seed];
                fill_set(seed, counts, tp);
                let mut v = [0u32; 6];
                crate::scene::matrix3::test_floats(seed + 90, &mut v);
                for (r, x) in STEPS_IN.iter().zip(v) {
                    game_write(*r, x);
                }
                game_write(FARTHEST_RVA[0], [seed % counts[0], (seed + 2) % counts[0]]);
                game_write(FARTHEST_RVA[1], [(seed + 1) % counts[1], 0u32]);
                let o: [u32; 6] =
                    std::array::from_fn(|k| (seed * 0x1357 * (k as u32 + 1)) % 0x4000);
                game_write(OFFSETS, o);
                game_write(crate::marks::kind_values::KIND_RVA, seed * 5 % 0x22);
            },
            &regions,
            &rust,
        );
    }
    unsafe { game_write(FARTHEST_RVA[0], [0x40u32, 0]) };
    t.check(unsafe { make_corners(&Regs::SENTINEL) }.is_err(), || {
        "a farthest pair of 0x40: the Rust version went on".into()
    });
    regions.pop();
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
