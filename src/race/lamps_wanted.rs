//! `race::lamps_wanted` (0x4536D7): `dword_63DB58` = `g_lampCount`, with bit 0 set when a
//! race car has +0x25C and +0x84 both set.
//!
//! In:  nothing
//! Out: eax the value, ecx the cars (0 with none), esi the last car's block (when any);
//!      the rest kept (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x455826), each view drawn. What +0x25C
//! and +0x84 are (the lights on, the car running: a reading) is not settled. Refuses
//! (before anything) more than eight cars. Shadow mode: by state. Self-test: the game's
//! blocks with +0x25C/+0x84 set by pattern, 0 .. 8 cars, lamp counts; all put back
//! (skipped with a null block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::lamps_wanted", rva(0x4536D7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { lamps_wanted() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(LAMPS_WANTED_RVA)], &rust);
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
    let regions: [Region; 1] = [at(LAMPS_WANTED_RVA)];
    let mut kept = regions.to_vec();
    kept.extend([at(CARS_RVA), at(LAMP_COUNT_RVA)]);
    for &b in &blocks {
        kept.extend([(b + 0x25C, 4), (b + 0x84, 4)]);
    }
    let saved = capture(&kept);
    for (cars, lamps, pattern) in [
        (0u32, 6u32, 0xFFFFu32),
        (1, 6, 0x0003),
        (1, 7, 0x0001),
        (8, 0x40, 0x5A5A),
        (8, 0x40, 0x8001),
        (5, 0xFFFF_FFFE, 0x0300),
        (4, 2, 0x0300),
    ] {
        check_state(
            &mut t,
            &format!("cars {cars} lamps {lamps:#x} pattern {pattern:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CARS_RVA, cars);
                game_write(LAMP_COUNT_RVA, lamps);
                game_write(LAMPS_WANTED_RVA, 0xAAAA_AAAAu32);
                for (n, &b) in blocks.iter().enumerate() {
                    let on = |bit: usize| u32::from(pattern >> bit & 1 != 0) * (0x100 + n as u32);
                    ((b + 0x25C) as *mut u32).write_unaligned(on(2 * n));
                    ((b + 0x84) as *mut u32).write_unaligned(on(2 * n + 1));
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    let refused = unsafe {
        game_write(CARS_RVA, 9u32);
        let r = lamps_wanted().is_err();
        restore(&kept, &saved);
        r
    };
    t.check(refused, || "nine cars not refused".into());
    t.finish()
}
