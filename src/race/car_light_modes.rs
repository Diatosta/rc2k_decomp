//! `race::car_light_modes` (0x455B70): each race car's lights mode in +0x6FC (see
//! `race/frame_draw.rs`): `dword_63DBA4` the drawn view's car; all eight blocks' +0x6FC 0;
//! the drawn cars with lights get their nibbles by the shown car's state, `dword_606274`,
//! `dword_606270` and `dword_6238B4`.
//!
//! In:  nothing
//! Out: ebx the race cars (8 with none), esi the last block (a `call` follows)
//!
//! Caller: `sub_4557AB` (0x4557D6), each view drawn. Refuses (before anything) a null
//! block among the eight or more than eight cars. Shadow mode: by state (+0x6FC of the
//! eight, `dword_63DBA4`). Self-test: the game's blocks with lights, +0x69C/+0x684 and the
//! switches set by pattern, modes 0, 1, 2, both views; all put back (skipped with a null
//! block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_light_modes", rva(0x455B70), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn blocks() -> Vec<u32> {
    (0..8)
        .map(|n| unsafe { game_read::<u32>(CAR_BLOCKS_RVA + 4 * n) })
        .collect()
}

fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = blocks()
        .into_iter()
        .filter(|&b| b != 0)
        .map(|b| (b + 0x6FC, 4))
        .collect();
    v.push(at(SHOWN_CAR_RVA));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { car_light_modes(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks = blocks();
    if blocks.contains(&0) {
        return t.skip("a car block is null (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    for a in [
        0x62389Cusize,
        0x63993C,
        0x606274,
        0x606270,
        0x6238B4,
        0x63DB9C,
        0x63DBA8,
        0x63DBAC,
    ] {
        kept.push(at(rva(a)));
    }
    kept.push(at(CARS_RVA));
    for &b in &blocks {
        kept.extend([(b + 0x25C, 4), (b + 0x684, 4), (b + 0x69C, 4)]);
    }
    let saved = capture(&kept);
    for (case, (mode, cars, view, switches)) in [
        (0u32, 8u32, 0u32, [0u32, 0, 0, 0]),
        (1, 8, 1, [0, 1, 1, 1]),
        (2, 5, 0, [0, 1, 2, 2]),
        (1, 3, 0, [1, 1, 4, 0]),
        (0, 0, 0, [0, 0, 0, 0]),
        (3, 8, 1, [0, 1, 3, 1]),
    ]
    .into_iter()
    .enumerate()
    {
        check_state(
            &mut t,
            &format!("mode {mode} cars {cars} view {view} switches {switches:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                game_write(rva(0x62389C), mode);
                game_write(CARS_RVA, cars);
                game_write(rva(0x63DB9C), view);
                game_write(rva(0x63DBA8), 2u32);
                game_write(rva(0x63DBAC), 5u32);
                game_write(rva(0x63993C), switches[0]);
                game_write(rva(0x606274), switches[1]);
                game_write(rva(0x606270), switches[2]);
                game_write(rva(0x6238B4), switches[3]);
                for (n, &b) in blocks.iter().enumerate() {
                    let bits = (n as u32).wrapping_mul(0x9E37) >> (case % 3);
                    w(b + 0x25C, bits & 1);
                    w(b + 0x684, bits >> 1 & 1);
                    w(b + 0x69C, bits >> 2 & 1);
                    w(b + 0x6FC, 0x5A5A);
                }
                // The shown cars always have lights here.
                w(blocks[2] + 0x25C, 1);
                w(blocks[5] + 0x25C, 1);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
