//! `effects::place_graded_points` (0x4BA16C): a spline record's first two grades of points
//! placed. For grade 0: the turns by the record's +0x20, +0x24, +0x28
//! (`splines::turn_record`), its +0xC points from its start (+0) turned into
//! `g_marksOrderedCoords`, bent (`effects::bend_points`), and each written plus +0xC54 ..
//! through the record's pointer for it (+0x454); for grade 1 the same with +0x2C for the y
//! turn, its start +4 and count +0x10, and no bend.
//!
//! In:  `s` a spline record (`g_marksSplinesC`)
//! Out: nothing (no caller reads the registers it leaves)
//!
//! Callers: 0x4B99D4, 0x4B9A2C. Refuses (before anything) a negative count (the loop runs
//! on), points past the record's 0x80, a null pointer among them, an angle `fsincos` cannot
//! take, and `bend_points`' refusals.

use crate::effects::bend_points::bend_points;
use crate::effects::splines::{
    MOST_POINTS, SPLINE_HEAD, SPLINE_KINDS, transform_int_points_from, turn_record,
    turnable_record, write_point,
};
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;

const POINTERS: u32 = 0x454;
const COORDS: u32 = 0x654;
/// Per grade: its start's and count's offsets and its turn's.
const GRADES: [(u32, u32, [u32; 3]); 2] =
    [(0, 0xC, [0x20, 0x24, 0x28]), (4, 0x10, [0x20, 0x2C, 0x28])];

fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a game field")
}

/// Why the Rust version refuses, if it does, worked out without writing.
fn refusal(s: u32) -> Result<(), Refused> {
    {
        for (start, count, turns) in GRADES {
            let (first, n) = (get(s + start)?, get(s + count)? as i32);
            if n < 0 {
                return Err(BadInput("a negative grade count: the loop runs on").into());
            }
            if n > 0 && first.saturating_add(n as u32) > MOST_POINTS {
                return Err(BadInput("a grade's points past the record's 0x80").into());
            }
            if crate::effects::try_any(0..n as u32, |k| {
                Ok(get(s + POINTERS + 4 * (first + k))? == 0)
            })? {
                return Err(BadInput("a null point pointer").into());
            }
            turnable_record(s, turns)?;
        }
        if SPLINE_HEAD.switch.get() != SPLINE_KINDS.chosen[0].get() && get(s + 0x30)? >= MOST_POINTS
        {
            return Err(BadInput("a spline point number of 0x80 or more").into());
        }
    }
    Ok(())
}

/// The Rust version (see the header).
pub fn place_graded_points(s: u32) -> Result<(), Refused> {
    {
        refusal(s)?;
        for (n, (start, count, turns)) in GRADES.into_iter().enumerate() {
            let (first, points) = (get(s + start)?, get(s + count)?);
            turn_record(s, turns)?;
            transform_int_points_from(s + COORDS + 12 * first, points)?;
            if n == 0 {
                bend_points(s)?;
            }
            for k in 0..points {
                write_point(s, k, k + first)?;
            }
        }
    }
    Ok(())
}
