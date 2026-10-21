//! `physics::fill_race_slot` (0x462263): a loose part (eax: its number) put into one of
//! the eight `g_raceSlots` (0x78 bytes each): the first empty one (+0 0), else the one
//! with the lowest +0x64 (unsigned, the first of equals; with every +0x64 -1 the original
//! takes ebp as it came: refused). There: +0 the part; its middle (`g_partMiddleX` /
//! `Y` / `Z`[part], whole numbers) turned by `g_crashMatrix` (`scene::rotate_point`),
//! `fistp` into `dword_635004` .. and plus the car's place into +4 .. +0xC; +0x60
//! `g_car` +0xE00, +0x64 `g_car` +8; the car's matrix (16 dwords) at +0x10; +0x50 ..
//! +0x58 the turned middle x `flt_71BE8C` plus the car's motion (+2.0 for y); +0x68 ..
//! +0x70 the car's spin. A reading: a part breaking off, flying with the car's motion.
//!
//! In:  eax the part
//! Out: eax `g_car` +8, ebx the part, ecx the slot's number (8 when none was empty),
//!      edx -1 or the lowest +0x64, esi the slot, ebp as the search leaves it; no flags read
//!
//! Caller: `physics::fill_slot_on_bit`.
//! Refuses (before anything) a part past 0xFF, or no slot to take.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): empty slots and full ones, parts by lot, under three precisions.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{
    TEST_CONTROL_WORDS, ext_to_int, int_80, int_mul_add, with_control_word,
};
use crate::scene::matrix3::test_floats;
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use rc2k_core::physics::parts::{LAST_PART, choose_slot};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::fill_race_slot", rva(0x462263), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const SLOTS: usize = 0x71BAC8; // g_raceSlots

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The steps on `r` (eax the part).
// Converted: `rc2k_core::physics::parts::choose_slot` (the slot; the floats stay x87).
pub fn fill_race_slot_in(r: &mut Regs) -> Result<(), Refused> {
    let part = r.eax;
    if part > LAST_PART {
        return Err(BadInput("a part past 0xFF").into());
    }
    let first = game_addr(rva(SLOTS)) as u32;
    unsafe {
        let slot_at = |c: usize| first + 0x78 * c as u32;
        let Some(choice) = choose_slot(|c| at(slot_at(c)), |c| at(slot_at(c) + 0x64)) else {
            return Err(BadInput("no race slot to take (every +0x64 -1)").into());
        };
        r.ecx = choice.ecx;
        r.esi = slot_at(choice.slot);
        if let Some(lowest) = choice.lowest {
            r.edx = lowest;
            r.ebp = r.esi;
        }
        let s = r.esi;
        put(s, part);
        r.ebx = part;
        let i = |t: usize| int_80(g(t + 4 * part as usize) as i32, false);
        let out = rotate_point(
            game_addr(rva(0x71A08C)) as u32,
            [i(0x633370), i(0x633770), i(0x633B70)],
        );
        for (k, e) in out.iter().enumerate() {
            let n = ext_to_int(*e) as u32;
            p(0x635004 + 4 * k, n);
            r.eax = n.wrapping_add(g(0x71A10C + 4 * k));
            put(s + 4 + 4 * k as u32, r.eax);
        }
        r.eax = g(0x71ADD8);
        put(s + 0x60, r.eax);
        r.eax = g(0x719FE0);
        put(s + 0x64, r.eax);
        for k in 0..16u32 {
            put(s + 0x10 + 4 * k, g(0x71A08C + 4 * k as usize));
        }
        let k = g(0x71BE8C);
        put(
            s + 0x50,
            int_mul_add(g(0x635004) as i32, k, g(0x71A170), None),
        );
        put(
            s + 0x54,
            int_mul_add(g(0x635008) as i32, k, g(0x71A174), Some(g(0x5C5D3C))),
        );
        put(
            s + 0x58,
            int_mul_add(g(0x63500C) as i32, k, g(0x71A178), None),
        );
        for k in 0..3u32 {
            put(s + 0x68 + 4 * k, g(0x71A15C + 4 * k as usize));
        }
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn fill_race_slot(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    fill_race_slot_in(&mut r)?;
    Ok(crate::race::stage_prefs::left_of(&r))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_race_slot(r) }
}

/// What the slots' steps write.
pub fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![(a(SLOTS), 8 * 0x78), (a(0x635004), 12), (a(0x71A270), 4)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The inputs the slot tests set.
pub fn inputs() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x633370), 0x400),
        (a(0x633770), 0x400),
        (a(0x633B70), 0x400),
        (a(0x71A08C), 0x40),
        (a(0x71A10C), 12),
        (a(0x71A15C), 0x20),
        (a(0x71ADD8), 4),
        (a(0x719FE0), 4),
        (a(0x71BE8C), 4),
        (a(0x71B3F8), 4),
        (a(0x71AA14), 0x1C),
        (a(0x71BA2C), 0x1C),
        (a(0x62DAD0), 4),
    ]
}

/// A slot case: `empty` slots from the n-th on, parts' middles and the car by `seed`.
pub fn set_slots(seed: u32, empty: Option<u32>) {
    let first = game_addr(rva(SLOTS)) as u32;
    unsafe {
        for c in 0..8u32 {
            let s = first + 0x78 * c;
            for k in 0..30u32 {
                put(s + 4 * k, 0x5A00_0000 | (c << 8) | k);
            }
            put(
                s,
                if empty.is_some_and(|e| c >= e) {
                    0
                } else {
                    c + 1
                },
            );
            put(
                s + 0x64,
                (seed.wrapping_mul(31) ^ c.wrapping_mul(0x77)) % 1000,
            );
        }
    }
    for k in 0..0x100usize {
        p(0x633370 + 4 * k, (k as u32 * 37 + seed) % 2000);
        p(0x633770 + 4 * k, (k as u32 * 53) % 900);
        p(0x633B70 + 4 * k, (k as u32 * 71 + seed * 3) % 3000);
    }
    let mut f = [0u32; 24];
    test_floats(seed * 5 + 1, &mut f);
    for (k, v) in f.iter().take(12).enumerate() {
        p(0x71A08C + 4 * k, (f32::from_bits(*v) % 2.0).to_bits());
    }
    for k in 12..16usize {
        p(0x71A08C + 4 * k, 0);
    }
    for k in 0..3usize {
        p(0x71A10C + 4 * k, seed.wrapping_mul(0x0012_3457) ^ k as u32);
        p(0x71A15C + 4 * k, f[12 + k]);
        p(0x71A170 + 4 * k, f[15 + k]);
    }
    p(0x71ADD8, seed);
    p(0x719FE0, seed * 7);
    p(0x71BE8C, 6.0e-5f32.to_bits());
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (seed, empty) in [
                (1u32, Some(0u32)),
                (2, Some(3)),
                (3, Some(7)),
                (4, None),
                (5, None),
            ] {
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}, empty from {empty:?}"),
                    call_original,
                    Regs {
                        eax: seed * 13,
                        ebp: 0x1234,
                        ..Regs::SENTINEL
                    },
                    &|| set_slots(seed, empty),
                    &all,
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
