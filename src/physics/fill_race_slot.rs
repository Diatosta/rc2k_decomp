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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87_run::{ext_to_int, int_80, int_mul_add};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};
use rc2k_core::physics::parts::{LAST_PART, choose_slot};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
