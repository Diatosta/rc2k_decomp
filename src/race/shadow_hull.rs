//! `race::shadow_hull` (0x45836F): the shadow's points (in angle order) reduced to a
//! convex hull (see `race/car_shadow.rs`): the first copied after the last, each point
//! that does not turn left (64-bit cross products of whole numbers, `qword_620460`)
//! taken out, starting again, until none; `dword_643ACC` the count.
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `cmp` follows, not of them)
//!
//! Refuses (before anything) fewer than two points or more than 0x40. Shadow mode: by
//! state. Self-test: point records of the test's (squares, stars, lines); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::car_shadow::*;
use crate::race::frame_draw::all_regs;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::shadow_hull", rva(0x45836F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { shadow_hull(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn regions() -> Vec<Region> {
    vec![
        (game_addr(RECORDS_RVA) as u32, 0x880),
        at(HULL_COUNT_RVA),
        (game_addr(PRODUCT_RVA) as u32, 8),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(COUNT_RVA));
    let saved = capture(&kept);
    let base = game_addr(RECORDS_RVA) as u32;
    let shapes: [&[(i32, i32)]; 4] = [
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        &[
            (0, 0),
            (50, 10),
            (100, 0),
            (90, 50),
            (100, 100),
            (50, 90),
            (0, 100),
            (10, 50),
        ],
        &[(0, 0), (10, 10), (20, 20), (30, 30)],
        &[
            (0, 0),
            (0x10_0000, -0x8_0000),
            (0x20_0000, 0x10_0000),
            (0x10_0000, 0x30_0000),
            (-0x8_0000, 0x10_0000),
        ],
    ];
    for (n, shape) in shapes.iter().enumerate() {
        check_state(
            &mut t,
            &format!("shape {n}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(COUNT_RVA, shape.len() as u32);
                for (k, &(x, z)) in shape.iter().enumerate() {
                    let r = base + 0x20 * k as u32;
                    for (o, v) in [
                        (0u32, x as u32),
                        (4, 9),
                        (8, z as u32),
                        (0xC, k as u32),
                        (0x10, 0),
                        (0x14, 0),
                        (0x18, 0),
                        (0x1C, 0),
                    ] {
                        ((r + o) as *mut u32).write_unaligned(v);
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
