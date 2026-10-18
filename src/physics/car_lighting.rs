//! `physics::car_lighting` (0x44ECE4): the light on the work car from the face under it.
//! The car's x / z into the probe point, the old light (+0x244) into `dword_5CCCA8`,
//! `dword_62DAFC` / `62DB00` 0, the last face (+0xE00) into `dword_5CCC78`;
//! `physics::find_ground`; no face: done. With `dword_5CCC6C` 1 the shares
//! (`dword_639800`, `804`) taken from 0x10000; kept in `dword_62DB08` / `0C`. The face's
//! four corner lights (+0x14 ..; with its texture's +0x2C 1 each scaled by
//! ((byte +0x22 + k) >> 1 ^ 0x7F) + 0x80, `mul`, >> 8) blended by the shares (`imul`,
//! `shrd` 16) into +0x244; with +0xFB4 set the change held to 10 a step (or none with
//! `dword_6235E8`); `g_lightTable[it]` into `dword_62DAFC` and +0x240. With bit 2 of the
//! face's +2 also two colour blends (+0x28 .., +0x2C ..) through `dword_635518[]` /
//! `dword_635918[]` into `dword_62DB00` / `62DB04`, each added to +0x240 byte by byte
//! with saturation (four bytes with `g_hasMmx`, else the low three).
//!
//! In:  nothing
//! Out: eax, ebx (the texture's record), ecx, edx, esi (the face) as left; no flags read
//!
//! Caller: `sub_43B635` (0x43B990), every physics step.
//! Refuses a light index past 0xFF (the tables are 256) and no faces; what was written is
//! put back. Shadow mode runs the original alone (`find_ground` through
//! `physics/reset_parts.rs`).
//! Self-test: a copy whose `find_ground` goes to a stand-in, run on the game's variables
//! (saved and put back) with test faces, against the Rust version: faces found and not,
//! textures, both MMX paths.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::reset_parts::{
    FIND_GROUND, both_with, copy_of, dispatch_reset, regions as reset_regions,
};
use crate::race::car_starts::{Case, Part, prepare};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::car_lighting", rva(0x44ECE4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn byte(a: u32) -> u32 {
    unsafe { (a as *const u8).read() as u32 }
}

/// lo + ((hi - lo) x s >> 16) (`imul`, `shrd`); edx the product's high half.
fn blend(r: &mut Regs, lo: u32, hi: u32, s: u32) -> u32 {
    let product = (hi.wrapping_sub(lo) as i32 as i64).wrapping_mul(s as i32 as i64);
    r.edx = (product >> 32) as u32;
    (((product as u64) >> 16) as u32).wrapping_add(lo)
}

/// The two-way blend of the four corners now in `dword_5CCC90` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners(r: &mut Regs) -> u32 {
    let (s, t) = (g(0x639800), g(0x639804));
    let a = blend(r, g(0x5CCC90), g(0x5CCC94), s);
    p(0x5CCCA4, a);
    let b = blend(r, g(0x5CCC98), g(0x5CCC9C), s);
    p(0x5CCCA0, b);
    blend(r, g(0x5CCCA4), g(0x5CCCA0), t)
}

/// The colour `c` added to +0x240 byte by byte with saturation, as the original by
/// `g_hasMmx`.
fn add_colour(c: u32) -> u32 {
    let light = g(0x71A218);
    let sat = |x: u32, y: u32, k: u32| (((x >> k) & 0xFF) + ((y >> k) & 0xFF)).min(0xFF) << k;
    if g(0x5183A8) != 0 {
        sat(c, light, 0) | sat(c, light, 8) | sat(c, light, 16) | sat(c, light, 24)
    } else {
        sat(c, light, 0) | sat(c, light, 8) | sat(c, light, 16) | (c & 0xFF00_0000)
    }
}

/// The steps on `r`.
pub fn car_lighting_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    p(0x639824, g(0x71A10C));
    p(0x63982C, g(0x71A114));
    p(0x5CCCA8, g(0x71A21C));
    r.eax = 0;
    p(0x62DAFC, 0);
    p(0x62DB00, 0);
    r.eax = g(0x71ADD8);
    p(0x5CCC78, r.eax);
    part(FIND_GROUND, r)?;
    r.ebx = g(0x5CCC74);
    if r.ebx == u32::MAX {
        return Ok(());
    }
    if g(0x5CCCBC) == 0 {
        return Err(BadInput("no course faces").into());
    }
    if g(0x5CCC6C) == 1 {
        r.eax = 0x10000u32.wrapping_sub(g(0x639800));
        p(0x639800, r.eax);
        r.eax = 0x10000u32.wrapping_sub(g(0x639804));
        p(0x639804, r.eax);
    }
    p(0x62DB08, g(0x639800));
    p(0x62DB0C, g(0x639804));
    let f = (r.ebx << 6).wrapping_add(g(0x5CCCBC));
    r.esi = f;
    unsafe {
        for k in 0..4u32 {
            p(0x5CCC90 + 4 * k as usize, byte(f + 0x14 + k));
        }
        r.ebx = (game_addr(rva(0x979F30)) as u32).wrapping_add(byte(f + 0x21) << 6);
        if byte(r.ebx + 0x2C) == 1 {
            for k in 0..4u32 {
                let v = ((byte(f + 0x22 + k) >> 1) ^ 0x7F) + 0x80;
                let product = v as u64 * g(0x5CCC90 + 4 * k as usize) as u64;
                r.edx = (product >> 32) as u32;
                p(0x5CCC90 + 4 * k as usize, (product as u32) >> 8);
            }
        }
        let mut light = corners(r);
        p(0x71A21C, light);
        if g(0x71AF8C) != 0 {
            let old = g(0x5CCCA8);
            if g(0x6235E8) != 0 {
                p(0x71A21C, old);
            } else {
                r.eax = 10;
                let gap = light.wrapping_sub(old);
                r.ecx = if (gap as i32) < 0 {
                    gap.wrapping_neg()
                } else {
                    gap
                };
                if r.ecx >= r.eax {
                    r.ecx = gap;
                    if (gap as i32) < 0 {
                        r.eax = r.eax.wrapping_neg();
                    }
                    r.eax = r.eax.wrapping_add(old);
                    p(0x71A21C, r.eax);
                }
            }
        }
        light = g(0x71A21C);
        if light > 0xFF {
            return Err(BadInput("a light index past 0xFF").into());
        }
        r.eax = g(0x635118 + 4 * light as usize);
        p(0x62DAFC, r.eax);
        p(0x71A218, r.eax);
        if byte(f + 2) & 4 == 0 {
            return Ok(());
        }
        for (first, table, to) in [
            (0x28u32, 0x635518usize, 0x62DB00usize),
            (0x2C, 0x635918, 0x62DB04),
        ] {
            for k in 0..4u32 {
                p(0x5CCC90 + 4 * k as usize, byte(f + first + k));
            }
            let i = corners(r);
            if i > 0xFF {
                return Err(BadInput("a colour index past 0xFF").into());
            }
            r.eax = g(table + 4 * i as usize);
            p(to, r.eax);
            r.eax = add_colour(r.eax);
            p(0x71A218, r.eax);
        }
    }
    Ok(())
}

/// What the function writes.
fn written() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x639800), 8),
        (a(0x639824), 12),
        (a(0x5CCC78), 4),
        (a(0x5CCC90), 0x1C),
        (a(0x62DAFC), 0x14),
        (a(0x71A218), 8),
    ]
}

/// The Rust version: what was written is put back on a refusal.
pub unsafe fn car_lighting(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let all = written();
    let saved = capture(&all);
    let mut r = *entry;
    match car_lighting_in(&mut r, part) {
        Ok(()) => Ok(r),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_reset(&HOOK, r, call_original, car_lighting);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x44ECE4,
        0x3C7,
        &[0xFF, 0x35, 0x0C, 0xA1, 0x71, 0x00],
        &[0x42],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut faces = vec![0u8; 0x80];
    let fa = faces.as_mut_ptr() as u32;
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut extra = written();
    extra.extend([
        (a(0x5CCC6C), 4),
        (a(0x5CCC74), 4),
        (a(0x5CCCBC), 4),
        (a(0x71AF8C), 4),
        (a(0x6235E8), 4),
        (a(0x5183A8), 4),
        (fa, 0x80),
    ]);
    let mut all = reset_regions();
    all.extend(extra.iter().copied());
    let saved = capture(&all);
    for seed in 0..16u32 {
        let fill = || unsafe {
            prepare(
                &Case {
                    mode: 2,
                    no_place: 0,
                    side: 0,
                    cars_2: 1,
                    current: seed % 4,
                    cars: 4,
                },
                seed.wrapping_mul(0x2026_1017),
            );
            for k in 0..0x80u32 {
                ((fa + k) as *mut u8).write((k.wrapping_mul(37) ^ seed.wrapping_mul(11)) as u8);
            }
            ((fa + 0x40 + 2) as *mut u8).write(if seed % 2 == 0 { 4 } else { 0 });
            p(0x5CCCBC, fa);
            p(0x5CCC74, if seed == 5 { u32::MAX } else { 1 });
            p(0x5CCC6C, seed % 3);
            p(0x639800, seed.wrapping_mul(0x1357) & 0xFFFF);
            p(0x639804, seed.wrapping_mul(0x2468) & 0xFFFF);
            p(0x71A21C, 100);
            p(0x71AF8C, (seed % 4 != 0) as u32);
            p(0x6235E8, (seed == 9) as u32);
            p(0x5183A8, seed % 2);
        };
        both_with(
            &mut t,
            &format!("seed {seed}"),
            &copy,
            0,
            Regs::SENTINEL,
            &fill,
            car_lighting,
            &extra,
        );
    }
    restore(&all, &saved);
    drop((copy, faces));
    t.finish()
}
