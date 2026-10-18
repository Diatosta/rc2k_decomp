//! `physics::hit_part_checks` (0x441046): +0x284 (`unk_71A25C`) 1; with bits 5, 6, 1, 2
//! of +0x298 all set -1; with bits 7, 8, 3, 4 also set 0 (a reading: which parts are
//! left); then, with +0x26C 0, `physics::drop_hit_parts` and `physics::sound_hit_parts`.
//!
//! In:  nothing
//! Out: as the calls leave them; CF as the last `bt` or the calls leave it (no caller
//!      reads it: the next is a call)
//!
//! Caller: `sub_43B635` (0x43B957), every physics step.
//! Refuses what the calls refuse; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): bit patterns, the switch, hits by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::physics::car_sounds::test_regions;
use crate::physics::drop_hit_parts::{drop_hit_parts_in, g, p, set_hits};
use crate::physics::fill_race_slot::{inputs, regions as slot_regions, set_slots};
use crate::physics::sound_hit_parts::sound_hit_parts_in;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::hit_part_checks", rva(0x441046), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn all_set(bits: &[u32]) -> bool {
    bits.iter().all(|&b| g(0x71A270) >> b & 1 != 0)
}

/// The steps on `r`.
pub fn hit_part_checks_in(r: &mut Regs) -> Result<(), Refused> {
    p(0x71A25C, 1);
    if !all_set(&[5, 6, 1, 2]) {
        return Ok(());
    }
    p(0x71A25C, u32::MAX);
    if !all_set(&[7, 8, 3, 4]) {
        return Ok(());
    }
    p(0x71A25C, 0);
    if g(0x71A244) != 0 {
        return Ok(());
    }
    drop_hit_parts_in(r)?;
    sound_hit_parts_in(r)
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut r = slot_regions();
    r.extend([(a(0x71A25C), 4), (a(0x71A2D0), 0x10), (a(0x71A32C), 0x10)]);
    let o = g(0x512A10 + 4 * (g(0x71B3F8) & 7) as usize);
    if o != 0 {
        r.extend([(o + 0x10, 4), (o + 0x214, 0x14)]);
    }
    r
}

/// The Rust version.
pub unsafe fn hit_part_checks(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match hit_part_checks_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { hit_part_checks(r) }
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
    all.extend(inputs());
    all.extend(slot_regions());
    all.extend([
        (a(0x71A25C), 4),
        (a(0x71A244), 4),
        (a(0x71A2D0), 0x10),
        (a(0x71A32C), 0x10),
        (a(0x719FE4), 4),
        (a(0x719FFC), 4),
    ]);
    let saved = capture(&all);
    for seed in 0..24u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                ebp: 0x1234,
                ..Regs::SENTINEL
            },
            &|| {
                unsafe {
                    for k in 0..0x98u32 {
                        ((o + 4 * k) as *mut u32).write(0xA5A5_0000 | k);
                    }
                }
                for k in 0..8usize {
                    p(0x512A10 + 4 * k, o);
                }
                set_slots(seed, Some(seed % 4));
                p(0x71B3F8, seed % 8);
                set_hits(seed);
                if seed % 3 != 0 {
                    p(0x71A270, 0x1FE | (seed << 12));
                }
                p(0x71A244, (seed % 5 == 4) as u32);
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    drop(object);
    t.finish()
}
