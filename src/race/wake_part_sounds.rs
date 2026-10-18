//! `race::wake_part_sounds` (0x45D1EA): for each race car (car 0 at least) and each of
//! its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 = 1; when +0x700 was
//! exactly 10 (the `jnz` after the `mov` reads the `cmp`'s flags) and the current car is
//! shown (`dword_641680[g_currentCar]` not -1), the car's sound object
//! (`g_raceSoundObjects[car & 7]`) +0x264 = 1 and +0x10 = 1.
//!
//! In:  nothing
//! Out: ecx the cars done, ebx 6, esi the last block, edi its sound object, eax the last
//!      view read (a `mov` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45429B` (0x454747), each frame drawn. What the six
//! slots are (+0x700 counts up; a part's sound, a reading) is not settled. Refuses (before
//! anything) more than eight cars, a null block, a null sound object it would write, and
//! `g_currentCar` past 7 where it is read. Shadow mode: by state. Self-test: the game's
//! blocks and sound objects with patterns (counts 0, 9, 10, 11, 0xFFFFFFFF; flags set
//! and not), cars 0, 1, 8, the current car shown and not; all put back (skipped with a
//! null block or object).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::objects::{CAR_BLOCKS_RVA, CARS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::wake_part_sounds", rva(0x45D1EA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SOUND_OBJECTS: usize = rva(0x512A10); // g_raceSoundObjects
const SHOWN: usize = rva(0x641680); // dword_641680[8]

fn pointers(table: usize) -> Vec<u32> {
    (0..8)
        .map(|n| unsafe { game_read::<u32>(table + 4 * n) })
        .collect()
}

/// The blocks' six flags and the objects' two dwords, of those that are there.
fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = pointers(CAR_BLOCKS_RVA)
        .into_iter()
        .filter(|&b| b != 0)
        .map(|b| (b + 0x48, 0x18))
        .collect();
    for o in pointers(SOUND_OBJECTS).into_iter().filter(|&o| o != 0) {
        v.extend([(o + 0x10, 4), (o + 0x264, 4)]);
    }
    v
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { wake_part_sounds() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks = pointers(CAR_BLOCKS_RVA);
    let objects = pointers(SOUND_OBJECTS);
    if blocks.contains(&0) || objects.contains(&0) {
        return t.skip("a car block or sound object is null (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        at(CARS_RVA),
        at(crate::drive::car::CURRENT_CAR_RVA),
        (crate::framework::game_addr(SHOWN) as u32, 0x20),
    ]);
    kept.extend(blocks.iter().map(|&b| (b + 0x700, 0x18)));
    let saved = capture(&kept);
    let counts = [0u32, 9, 10, 11, 0xFFFF_FFFF, 10];
    for (cars, current, shown, flags) in [
        (0u32, 0u32, 0u32, 0u32),
        (1, 0, 0, 0x15),
        (8, 3, 0xFFFF_FFFF, 0),
        (8, 7, 1, 0x2A),
        (5, 2, 0, 0x3F),
    ] {
        check_state(
            &mut t,
            &format!("cars {cars} current {current} shown {shown:#x} flags {flags:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                game_write(CARS_RVA, cars);
                game_write(crate::drive::car::CURRENT_CAR_RVA, current);
                for k in 0..8 {
                    game_write(
                        SHOWN + 4 * k,
                        if k as u32 == current { shown } else { k as u32 },
                    );
                }
                for (n, &b) in blocks.iter().enumerate() {
                    for s in 0..6u32 {
                        w(b + 0x48 + 4 * s, (flags >> s) & 1);
                        w(b + 0x700 + 4 * s, counts[(s as usize + n) % counts.len()]);
                    }
                }
                for &o in &objects {
                    w(o + 0x10, 0x55);
                    w(o + 0x264, 0x66);
                }
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(CARS_RVA, 9u32);
        wake_part_sounds().is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "nine cars not refused".into());
    t.finish()
}
