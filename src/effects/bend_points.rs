//! `effects::bend_points` (0x4BA646): unless `dword_7695CA` is `g_marksSplineChosen`, the
//! first record ebp +0xC ordered points (`g_marksOrderedCoords`) bent: A the record's point
//! +0x30 (into `dword_76A1D2` ..), C (+0x44 made 0x2EE0, or -16000 unless `dword_7695CA` is
//! 1; +0x48, +0x4C) into `dword_76A1EA` .., B (C's x, A's y, C's z) into `dword_76A1DE` ..;
//! D their blend by `g_marksSplinesMade`'s weights (`effects::blend_points`), rounded
//! (`fistp`, z first) into `dword_76A1F6` ..; each point less A plus D.
//!
//! In:  `s` a spline record
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Caller: `sub_4BA16C`. Refuses (before anything) a point number of 0x80 or more and a
//! count above 0x80.

use crate::effects::blend_points::blend_points;
use crate::effects::splines::{MOST_POINTS, SPLINE_HEAD, SPLINE_KINDS, SPLINE_WORK};
use crate::errors::{BadInput, Refused};
use crate::math::x87_run::ext_to_int;
use crate::pointers::GamePtr;
use crate::session::write_first;

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// The Rust version (see the header).
pub fn bend_points(s: u32) -> Result<(), Refused> {
    {
        let switch: u32 = SPLINE_HEAD.switch.get();
        if switch == SPLINE_KINDS.chosen[0].get() {
            return Ok(());
        }
        let point = get(s + 0x30)?;
        let count = get(s + 0xC)? as i32;
        if point >= MOST_POINTS {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
        if count > MOST_POINTS as i32 {
            return Err(BadInput("more than 0x80 points to bend").into());
        }
        let p = s + 0x654 + point * 12;
        let a = [get(p)?, get(p + 4)?, get(p + 8)?];
        write_first(&SPLINE_WORK.bend[0..3], &a);
        let height: u32 = if switch == 1 { 0x2EE0 } else { 0xFFFF_C180 };
        GamePtr::new(s + 0x44).write::<u32>(height, "a mark's field")?;
        let c = [height, get(s + 0x48)?, get(s + 0x4C)?];
        write_first(&SPLINE_WORK.bend[6..9], &c);
        let b = [c[0], a[1], c[2]];
        write_first(&SPLINE_WORK.bend[3..6], &b);
        let at = |n: usize| SPLINE_WORK.bend[n].addr();
        let v = blend_points(at(0), at(3), at(6), SPLINE_HEAD.made.get() as u8)?;
        let d = [ext_to_int(v[0]), ext_to_int(v[1]), ext_to_int(v[2])];
        write_first(&SPLINE_WORK.bend[9..12], &d.map(|v| v as u32));
        let mut q_at = SPLINE_WORK.ordered_coords[0].addr();
        for _ in 0..count.max(0) {
            let q = [get(q_at)?, get(q_at + 4)?, get(q_at + 8)?];
            for k in 0..3 {
                GamePtr::new(q_at + 4 * k as u32).write::<u32>(
                    q[k].wrapping_sub(a[k]).wrapping_add(d[k] as u32),
                    "a mark's field",
                )?;
            }
            q_at += 12;
        }
    }
    Ok(())
}
