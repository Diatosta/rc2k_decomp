//! `marks::place_spline` (0x4B98FD): spline eax (of the 8) placed: `g_marksSplineOffset`
//! its number times 0xE90; the settings' +0x26DC by `flt_788F19` (unrounded) into its A, B
//! and C records' +0x24, and that by `flt_788F1D` into D's; A and D by
//! `marks::place_shape_points`; B's +0x2C its +0x24 by `flt_788F29` when under
//! `flt_788F21`, else by `flt_788F25`, `dword_7695CA` 0, B by `marks::place_graded_points`;
//! C's +0x2C its +0x24 by `flt_788F35` when above `flt_788F2D` (the status 0), else by
//! `flt_788F31`, `dword_7695CA` 1, C by `place_graded_points`. Each compare's C0 .. C2 into
//! `g_fpuStatus`.
//!
//! In:  eax the spline (`g_marksSettings`)
//! Out: ebp C's record, the rest as its last `place_graded_points` leaves them
//!
//! Caller: `sub_4B978A`. Refuses null settings, a spline number of 8 or more (before
//! anything), and its steps' refusals (all it may write is put back first). Self-test: the
//! game's settings and spline tables (skipped without settings), spline numbers, sizes and
//! switches by lot; compared on all it may write.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::place_graded_points::{self, place_graded_points};
use crate::marks::place_shape_points;
use crate::marks::splines::{OFFSET_RVA, SPLINE_LEN, TABLES, place_shape_points as place_shape};
use crate::math::x87;
use crate::scene::vector3::STATUS_RVA;
use crate::service::harness::{Region, at};

const SPLINES: u32 = 8;
const SWITCH_RVA: usize = rva(0x7695CA); // dword_7695CA
/// flt_788F19 .. flt_788F35.
const K: [usize; 8] = [
    rva(0x788F19),
    rva(0x788F1D),
    rva(0x788F21),
    rva(0x788F25),
    rva(0x788F29),
    rva(0x788F2D),
    rva(0x788F31),
    rva(0x788F35),
];

/// The four records of spline `n`: A, B, C, D.
fn records(n: u32) -> [u32; 4] {
    TABLES.map(|t| game_addr(rva(t)) as u32 + n * SPLINE_LEN)
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `fld a; fmul k; fst` each of three (the first), `fmul k2; fstp` (the second).
fn sizes(a: u32, k: u32, k2: u32) -> (u32, u32) {
    let abc = x87::mul(a, k);
    let d = x87::mul2(a, k, k2);
    (abc, d)
}

/// What it may write, for spline `n`.
pub fn regions(n: u32) -> Vec<Region> {
    let mut r: Vec<Region> = [OFFSET_RVA, SWITCH_RVA, STATUS_RVA].map(at).to_vec();
    if n < SPLINES {
        let [a, b, c, d] = records(n);
        r.extend(place_shape_points::regions(a));
        r.extend(place_shape_points::regions(d));
        r.extend(place_graded_points::regions(b));
        r.extend(place_graded_points::regions(c));
    }
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn place_spline(entry: &Regs) -> Result<Regs, Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    if entry.eax >= SPLINES {
        return Err(BadInput("a spline number of 8 or more").into());
    }

    unsafe { run(entry, settings) }
}

unsafe fn run(entry: &Regs, settings: u32) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.esi = settings;
        let k = K.map(|r| game_read::<u32>(r));
        let n = g.eax;
        g.eax = n * SPLINE_LEN;
        game_write(OFFSET_RVA, g.eax);
        let [a, b, c, d] = records(n);
        let (abc, dsize) = sizes(get(settings + 0x26DC), k[0], k[1]);
        for r in [a, b, c] {
            put(r + 0x24, abc);
        }
        put(d + 0x24, dsize);
        for r in [a, d] {
            g.ebp = r;
            g = place_shape(&g)?;
        }
        g.ebp = b;
        let status = x87::compare_status(get(b + 0x24), k[2]);
        game_write(STATUS_RVA, status as u32);
        let f = if status == 0x100 { k[4] } else { k[3] };
        put(b + 0x2C, x87::mul(get(b + 0x24), f));
        game_write(SWITCH_RVA, 0u32);
        g = place_graded_points(&g)?;
        g.ebp = c;
        let status = x87::compare_status(get(c + 0x24), k[5]);
        game_write(STATUS_RVA, status as u32);
        let f = if status == 0 { k[7] } else { k[6] };
        put(c + 0x2C, x87::mul(get(c + 0x24), f));
        game_write(SWITCH_RVA, 1u32);
        g = place_graded_points(&g)?;
    }
    Ok(g)
}
