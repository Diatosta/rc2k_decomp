//! `race::add_car_depth_entry` (0x4574A0): a car's place (block esi +0x134 ..) through
//! `g_viewMatrix` into a depth entry (the key from its depth, the second word ebx +
//! 0x2000000), for the car's lights (see `race/frame_draw.rs`).
//!
//! In:  esi = the car's block, ebx = the car
//! Out: eax the place's z, ebx + 0x2000000, ecx the key, edx the entry, esi
//!      `g_viewMatrix` (its caller calls it inside `pusha`)
//!
//! Caller: `sub_455CFF` (0x455D22). Refuses (before anything) a depth entry past the
//! 0x1000. Shadow mode: by state. Self-test: a test block's places, the view matrix and
//! counts set; all put back; 0x1000 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::add_car_depth_entry", rva(0x4574A0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x63501C, 0xC),
        g(0x63502C, 0xC),
        at(FACE_KEY_RVA),
        at(DEPTH_COUNT_RVA),
        g(0x9D3730, 0x8000),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_car_depth_entry(r.esi, r.ebx) }
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
    let regions = regions();
    let mut kept = regions.clone();
    kept.push((game_addr(rva(0x5EA2CC)) as u32, 0x40));
    let saved = capture(&kept);
    let mut block = vec![0u32; 0x50];
    let b = block.as_mut_ptr() as u32;
    let mut seed = 0xCA2D_0001u32;
    for (count, car) in [(0u32, 0u32), (0x10, 3), (0xFFF, 7)] {
        let m = crate::scene::matrix::test_floats(&mut seed);
        let place = [
            seed % 0x40_0000,
            seed % 0x1_0000,
            seed.rotate_left(7) % 0x40_0000,
        ];
        check_state(
            &mut t,
            &format!("count {count:#x} car {car} place {place:x?}"),
            call_original,
            Regs {
                esi: b,
                ebx: car,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5EA2CC), m);
                game_write(DEPTH_COUNT_RVA, count);
                for (k, &p) in place.iter().enumerate() {
                    ((b + 0x134 + 4 * k as u32) as *mut u32).write_unaligned(p);
                }
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(DEPTH_COUNT_RVA, 0x1000u32);
        add_car_depth_entry(b, 0).is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "entry 0x1000 not refused".into());
    drop(block);
    t.finish()
}
