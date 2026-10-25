//! `marks::place_spline` (0x4B98FD): spline eax (of the 8) placed: `g_marksSplineOffset`
//! its number times 0xE90; the settings' +0x26DC by `flt_788F19` (unrounded) into its A, B
//! and C records' +0x24, and that by `flt_788F1D` into D's; A and D by
//! `marks::place_shape_points`; B's +0x2C its +0x24 by `flt_788F29` when under
//! `flt_788F21`, else by `flt_788F25`, `dword_7695CA` 0, B by `marks::place_graded_points`;
//! C's +0x2C its +0x24 by `flt_788F35` when above `flt_788F2D` (the status 0), else by
//! `flt_788F31`, `dword_7695CA` 1, C by `place_graded_points`. Each compare's C0 .. C2 into
//! `g_fpuStatus`.
//!
//! In:  `n` the spline (`g_marksSettings`)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `sub_4B978A`. Refuses null settings, a spline number of 8 or more (before
//! anything), and its steps' refusals (all it may write is put back first).

use crate::errors::{BadInput, Refused};
use crate::marks::place_graded_points::place_graded_points;
use crate::marks::splines::{OFFSET_RVA, SPLINE_LEN, TABLES, place_shape_points as place_shape};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87;
use crate::scene::vector3::STATUS_RVA;

const SPLINES: u32 = 8;
const SWITCH_RVA: Slot = Slot::at(0x7695CA); // dword_7695CA
/// flt_788F19 .. flt_788F35.
const K: [Slot; 8] = [
    Slot::at(0x788F19),
    Slot::at(0x788F1D),
    Slot::at(0x788F21),
    Slot::at(0x788F25),
    Slot::at(0x788F29),
    Slot::at(0x788F2D),
    Slot::at(0x788F31),
    Slot::at(0x788F35),
];

/// The four records of spline `n`: A, B, C, D.
fn records(n: u32) -> [u32; 4] {
    TABLES.map(|t| game_addr(t) as u32 + n * SPLINE_LEN)
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

/// The Rust version (see the header).
pub unsafe fn place_spline(n: u32) -> Result<(), Refused> {
    let settings: u32 = crate::marks::setup::SETTINGS.pointer.get();
    if settings == 0 {
        return Err(BadInput("no marks settings").into());
    }
    if n >= SPLINES {
        return Err(BadInput("a spline number of 8 or more").into());
    }
    unsafe {
        let k = K.map(|r| game_read::<u32>(r));
        game_write(OFFSET_RVA, n * SPLINE_LEN);
        let [a, b, c, d] = records(n);
        let (abc, dsize) = sizes(get(settings + 0x26DC), k[0], k[1]);
        for r in [a, b, c] {
            put(r + 0x24, abc);
        }
        put(d + 0x24, dsize);
        for r in [a, d] {
            place_shape(r)?;
        }
        let status = x87::compare_status(get(b + 0x24), k[2]);
        game_write(STATUS_RVA, status as u32);
        let f = if status == 0x100 { k[4] } else { k[3] };
        put(b + 0x2C, x87::mul(get(b + 0x24), f));
        game_write(SWITCH_RVA, 0u32);
        place_graded_points(b)?;
        let status = x87::compare_status(get(c + 0x24), k[5]);
        game_write(STATUS_RVA, status as u32);
        let f = if status == 0 { k[7] } else { k[6] };
        put(c + 0x2C, x87::mul(get(c + 0x24), f));
        game_write(SWITCH_RVA, 1u32);
        place_graded_points(c)?;
    }
    Ok(())
}
