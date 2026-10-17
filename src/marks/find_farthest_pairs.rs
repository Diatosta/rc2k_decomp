//! `marks::find_farthest_pairs` (0x4BDD1D): of each points list, the two points of one face
//! farthest apart.
//!
//! In:  nothing (points lists A and B, `g_marksTurnedA` / `B`)
//! Out: as the last pair looked at leaves them (edx the farthest of B)
//!
//! See `marks/shape_points.rs`. One caller, `sub_4BD1A1`. Refuses a list of more than 0x40.
//! Self-test: the original and the Rust version on lists and points by lot (0 .. 0x40, a
//! few faces), the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::marks::masks::{MOST_POINTS, POINTS_A, POINTS_B};
use crate::marks::shape_points::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "marks::find_farthest_pairs",
    rva(0x4BDD1D),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { find_farthest_pairs(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &farthest_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs: Vec<Region> = POINTS_A.regions();
    inputs.extend(POINTS_B.regions());
    inputs.push((game_addr(TURNED_RVA[0]) as u32, 0x600));
    let regions = farthest_regions();
    let saved = (capture(&inputs), capture(&regions));
    for (a, b) in [
        (0u32, 0u32),
        (1, 2),
        (5, 9),
        (0x20, 0x11),
        (MOST_POINTS, MOST_POINTS),
    ] {
        for seed in [1u32, 2] {
            check_state(
                &mut t,
                &format!("{a:#x} and {b:#x} points, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    let mut s = seed * 31 + a + b;
                    let mut next = move || {
                        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                        s >> 8
                    };
                    for (list, n) in [(&POINTS_A, a), (&POINTS_B, b)] {
                        game_write(list.count, n);
                        let faces: [u32; MOST_POINTS as usize] =
                            std::array::from_fn(|_| 0x1000 + next() % 4);
                        game_write(list.faces, faces);
                    }
                    let points: [u32; 0x180] =
                        std::array::from_fn(|_| (next() % 0x10_0000).wrapping_sub(0x8_0000));
                    game_write(TURNED_RVA[0], points);
                    game_write(FARTHEST_RVA[0], [0xAAAA_AAAAu32; 4]);
                },
                &regions,
                &rust,
            );
        }
    }
    unsafe { game_write(POINTS_A.count, MOST_POINTS + 1) };
    t.check(
        unsafe { find_farthest_pairs(&Regs::SENTINEL) }.is_err(),
        || "0x41 points: the Rust version went on".into(),
    );
    restore(&inputs, &saved.0);
    restore(&regions, &saved.1);
    t.finish()
}
