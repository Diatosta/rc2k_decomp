//! `physics::sound_hit_parts` (0x4411EC): for kinds with a `dword_5C5BD0` entry (below
//! 0x22), the same four tests as `physics::drop_hit_parts` on the timers +0x354 ..
//! +0x360 and the bits 1 .. 4 of +0x298, by `physics::sound_on_bit` (the timer 256).
//!
//! In:  nothing
//! Out: as the calls leave them; CF when the kind has no entry (no caller reads it)
//!
//! Caller: `physics::hit_part_checks`.
//! Refuses what `sound_on_bit` refuses.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test sound object.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::car_sounds::test_regions;
use crate::physics::drop_hit_parts::{Tests, g, hit_tests, p, set_hits};
use crate::physics::sound_on_bit::sound_on_bit;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_hit_parts", rva(0x4411EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TESTS: Tests = [
    (5, 0x71A32C, 1),
    (9, 0x71A330, 2),
    (0xE, 0x71A334, 3),
    (0x19, 0x71A338, 4),
];

/// The steps on `r`; Some(carry) when it ended on the kind's test.
pub fn sound_hit_parts_in(r: &mut Regs) -> Result<(), Refused> {
    let kind = g(0x719FFC);
    if kind >= 0x22 || g(0x5C5BD0 + 4 * kind as usize) == 0 {
        r.set_carry(true);
        return Ok(());
    }
    hit_tests(r, &TESTS, &mut |r| {
        unsafe { sound_on_bit(r.ecx & 0xFF) }?.apply(r);
        Ok(())
    })
}

/// The Rust version.
pub unsafe fn sound_hit_parts(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    sound_hit_parts_in(&mut r)?;
    let left = crate::race::stage_prefs::left_of(&r);
    Ok(left)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { sound_hit_parts(r) }
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut r = vec![(a(0x71A32C), 0x10), (a(0x71A270), 4)];
    let o = g(0x512A10 + 4 * (g(0x71B3F8) & 7) as usize);
    if o != 0 {
        r.extend([(o + 0x10, 4), (o + 0x214, 0x14)]);
    }
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut object = vec![0u32; 0x98];
    let o = object.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all = test_regions(o);
    all.extend([
        (a(0x71A32C), 0x10),
        (a(0x71A2D0), 0x10),
        (a(0x71A270), 4),
        (a(0x719FE4), 4),
        (a(0x719FFC), 4),
    ]);
    let saved = capture(&all);
    for seed in 0..20u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                unsafe {
                    for k in 0..0x98u32 {
                        ((o + 4 * k) as *mut u32).write(0xA5A5_0000 | k);
                    }
                }
                for k in 0..8usize {
                    p(0x512A10 + 4 * k, o);
                }
                p(0x71B3F8, seed % 8);
                set_hits(seed);
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    drop(object);
    t.finish()
}
