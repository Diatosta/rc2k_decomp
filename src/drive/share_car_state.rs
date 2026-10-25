//! `drive::share_car_state` (0x45699C): one car's state from its block (esi, 0xFE0 bytes)
//! into its record (edi, `off_71B3D8[n]`, read by the drawing and the sounds) and back:
//! - a count at +0xFAC: when not 0, record +0x30 = 1 and the count less 1;
//! - record +0x2C = the sound object's (ebp) +0xF0, which is cleared (`xchg`);
//! - two levels (record +0x4C0, +0x4C4) from block +0x2B4, +0x2B8: 0 below
//!   `flt_63DB5C`, else (x - it) / `flt_5C5D44` `fistp`'d (through `dword_62400C`) held to
//!   0 .. 7 (the sign of the difference);
//! - the car file's light record (`g_lightRecords`, 0x134 each, by
//!   `race::car_file_number` of block +0x24) +0x2C .. +0x38 into record +0xC .. +0x18;
//! - about eighty fields moved as the original moves them (`MOVES`, the wheels' four
//!   each), four of them cleared by the block's switches (+0x600 .. and bits 5 .. 8 of
//!   +0x298), a part only when block +0xA34 is set;
//! - record +0x258 (6 dwords) into block +0x700, whose 0x80000000 entries are then
//!   filled from the next one that is not (round, at most six looked at); record +0x32C
//!   (8) into block +0xA3C, record +0x3F4 (0x11) into block +0xA5C.
//!
//! In:  esi the block, edi the record, ebp the sound object
//! Out: eax block +0x1A0 (with +0xA34 set) or +0x1BC; ebx +0x1A4, ecx +0x1AC, edx +0x92C;
//!      the rest kept (the copies are inside `pusha` / `popa`); no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), for each of the eight cars, inside `pusha` /
//! `popa`. Refuses (before anything) a car file number above 0x21 (past `g_lightRecords`).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::{compare_full, sub_div_to_int};
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::mud::car_file_number;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_store::RACE_VIEW;
use crate::scene::draw_store::DRAW;
use rc2k_core::drive::sharing;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The light record's number for the block at `s`, or the refusal.
unsafe fn light_record(s: u32) -> Result<u32, Refused> {
    let n = car_file_number(unsafe { at(s + 0x24) });
    sharing::check_light_record(n)?;
    Ok(n)
}

/// The Rust core: block `s`, record `o`, sound object `p`, the scratch at `scratch`.
// Converted: `rc2k_core::drive::sharing` (the levels' x87 sums stay).
pub unsafe fn share_car_state_at(s: u32, o: u32, p: u32, scratch: u32) -> Result<(), Refused> {
    unsafe {
        let n = light_record(s)?;
        if let Some(count) = sharing::count_down(at(s + 0xFAC)) {
            put(o + 0x30, 1);
            put(s + 0xFAC, count);
        }
        put(o + 0x2C, at(p + 0xF0));
        put(p + 0xF0, 0);
        let from = RACE_VIEW.level_from.bits();
        let step = NUMBERS.four.bits();
        for (out, field) in [(0x4C0, 0x2B4), (0x4C4, 0x2B8)] {
            put(o + out, 0);
            if compare_full(at(s + field), from) & 0x100 == 0 {
                let v = sub_div_to_int(at(s + field), from, step);
                put(scratch, v as u32);
                put(o + out, sharing::level(v));
            }
        }
        put(o + 0x1C, at(s + 0x240));
        let light = LIGHT_RECORDS.record_addr(n);
        for (r, l) in sharing::LIGHT_MOVES {
            put(o + r, at(light + l));
        }
        put(o + 0x24, s + 0x8C0);
        put(o + 0x28, s + 0x740);
        put(o + 8, x87::loaded(at(s + 0x1D8)));
        put(o + 4, n);
        for (r, b) in sharing::MOVES_A {
            put(o + r, at(s + b));
        }
        for (offsets, first) in sharing::QUADS {
            let (a, bb, c) = (at(s + first), at(s + 0x1A4), at(s + 0x1AC));
            for base in [offsets[0], offsets[0] + 0x48] {
                put(o + base, a);
                put(o + base + 4, bb);
                put(o + base + 8, 0);
                put(o + base + 0xC, c);
            }
        }
        for (k, r) in sharing::WHEELS.into_iter().enumerate() {
            put(o + r, at(s + 0x920 + 4 * k as u32));
        }
        for (k, r) in sharing::WHEELS.into_iter().enumerate() {
            let k = k as u32;
            if sharing::wheel_cleared(at(s + 0x600 + 4 * k), at(s + 0x298), k) {
                put(o + r, 0);
            }
        }
        put(o + 0x210, 0);
        put(o + 0x21C, 0);
        let zero = x87::loaded(0);
        for r in [0x24C, 0x250, 0x254, 0x244] {
            put(o + r, zero);
        }
        let switched = at(s + 0xA34) != 0;
        if switched {
            for (r, b) in sharing::MOVES_B {
                put(o + r, at(s + b));
            }
            for (r, b) in [(0x230, 0xA28), (0x234, 0xA2C), (0x238, 0xA30)] {
                put(o + r, x87::from_int(at(s + b) as i32));
            }
            put(o + 0x22C, at(s + 0xA38));
        }
        for k in 0..8 {
            put(o + 0x2CC + 4 * k, at(s + 0x64 + 4 * k));
        }
        let mut kept: [u32; 6] = std::array::from_fn(|k| at(o + 0x258 + 4 * k as u32));
        sharing::fill_holes(&mut kept);
        for (k, v) in kept.into_iter().enumerate() {
            put(s + 0x700 + 4 * k as u32, v);
        }
        for k in 0..8 {
            put(s + 0xA3C + 4 * k, at(o + 0x32C + 4 * k));
        }
        for k in 0..0x11 {
            put(s + 0xA5C + 4 * k, at(o + 0x3F4 + 4 * k));
        }
        Ok(())
    }
}

/// The Rust version.
/// The Rust version: the car block `block` (esi), its record `record` (edi), its sound
/// object `sound` (ebp). The registers it leaves are read by no caller (liveness: only
/// DF after `share_all_cars`' call).
pub unsafe fn share_car_state(block: u32, record: u32, sound: u32) -> Result<(), Refused> {
    unsafe {
        share_car_state_at(
            block,
            record,
            sound,
            DRAW.scratch.addr() as u32, /* dword_62400C */
        )
    }
}
