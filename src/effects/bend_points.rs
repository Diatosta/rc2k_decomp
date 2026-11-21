//! `effects::bend_points`: unless the grade flag is the chosen spline, the first record's
//! point count of ordered points bent: A the record's point (the one its point number names), C
//! (the record's height made 0x2EE0, or -16000 unless `SplineHead.switch` is 1, with its next two
//! values), B (C's x, A's y, C's z); D their blend by the splines' weights
//! (`effects::blend_points`), rounded (z first); each point less A plus D.
//!
//! Refuses (before anything) a point number of 0x80 or more and a count above 0x80.

use crate::effects::blend_points::blend_points;
use crate::effects::paths_store::Paths;
use crate::effects::splines::{MOST_POINTS, Spline};
use crate::errors::{BadInput, Refused};
use crate::math::x87::ext_to_int;

pub(super) fn bend_points(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        let switch: u32 = paths.spline_head.borrow().switch;
        let chosen = paths.spline_kinds.borrow().chosen[0];
        if switch == chosen {
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
        let height: u32 = if switch == 1 { 0x2EE0 } else { 0xFFFF_C180 };
        s.put(paths, 0x44, height)?;
        let c = [height, s.get(paths, 0x48)?, s.get(paths, 0x4C)?];
        let b = [c[0], a[1], c[2]];
        let made = paths.spline_head.borrow().made;
        let v = blend_points(paths, a, b, c, made as u8);
        let d = [ext_to_int(v[0]), ext_to_int(v[1]), ext_to_int(v[2])];
        let mut work = paths.spline_work.borrow_mut();
        for q in work
            .ordered_coords
            .chunks_mut(3)
            .take(count.max(0) as usize)
        {
            for k in 0..3 {
                q[k] = q[k].wrapping_sub(a[k]).wrapping_add(d[k] as u32);
            }
        }
    }
    Ok(())
}
