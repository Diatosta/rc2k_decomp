//! The marks' splines' whole-number steps (converted 2026-10-21 from
//! `marks::add_face_points` 0x4B9E29, `centre_spline_points` 0x4B9F95,
//! `grade_spline_points` 0x4BA316, `order_spline_points` 0x4BA388 and `find_spline_ends`
//! 0x4BA4F1): a spline record (`MarkSpline`, 0xE90 bytes) holds up to 0x80 points of an
//! object's faces of one kind, with their ids, two values each, a centre and a grade by
//! height.

use crate::Invalid;
use crate::marks::points::{Collected, add_points};

/// The most points a record holds.
pub const MOST_POINTS: u32 = 0x80;
/// The heights the grades part at (`sub_4BA316`'s; its reading of +0x18 and +0x1C is
/// overwritten).
pub const LOW: i32 = 0x3830;
pub const HIGH: i32 = 0x6720;
/// The grades.
pub const GRADES: u32 = 3;

/// A record's point count as the loops take it: refused past 0x80 (signed; the loops
/// would run past the record), none when negative.
pub fn points(count: u32) -> Result<usize, Invalid> {
    if count as i32 > MOST_POINTS as i32 {
        return Err(Invalid("a spline of more than 0x80 points"));
    }
    Ok((count as i32).max(0) as usize)
}

/// A record's points as `add_face_points` fills them: the count, the ids and the two
/// values each came with.
pub struct FacePoints<'a> {
    pub count: u32,
    pub ids: &'a mut [u32],
    pub firsts: &'a mut [u32],
    pub seconds: &'a mut [u32],
}

impl FacePoints<'_> {
    /// `add_face_points`: the face's `points` points (signed; `point(k)` the id, asked
    /// for each looked at; `values(k)` its two values, asked for each added) added unless
    /// there already, up to 0x80. Refuses a record of more than 0x80 (unsigned).
    pub fn add_face(
        &mut self,
        points: u32,
        point: impl Fn(u32) -> u32,
        values: impl Fn(u32) -> [u32; 2],
    ) -> Result<Collected, Invalid> {
        if self.count > MOST_POINTS {
            return Err(Invalid("a spline of more than 0x80 points"));
        }
        let most = MOST_POINTS as usize;
        let (firsts, seconds) = (&mut *self.firsts, &mut *self.seconds);
        Ok(add_points(
            &mut self.count,
            &mut self.ids[..most],
            points,
            point,
            |n, k| [firsts[n], seconds[n]] = values(k),
        ))
    }
}

/// `centre_spline_points`: each point less the centre (wrapping).
pub fn centre(points: &mut [[u32; 3]], centre: [u32; 3]) {
    for p in points.iter_mut() {
        *p = std::array::from_fn(|c| p[c].wrapping_sub(centre[c]));
    }
}

/// `grade_spline_points`: a height's grade: 0 below `LOW`, 1 below `HIGH`, else 2.
pub fn grade(height: i32) -> u32 {
    if height < LOW {
        0
    } else if height < HIGH {
        1
    } else {
        2
    }
}

/// What `order_spline_points` works out: each grade's start and count (the record's +0
/// .. +8, +0xC .. +0x14), how many it placed, and the last point it took.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Order {
    pub starts: [u32; 3],
    pub counts: [u32; 3],
    pub placed: u32,
    pub last_taken: Option<u32>,
}

/// `order_spline_points`: the points' ids and coordinates put in grade order (each
/// grade's the last first) into `ordered_ids` and `ordered_coords`; a point of no grade
/// (past 2) is left out.
pub fn order(
    grades: &[u32],
    ids: &[u32],
    coords: &[[u32; 3]],
    ordered_ids: &mut [u32],
    ordered_coords: &mut [[u32; 3]],
) -> Order {
    let mut o = Order::default();
    for g in 0..GRADES {
        for k in (0..grades.len()).rev() {
            if grades[k] != g {
                continue;
            }
            ordered_ids[o.placed as usize] = ids[k];
            ordered_coords[o.placed as usize] = coords[k];
            o.placed += 1;
            o.last_taken = Some(k as u32);
        }
        o.counts[g as usize] = o.placed - o.starts[g as usize];
        if g < 2 {
            o.starts[g as usize + 1] = o.placed;
        }
    }
    o
}

/// What `find_spline_ends` finds in one run of points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct End {
    /// The point found (from the run's start), and its value.
    pub at: u32,
    pub value: u32,
    /// The last point read, if any.
    pub last: Option<u32>,
}

/// The run's limits (start, count) as `find_spline_ends` takes them: refused when
/// negative or past 0x80 points.
pub fn run_fits(start: u32, count: u32) -> Result<(), Invalid> {
    let (a, c) = (start as i32, count as i32);
    if a < 0 || c < 0 || a as i64 + c as i64 > MOST_POINTS as i64 {
        return Err(Invalid("a spline's grade past its points"));
    }
    Ok(())
}

/// `find_spline_ends`' search: of `count` points from `start`, the last with the highest
/// value (`highest`) or the lowest (`value(n)`, signed), starting from 0x80000001 or
/// 0x7FFFFFFF.
pub fn find_end(start: u32, count: u32, highest: bool, value: impl Fn(u32) -> u32) -> End {
    let mut end = End {
        at: 0,
        value: if highest { 0x8000_0001 } else { 0x7FFF_FFFF },
        last: None,
    };
    for k in 0..count {
        let n = k + start;
        let v = value(n);
        end.last = Some(n);
        let take = if highest {
            end.value as i32 <= v as i32
        } else {
            end.value as i32 >= v as i32
        };
        if take {
            end.at = k;
            end.value = v;
        }
    }
    end.at = end.at.wrapping_add(start);
    end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        assert!(points(0x81).is_err());
        assert_eq!((points(0x80), points(u32::MAX)), (Ok(0x80), Ok(0)));
        let mut p = [[5, 0, 1], [0, 0, 0]];
        centre(&mut p, [1, 2, 3]);
        assert_eq!(
            p,
            [
                [4, u32::MAX - 1, u32::MAX - 1],
                [u32::MAX, u32::MAX - 1, u32::MAX - 2]
            ]
        );
        assert_eq!(
            [i32::MIN, LOW - 1, LOW, HIGH - 1, HIGH].map(grade),
            [0, 0, 1, 1, 2]
        );
    }

    #[test]
    fn faces() {
        let (mut ids, mut firsts, mut seconds) = ([0u32; 0x80], [0u32; 0x80], [0u32; 0x80]);
        let mut f = FacePoints {
            count: 0,
            ids: &mut ids,
            firsts: &mut firsts,
            seconds: &mut seconds,
        };
        let c = f
            .add_face(3, |k| [8, 9, 8][k as usize], |k| [k, 10 + k])
            .unwrap();
        assert_eq!((f.count, c.stopped, c.last_point), (2, Some(0), Some(8)));
        assert_eq!(
            (&ids[..2], &firsts[..2], &seconds[..2]),
            (&[8, 9][..], &[0, 1][..], &[10, 11][..])
        );
        let mut f = FacePoints {
            count: 0x81,
            ids: &mut ids,
            firsts: &mut firsts,
            seconds: &mut seconds,
        };
        assert!(f.add_face(1, |_| 0, |_| [0, 0]).is_err());
    }

    #[test]
    fn orders() {
        let grades = [1, 0, 2, 0, 7];
        let ids = [10, 11, 12, 13, 14];
        let coords = ids.map(|i| [i, i, i]);
        let (mut oi, mut oc) = ([0u32; 5], [[0u32; 3]; 5]);
        let o = order(&grades, &ids, &coords, &mut oi, &mut oc);
        assert_eq!(
            o,
            Order {
                starts: [0, 2, 3],
                counts: [2, 1, 1],
                placed: 4,
                last_taken: Some(2)
            }
        );
        assert_eq!(&oi[..4], &[13, 11, 10, 12]);
        assert_eq!(oc[3], [12, 12, 12]);
    }

    #[test]
    fn ends() {
        assert!(run_fits(u32::MAX, 0).is_err());
        assert!(run_fits(0x7F, 2).is_err());
        assert!(run_fits(0x7FFF_FFFF, 0x7FFF_FFFF).is_err());
        assert!(run_fits(0x10, 0x70).is_ok());
        let v = [3u32, 9, 9, 1, (-5i32) as u32];
        let e = find_end(1, 4, true, |n| v[n as usize]);
        assert_eq!(
            e,
            End {
                at: 2,
                value: 9,
                last: Some(4)
            }
        );
        let e = find_end(0, 5, false, |n| v[n as usize]);
        assert_eq!(
            e,
            End {
                at: 4,
                value: (-5i32) as u32,
                last: Some(4)
            }
        );
        let e = find_end(7, 0, false, |_| unreachable!());
        assert_eq!(
            e,
            End {
                at: 7,
                value: 0x7FFF_FFFF,
                last: None
            }
        );
    }
}
