//! `effects::place_graded_points`: a spline record's first two grades of points placed. For grade
//! 0: the turns by the record's +0x20, +0x24, +0x28 (`splines::turn_record`), its +0xC points from
//! its start (+0) turned into the ordered coordinates, bent (`effects::bend_points`), and each
//! written plus +0xC54 .. through the record's pointer for it (+0x454); for grade 1 the same with
//! +0x2C for the y turn, its start +4 and count +0x10, and no bend.
//!
//! Refuses (before anything) a negative count (the loop runs on), points past the record's 0x80, a
//! null pointer among them, an angle with no sine and cosine, and `bend_points`' refusals.

use crate::effects::bend_points::bend_points;
use crate::effects::paths::Paths;
use crate::effects::splines::{
    MOST_POINTS, Spline, transform_int_points_from, turn_record, turnable_record, write_point,
};
use crate::errors::{BadInput, Refused};

const POINTERS: u32 = 0x454;
/// Per grade: its start's and count's offsets and its turn's.
const GRADES: [(u32, u32, [u32; 3]); 2] =
    [(0, 0xC, [0x20, 0x24, 0x28]), (4, 0x10, [0x20, 0x2C, 0x28])];

/// Why the function refuses, if it does, worked out without writing.
fn refusal(paths: &Paths, s: Spline) -> Result<(), Refused> {
    {
        for (start, count, turns) in GRADES {
            let (first, n) = (s.get(paths, start)?, s.get(paths, count)? as i32);
            if n < 0 {
                return Err(BadInput("a negative grade count: the loop runs on").into());
            }
            if n > 0 && first.saturating_add(n as u32) > MOST_POINTS {
                return Err(BadInput("a grade's points past the record's 0x80").into());
            }
            if crate::effects::try_any(0..n as u32, |k| {
                Ok(s.get(paths, POINTERS + 4 * (first + k))? == 0)
            })? {
                return Err(BadInput("a null point pointer").into());
            }
            turnable_record(paths, s, turns)?;
        }
        if paths.spline_head.switch.get() != paths.spline_kinds.chosen[0].get()
            && s.get(paths, 0x30)? >= MOST_POINTS
        {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
    }
    Ok(())
}

pub fn place_graded_points(
    object_points: &crate::scene::object_points::ObjectPoints,
    paths: &Paths,
    s: Spline,
) -> Result<(), Refused> {
    {
        refusal(paths, s)?;
        for (n, (start, count, turns)) in GRADES.into_iter().enumerate() {
            let (first, points) = (s.get(paths, start)?, s.get(paths, count)?);
            turn_record(paths, s, turns)?;
            transform_int_points_from(paths, s, first, points)?;
            if n == 0 {
                bend_points(paths, s)?;
            }
            for k in 0..points {
                write_point(object_points, paths, s, k, k + first)?;
            }
        }
    }
    Ok(())
}
