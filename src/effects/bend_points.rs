//! `effects::bend_points` (0x4BA646): unless `dword_7695CA` is `g_marksSplineChosen`, the
//! first record ebp +0xC ordered points (`g_marksOrderedCoords`) bent: A the record's point
//! +0x30 (into `dword_76A1D2` ..), C (+0x44 made 0x2EE0, or -16000 unless `dword_7695CA` is
//! 1; +0x48, +0x4C) into `dword_76A1EA` .., B (C's x, A's y, C's z) into `dword_76A1DE` ..;
//! D their blend by `g_marksSplinesMade`'s weights (`effects::blend_points`), rounded
//! (`fistp`, z first) into `dword_76A1F6` ..; each point less A plus D.
//!
//! Takes: `s` a spline record
//!
//! Caller: `sub_4BA16C`. Refuses (before anything) a point number of 0x80 or more and a
//! count above 0x80.

use crate::effects::blend_points::blend_points;
use crate::effects::paths::Paths;
use crate::effects::splines::{MOST_POINTS, Spline};
use crate::errors::{BadInput, Refused};
use crate::math::x87_run::ext_to_int;
use crate::session::write_first;

/// The Rust version (see the header).
pub fn bend_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let switch: u32 = paths.spline_head.switch.get();
        if switch == paths.spline_kinds.chosen[0].get() {
            return Ok(());
        }
        let point = s.get(paths, 0x30)?;
        let count = s.get(paths, 0xC)? as i32;
        if point >= MOST_POINTS {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
        if count > MOST_POINTS as i32 {
            return Err(BadInput("more than 0x80 points to bend").into());
        }
        let p = 0x654 + point * 12;
        let a = [s.get(paths, p)?, s.get(paths, p + 4)?, s.get(paths, p + 8)?];
        write_first(&paths.spline_work.bend[0..3], &a);
        let height: u32 = if switch == 1 { 0x2EE0 } else { 0xFFFF_C180 };
        s.put(paths, 0x44, height)?;
        let c = [height, s.get(paths, 0x48)?, s.get(paths, 0x4C)?];
        write_first(&paths.spline_work.bend[6..9], &c);
        let b = [c[0], a[1], c[2]];
        write_first(&paths.spline_work.bend[3..6], &b);
        let v = blend_points(paths, a, b, c, paths.spline_head.made.get() as u8);
        let d = [ext_to_int(v[0]), ext_to_int(v[1]), ext_to_int(v[2])];
        write_first(&paths.spline_work.bend[9..12], &d.map(|v| v as u32));
        let ordered = &paths.spline_work.ordered_coords;
        for q in ordered.chunks(3).take(count.max(0) as usize) {
            for k in 0..3 {
                q[k].set(q[k].get().wrapping_sub(a[k]).wrapping_add(d[k] as u32));
            }
        }
    }
    Ok(())
}
