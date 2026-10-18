//! `race::car_part_matrix_4` (0x461FE4): part 4's matrix (`flt_6418A0`, the car's +0x800) times
//! the body's turn, the car's turn and `g_viewMatrix` into `g_madeObjectMatrix` (see
//! `race/car_draw.rs`).
//!
//! In:  nothing
//! Out: esi, edi, ebp the last product's matrices, the rest as `scene::multiply_matrices`
//!      leaves them (a `call` follows)
//!
//! Caller: `sub_4616AA`. Shadow mode: by state. Self-test: the game's matrices and two
//! sets of test floats in them; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::car_draw::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_part_matrix_4", rva(0x461FE4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut out = *r;
    unsafe { car_part_matrix(&mut out, PART_MATRICES[3]) };
    Ok(all_regs(&out))
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![g(0x5EA10C, 0x40), g(0x5EA14C, 0x40), g(0x5EA30C, 0x40)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        g(0x6418A0, 0x40),
        g(0x5EA08C, 0x40),
        g(0x641760, 0x40),
        g(0x5EA2CC, 0x40),
    ]);
    let saved = capture(&kept);
    for seed in [0u32, 1, 2] {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                if seed != 0 {
                    for &(a, len) in &kept[3..] {
                        for k in 0..len as u32 / 4 {
                            let v = (((k * 7 + seed * 3) % 17) as f32 - 8.0) * 0.37 * seed as f32;
                            ((a + 4 * k) as *mut u32).write_unaligned(v.to_bits());
                        }
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
