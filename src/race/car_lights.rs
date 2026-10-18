//! `race::car_lights` (0x455CFF): with `dword_621480` set, each race car's lights when they
//! are on (+0x6FC bit 5): its depth entry (`race::add_car_depth_entry`), then its flares
//! (`race::car_light_flares`), inside `pusha` (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: ebx the race cars; the rest kept (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x455858), each view drawn. Refuses (before anything) no cars or
//! more than eight, a null block, a depth list without room for all the cars' entries.
//! Shadow mode: by state. Self-test: the game's blocks given lights and frames by
//! pattern; all put back (skipped with a null block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::car_light_flares::{fill_light_block, flare_inputs, flare_regions};
use crate::race::frame_draw::*;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_lights", rva(0x455CFF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut v = flare_regions();
    v.extend([(crate::framework::game_addr(rva(0x63502C)) as u32, 0xC)]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_lights(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = (0..8)
        .map(|n| unsafe { game_read::<u32>(CAR_BLOCKS_RVA + 4 * n) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("a car block is null (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend(flare_inputs());
    kept.push(at(CARS_RVA));
    kept.extend(blocks.iter().map(|&b| (b, 0xFE0)));
    let saved = capture(&kept);
    let mut seed = 0xCA21_0001u32;
    for (cars, hiding) in [(8u32, 1u32), (3, 1), (8, 0)] {
        let s = seed;
        seed = seed.wrapping_mul(0x9E37_79B9).wrapping_add(1);
        check_state(
            &mut t,
            &format!("cars {cars} hiding {hiding}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                let mut s2 = s;
                for (n, &b) in blocks.iter().enumerate() {
                    let block = std::slice::from_raw_parts_mut(b as *mut u32, 0x3F8);
                    fill_light_block(block, &mut s2, (n as u32 * 5) % 0x24, 40.0 * n as f32);
                    block[0x6FC / 4] = if n % 2 == 0 { 0x21 } else { 0x11 };
                }
                game_write(CARS_RVA, cars);
                game_write(rva(0x621480), hiding);
                game_write(rva(0x621870), 0u32);
                game_write(rva(0x62DAD0), 1u32);
                game_write(FLARE_COUNT_RVA, 0u32);
                game_write(DEPTH_COUNT_RVA, 0u32);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
