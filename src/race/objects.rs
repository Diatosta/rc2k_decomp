//! `race::loadlevel_init`'s "reloadobjects" (IDA, 2026-10-16): the object slots' kinds
//! (the cars' three each), the objects loaded, and each car's collision points: its
//! points (12 bytes: x, y, z, whole numbers) sorted by four quicksorts of the original's
//! own (each step of theirs kept: the left side moves one place a pass, keys compared
//! unsigned), turned to floats (/ 65536) into the car's block, two points put in at 4.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const COUNTS_RVA: usize = rva(0x63239C); // dword_63239C: each slot's points
pub const FIRSTS_RVA: usize = rva(0x631F9C); // dword_631F9C: each slot's first point
pub const POINTS_RVA: usize = rva(0x5C6FA4); // g_objectPoints
pub const CAR_POINTS_RVA: usize = rva(0x6D31BC); // dword_6D31BC
const SCRATCH_RVA: usize = rva(0x62400C); // dword_62400C: the x87's way in
const SCALE_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
const LIFT: u32 = 0x5_0000;
const MOST_SORTED: u32 = 0x3D;
const MOST_CONVERTED: u32 = 0x40;
const XS: u32 = 0xAF0;
const YS: u32 = 0xBF0;
const ZS: u32 = 0xCF0;
pub const CARS_RVA: usize = rva(0x63DB90); // g_raceCars
pub const CAR_BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
pub const SLOT_KINDS_RVA: usize = rva(0x5CCE7C); // g_slotKinds
const KIND_DEFAULTS_RVA: usize = rva(0x6091B4); // dword_6091B4: slots 1 .. 0x3F
pub const FIXED_KINDS_RVA: usize = rva(0x609294); // dword_609294 .. dword_6092B0
const FIXED_KINDS: [u32; 8] = [0xCF, 0x79, 0xCC, 0xCD, 0xC8, 0xC9, 0xCA, 0xCB];
pub const KEPT_RVA: usize = rva(0x5CCF7C); // dword_5CCF7C = dword_621844
const KEPT_FROM_RVA: usize = rva(0x621844);
pub const KIND_10_TABLE_RVA: usize = rva(0x5C5818); // dword_5C5818
pub const KIND_11_TABLE_RVA: usize = rva(0x5C58A0); // dword_5C58A0
const FIRST_CAR_SLOT: u32 = 0x50;
const CAR_BLOCKS: u32 = 8;
const CAR_SLOT_STEP: u32 = 0x14;

/// `race::car_kind_10` (0x41F4A1) and `race::car_kind_11` (0x41F485): a car's entry of
/// one of the two tables (the car held to 0 .. 0x21).
pub fn car_table(table: usize, car: u32) -> u32 {
    let n = (car as i32).clamp(0, 0x21) as usize;
    unsafe { game_read(table + 4 * n) }
}

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// One of the four sorts: the key's offset in the point, and whether the order is
/// downwards.
#[derive(Clone, Copy)]
pub struct Sort {
    pub key: u32,
    pub down: bool,
}

pub const BY_Y: Sort = Sort {
    key: 4,
    down: false,
};
pub const BY_Z: Sort = Sort {
    key: 8,
    down: false,
};
pub const BY_X_DOWN: Sort = Sort { key: 0, down: true };

/// The sorts (0x426BE6 by y, 0x426CA2 by z, 0x426D5E and 0x426E16 by x downwards): the
/// points from esi to edi (both included), step by step as the original, the registers as
/// it leaves them.
pub unsafe fn sort_points(s: Sort, g: &mut Regs) {
    unsafe {
        if g.esi >= g.edi {
            return;
        }
        g.ecx = g.esi;
        g.edx = g.edi;
        g.eax = (g.edx.wrapping_sub(g.ecx) >> 1) / 12 * 12;
        g.eax = get(g.eax.wrapping_add(g.ecx).wrapping_add(s.key));
        // Keeps its place when the key belongs on the pivot's left.
        let stays_left = |pivot: u32, key: u32| if s.down { pivot < key } else { pivot > key };
        let goes_on_right = |pivot: u32, key: u32| if s.down { pivot > key } else { pivot < key };
        loop {
            g.ebx = get(g.ecx + s.key);
            g.ecx = g.ecx.wrapping_add(12);
            if !stays_left(g.eax, g.ebx) {
                g.ecx = g.ecx.wrapping_sub(12);
            }
            loop {
                g.ebx = get(g.edx + s.key);
                g.edx = g.edx.wrapping_sub(12);
                if !goes_on_right(g.eax, g.ebx) {
                    break;
                }
            }
            g.edx = g.edx.wrapping_add(12);
            if g.ecx == g.edx {
                break;
            }
            for k in [0u32, 4, 8] {
                g.ebp = get(g.ecx + k);
                g.ebx = get(g.edx + k);
                put(g.edx + k, g.ebp);
                put(g.ecx + k, g.ebx);
            }
            g.ebx = get(g.ecx + s.key);
            if g.ebx == get(g.edx + s.key) {
                g.ecx = g.ecx.wrapping_add(12);
            }
        }
        let edi = g.edi;
        g.edi = g.ecx.wrapping_sub(12);
        if g.esi < g.edi {
            sort_points(s, g);
        }
        g.esi = g.edi.wrapping_add(0x18);
        g.edi = edi;
        if g.esi < g.edi {
            sort_points(s, g);
        }
    }
}

/// A call: 0 .. 3 the sorts (by y, by z, by x down, the second by x down: esi, edi), 4 a
/// text to the log (eax; 13 with a line's end), 5 a number to the log (eax; 14 with a
/// line's end), 6 `scene::clear_slot_kinds`, 7 `scene::known_kind` (eax), 8 `car_kind_11`, 9
/// `car_kind_10`, 10 `scene::load_front_end_objects`, 11 `race::set_up_car_points` (esi
/// the slot, edi the car's block), 12 `race::set_object_kinds`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The log's calls of `set_up_car_points` and `reload_objects`, inside `pusha`.
fn log(text: usize, newline: bool, g: &Regs, part: Part) -> Result<(), Refused> {
    let mut r = Regs {
        eax: game_addr(text) as u32,
        ..*g
    };
    part(if newline { 13 } else { 4 }, &mut r)
}

fn log_number(value: u32, newline: bool, g: &Regs, part: Part) -> Result<(), Refused> {
    let mut r = Regs { eax: value, ..*g };
    part(if newline { 14 } else { 5 }, &mut r)
}

/// The variables `set_up_car_points` uses, by address: the game's, or a self-test's.
pub struct CarPointVars {
    pub counts: u32,
    pub firsts: u32,
    pub points: u32,
    pub car_points: u32,
    pub scratch: u32,
}

impl CarPointVars {
    pub fn game() -> Self {
        let a = |r: usize| game_addr(r) as u32;
        Self {
            counts: a(COUNTS_RVA),
            firsts: a(FIRSTS_RVA),
            points: a(POINTS_RVA),
            car_points: a(CAR_POINTS_RVA),
            scratch: a(SCRATCH_RVA),
        }
    }
}

/// `race::set_up_car_points` (0x426B67): slot esi's points for the car's block edi (see
/// the header): with n points, those held to 0 .. 0x3D lifted by 0x50000 in y, sorted by y
/// (all), by z (the first 4), by x downwards (the first 2, and the 2 after), let down
/// again; all n (not held) as floats (each through `dword_62400C`) into the block's +0xAF0 (x), +0xBF0 (y), +0xCF0 (z);
/// moved two on from 4, the new 4 and 5 made (0, the y of 6 at 5's y); `dword_6D31BC` n +
/// 2; logged ("carnum", the car, "numcollpts", the count) and the count into +0x260. With
/// no points only the log and +0x260 (the count as it was). Refuses more than 0x40 points
/// (the original writes past the block's three lists).
pub unsafe fn set_up_car_points(
    entry: &Regs,
    v: &CarPointVars,
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count = get(v.counts + 4 * g.esi);
        g.ecx = count;
        let block = g.edi;
        if count != 0 {
            if count > MOST_CONVERTED {
                return Err(BadInput("more than 0x40 collision points").into());
            }
            if count == 1 {
                // Its first sort's test leaves by a `retn` inside `pusha` (section 16).
                return Err(
                    BadInput("one collision point: the original returns through edi").into(),
                );
            }
            let first = get(v.firsts + 4 * g.esi);
            let points = first.wrapping_mul(12).wrapping_add(get(v.points));
            let held = (count as i32).clamp(0, MOST_SORTED as i32) as u32;
            put(v.car_points, held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                put(y, get(y).wrapping_add(LIFT));
            }
            // As the original has them at each sort: ebx the points, ecx the count held,
            // eax the sort's count * 12.
            g.ebx = points;
            g.ecx = held;
            for (sort, start, n) in [(0u32, 0u32, held), (1, 0, 4), (2, 0, 2), (3, 2, 2)] {
                put(v.car_points, n);
                if n >= 2 {
                    let mut r = Regs {
                        eax: n * 12,
                        esi: points + start * 12,
                        edi: points + start * 12 + n * 12 - 12,
                        ..g
                    };
                    part(sort, &mut r)?;
                }
            }
            put(v.car_points, held);
            for n in 0..held {
                let y = points + n * 12 + 4;
                put(y, get(y).wrapping_sub(LIFT));
            }
            let scale: u32 = game_read(SCALE_RVA);
            for n in 0..count {
                let p = points + n * 12;
                for (k, list) in [XS, YS, ZS].into_iter().enumerate() {
                    // The original goes through `dword_62400C` for each `fild`.
                    put(v.scratch, get(p + 4 * k as u32));
                    put(
                        block + list + 4 * n,
                        x87::int_div(get(p + 4 * k as u32) as i32, scale),
                    );
                }
            }
            g.ebx = first.wrapping_add(count);
            g.edx = count;
            put(v.car_points, count);
            for n in (4..=0x3Du32).rev() {
                for list in [XS, YS, ZS] {
                    g.eax = get(block + list + 4 * n);
                    put(block + list + 4 * (n + 2), g.eax);
                }
            }
            g.ecx = 3;
            put(block + XS + 0x10, 0);
            put(block + YS + 0x10, 0);
            put(block + ZS + 0x10, 0);
            put(block + XS + 0x14, 0);
            put(block + YS + 0x14, get(block + YS + 0x18));
            put(block + ZS + 0x14, 0);
            put(v.car_points, count + 2);
        }
        log(rva(0x4DB9D9), false, &g, part)?;
        log_number(get(block + 0x24), false, &g, part)?;
        log(rva(0x4DB9E0), false, &g, part)?;
        log_number(get(v.car_points), true, &g, part)?;
        put(block + 0x260, get(v.car_points));
    }
    Ok(g)
}

/// `race::set_object_kinds` (0x425BB4): `scene::clear_slot_kinds`, the eight fixed kinds at
/// 0x609294, slots 1 .. 0x3F of `g_slotKinds` from `dword_6091B4`, `dword_5CCF7C` =
/// `dword_621844`; with cars, each car's three kinds (its `known_kind`, `car_kind_11`,
/// `car_kind_10` of its block's +0x24) at slots 0x50 + 0x14 * n, +11, +10; the objects
/// loaded; each car's points set up. Refuses more than the eight cars' blocks. Leaves the registers as the last call does.
pub unsafe fn set_object_kinds(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(6, &mut g)?;
        game_write(FIXED_KINDS_RVA, FIXED_KINDS);
        let kinds = game_addr(SLOT_KINDS_RVA) as u32;
        for n in 1..0x40u32 {
            g.eax = game_read(KIND_DEFAULTS_RVA + 4 * n as usize);
            put(kinds + 4 * n, g.eax);
        }
        g.ecx = 0x40;
        g.eax = game_read(KEPT_FROM_RVA);
        game_write(KEPT_RVA, g.eax);
        let cars: u32 = game_read(CARS_RVA);
        if cars == 0 {
            return Ok(g);
        }
        if cars > CAR_BLOCKS {
            return Err(BadInput("more cars than the eight blocks").into());
        }
        g.edi = FIRST_CAR_SLOT;
        for n in 0..cars {
            g.ebx = n;
            g.esi = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            for (place, slot) in [(7u32, 0u32), (8, 11), (9, 10)] {
                g.eax = get(g.esi + 0x24);
                part(place, &mut g)?;
                put(kinds + 4 * (g.edi + slot), g.eax);
            }
            g.edi += CAR_SLOT_STEP;
        }
        g.ebx = cars;
        part(10, &mut g)?;
        g.esi = FIRST_CAR_SLOT;
        for n in 0..cars {
            g.ebx = n;
            g.edi = game_read(CAR_BLOCKS_RVA + 4 * n as usize);
            let mut r = g;
            part(11, &mut r)?;
            g.esi += CAR_SLOT_STEP;
        }
        g.ebx = cars;
    }
    Ok(g)
}

/// `race::reload_objects` (0x425B2F): "LOAD OBJECTS - START" between two lines of dashes,
/// `set_object_kinds`, then "total points", "total polys" and "facemem used" with their
/// numbers, "LOAD OBJECTS - FINIS" and the dashes. Leaves the registers as
/// `set_object_kinds` does.
pub unsafe fn reload_objects(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        log(rva(0x4DB8E6), true, &g, part)?;
        log(rva(0x4DB90C), true, &g, part)?;
        part(12, &mut g)?;
        log(rva(0x4DB921), false, &g, part)?;
        log_number(game_read(rva(0x5CD680)), false, &g, part)?;
        log(rva(0x4DB92E), false, &g, part)?;
        log_number(game_read(rva(0x5CD67C)), false, &g, part)?;
        log(rva(0x4DB93A), false, &g, part)?;
        log_number(game_read(rva(0x634FA0)), true, &g, part)?;
        log(rva(0x4DB947), true, &g, part)?;
        log(rva(0x4DB95C), true, &g, part)?;
    }
    Ok(g)
}

/// The game's calls: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    unsafe {
        match place {
            0 => sort_points(BY_Y, r),
            1 => sort_points(BY_Z, r),
            2 | 3 => sort_points(BY_X_DOWN, r),
            4 | 13 => log_write(r.eax as *const u8, place == 13, 1),
            5 | 14 => log_int(r.eax as i32, place == 14, 1),
            6 => {
                crate::scene::clear_slot_kinds::clear_slot_kinds().apply(r);
            }
            7 => r.eax = crate::scene::known_kind::known_kind(r.eax),
            8 => r.eax = car_table(KIND_11_TABLE_RVA, r.eax),
            9 => r.eax = car_table(KIND_10_TABLE_RVA, r.eax),
            10 => crate::scene::load_front_end_objects::load_front_end_objects_in_game(r)?,
            11 => *r = set_up_car_points(r, &CarPointVars::game(), &mut game_part)?,
            _ => *r = set_object_kinds(r, &mut game_part)?,
        }
    }
    Ok(())
}

/// The callees by place, as IDA shows them, for the tests' copies.
pub const CALLEES: &[(usize, u32)] = &[
    (0x426BE6, 0),
    (0x426CA2, 1),
    (0x426D5E, 2),
    (0x426E16, 3),
    (0x401427, 4),
    (0x401472, 5),
    (0x426AB7, 6),
    (0x41F469, 7),
    (0x41F485, 8),
    (0x41F4A1, 9),
    (0x425CA9, 10),
    (0x426B67, 11),
    (0x425BB4, 12),
    (0x40142B, 13),
    (0x401476, 14),
];
