//! `marks::make_profiles` (0x4C3B01), the body of `init_marks`' step 3 (`sub_4C4E1C` is
//! only its call; IDA, 2026-10-16): two rings of 0x100 indexes, once the 13 profile
//! tables (0x100 whole numbers and their 0x100 differences each), then the corners from
//! the course and `allocate_links`, `reset_settings_blocks`.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const RINGS_RVA: usize = rva(0x8DD061); // dword_8DD061, dword_8DD461, then dword_8DD861
pub const DONE_RVA: usize = rva(0x91FF3E); // dword_91FF3E: 0, or where to go on
pub const RESUME_RVA: usize = rva(0x4C3EA6); // loc_4C3EA6
const RAMPS_RVA: usize = rva(0x9152F6); // off_9152F6: 3 tables
const SINES_RVA: usize = rva(0x915302); // off_915302: 8 tables
const CURVES_RVA: usize = rva(0x915322); // off_915322: 2 tables
const RAMP_STEPS_RVA: usize = rva(0x91FF42); // flt_91FF42, 256.0
const CURVE_STEPS_RVA: usize = rva(0x91FF46); // flt_91FF46, 256.0
/// The profiles' work, `flt_8EB871` .. `flt_8EB899`.
pub const WORK_RVA: usize = rva(0x8EB871);
pub const COURSE_INFO_RVA: usize = rva(0x5C6FA0); // g_courseInfo
pub const COURSE_CORNERS: u32 = 0x1C_2000;
pub const CORNERS_LEN: usize = 0x8000;

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) };
}

unsafe fn work(n: usize, v: u32) {
    unsafe { game_write(WORK_RVA + 4 * n, v) };
}

/// A table's second half: each the next less this (whole numbers, so the last reads the
/// second half's first, already a float), the first half made floats.
unsafe fn differences(table: u32) {
    unsafe {
        for n in 0..0x100 {
            let p = table + 0x400 + 4 * n;
            put(
                p,
                x87::int_less_int(get(p - 0x3FC) as i32, get(p - 0x400) as i32),
            );
            put(p - 0x400, x87::from_int(get(p - 0x400) as i32));
        }
    }
}

/// The once-only tables.
unsafe fn make_tables() {
    unsafe {
        for k in 0..3 {
            let t: u32 = game_read(RAMPS_RVA + 4 * k);
            let step =
                x87::int_less_int_over(get(t + 4) as i32, get(t) as i32, game_read(RAMP_STEPS_RVA));
            work(2, step);
            let mut v = x87::from_int(get(t) as i32);
            work(0, v);
            for n in 0..0x100 {
                put(t + 4 * n, x87::to_int(v) as u32);
                v = x87::add(v, step);
                work(0, v);
            }
            differences(t);
        }
        for k in 0..8 {
            let t: u32 = game_read(SINES_RVA + 4 * k);
            let p = [0, 1, 2, 3].map(|i| x87::loaded(get(t + 4 * i)));
            for (i, &v) in p.iter().enumerate() {
                work(3 + i, v);
            }
            let mut a = p[0];
            for n in 0..0x100 {
                put(t + 4 * n, x87::sine_times_plus_to_int(a, p[2], p[1]) as u32);
                a = x87::add(a, p[3]);
                work(3, a);
            }
            differences(t);
        }
        for k in 0..2 {
            let t: u32 = game_read(CURVES_RVA + 4 * k);
            let mut x = x87::loaded(get(t));
            work(7, x);
            let step = x87::sub_div(get(t + 4), get(t), game_read(CURVE_STEPS_RVA));
            work(8, step);
            let (scale, top) = (x87::loaded(get(t + 8)), x87::loaded(get(t + 12)));
            work(9, scale);
            work(10, top);
            for n in 0..0x100 {
                put(
                    t + 4 * n,
                    x87::negated_square_times_plus_to_int(x, scale, top) as u32,
                );
                x = x87::add(x, step);
                work(7, x);
            }
            differences(t);
        }
    }
}

/// `marks::make_profiles` (0x4C3B01): `dword_8DD861` 0, the rings' before and after (of
/// 0x100); unless done (`dword_91FF3E` = `resume`): the 3 ramps (start to end over 256
/// steps), 8 sines (base + amplitude sin, the angle stepped), 2 curves (top - x² scale),
/// each table's differences after it (a fourth list is of no tables), the mark `resume`;
/// then 0x8000 bytes of corners from `g_courseInfo` + 0x1C2000 (twice, as the original),
/// `allocate_links` (place 2) and `reset_settings_blocks` (place 3). Refuses a mark
/// that is neither 0 nor `resume` (the original jumps to it). `resume` is `loc_4C3EA6` in
/// the game, its place in a test's copy.
pub unsafe fn make_profiles(
    entry: &Regs,
    resume: u32,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let done: u32 = game_read(DONE_RVA);
        if done != 0 && done != resume {
            return Err(BadInput("the profiles' mark is neither 0 nor loc_4C3EA6").into());
        }
        let rings = game_addr(RINGS_RVA) as u32;
        put(rings + 0x800, 0);
        for n in 0..0x100u32 {
            g.ebx = n.wrapping_sub(1) & 0xFF;
            g.ecx = (n + 1) & 0xFF;
            put(rings + 4 * n, g.ebx);
            put(rings + 0x400 + 4 * n, g.ecx);
        }
        g.eax = 0x100;
        if done == 0 {
            make_tables();
            g.esi = game_read::<u32>(CURVES_RVA + 4) + 0x800;
            g.eax = game_addr(DONE_RVA) as u32;
            game_write(DONE_RVA, resume);
        }
        let from = game_read::<u32>(COURSE_INFO_RVA).wrapping_add(COURSE_CORNERS);
        let to = game_addr(crate::marks::blocks::CORNERS_RVA) as u32;
        std::ptr::copy(from as *const u8, to as *mut u8, CORNERS_LEN);
        g.esi = from + CORNERS_LEN as u32;
        g.edi = to + CORNERS_LEN as u32;
        g.ecx = 0;
        part(2, &mut g)?;
        part(3, &mut g)?;
    }
    Ok(g)
}
