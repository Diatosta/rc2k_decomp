//! `marks::bend_points` (0x4BA646): unless `dword_7695CA` is `g_marksSplineChosen`, the
//! first record ebp +0xC ordered points (`g_marksOrderedCoords`) bent: A the record's point
//! +0x30 (into `dword_76A1D2` ..), C (+0x44 made 0x2EE0, or -16000 unless `dword_7695CA` is
//! 1; +0x48, +0x4C) into `dword_76A1EA` .., B (C's x, A's y, C's z) into `dword_76A1DE` ..;
//! D their blend by `g_marksSplinesMade`'s weights (`marks::blend_points`), rounded
//! (`fistp`, z first) into `dword_76A1F6` ..; each point less A plus D.
//!
//! In:  `s` a spline record
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `sub_4BA16C`. Refuses (before anything) a point number of 0x80 or more and a
//! count above 0x80. Self-test: a record of the test's (`TestSpline`), several points,
//! counts and switches; compared on all it writes and the registers.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::marks::blend_points::blend_points;
use crate::marks::splines::{CHOSEN_RVA, MADE_RVA, MOST_POINTS, ORDERED_COORDS_RVA};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::math::x87_run::ext_to_int;

const SWITCH_RVA: Slot = Slot::at(0x7695CA); // dword_7695CA
const A_RVA: Slot = Slot::at(0x76A1D2);
const B_RVA: Slot = Slot::at(0x76A1DE);
const C_RVA: Slot = Slot::at(0x76A1EA);
const D_RVA: Slot = Slot::at(0x76A1F6);
unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust version (see the header).
pub unsafe fn bend_points(s: u32) -> Result<(), Refused> {
    unsafe {
        let switch: u32 = game_read(SWITCH_RVA);
        if switch == game_read::<u32>(CHOSEN_RVA) {
            return Ok(());
        }
        let point = get(s + 0x30);
        let count = get(s + 0xC) as i32;
        if point >= MOST_POINTS {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
        if count > MOST_POINTS as i32 {
            return Err(BadInput("more than 0x80 points to bend").into());
        }
        let p = s + 0x654 + point * 12;
        let a = [get(p), get(p + 4), get(p + 8)];
        game_write(A_RVA, a);
        let height: u32 = if switch == 1 { 0x2EE0 } else { 0xFFFF_C180 };
        ((s + 0x44) as *mut u32).write_unaligned(height);
        let c = [height, get(s + 0x48), get(s + 0x4C)];
        game_write(C_RVA, c);
        let b = [c[0], a[1], c[2]];
        game_write(B_RVA, b);
        let at = |r: Slot| game_addr(r) as u32;
        let v = blend_points(
            at(A_RVA),
            at(B_RVA),
            at(C_RVA),
            game_read::<u32>(MADE_RVA) as u8,
        );
        let d = [ext_to_int(v[0]), ext_to_int(v[1]), ext_to_int(v[2])];
        game_write(D_RVA, d);
        let mut q_at = at(ORDERED_COORDS_RVA);
        for _ in 0..count.max(0) {
            let q = [get(q_at), get(q_at + 4), get(q_at + 8)];
            for k in 0..3 {
                ((q_at + 4 * k as u32) as *mut u32)
                    .write_unaligned(q[k].wrapping_sub(a[k]).wrapping_add(d[k] as u32));
            }
            q_at += 12;
        }
    }
    Ok(())
}
