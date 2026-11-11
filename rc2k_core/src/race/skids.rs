//! The skidmarks' points (from `race::add_skid_point` 0x45860C): a
//! ring of `POINTS` points (`g_skidPoints`, each the mark's left and right edges) linked
//! in strips (`g_skidLinks`: the next, the previous, the angle, the car's bit, words).
//! The angle between two places and the edges' offset for an angle are the game's turns,
//! asked for through closures.

use crate::Invalid;

/// The points in the ring.
pub const POINTS: u32 = 0x400;
/// A link to no point.
pub const NONE: u16 = 0xFFFF;
/// Half a mark's width (the edges' offset before it is turned).
pub const HALF_WIDTH: i32 = 0x1555;

/// A point: its left edge's x, y, z, then its right edge's.
pub type Point = [u32; 6];

/// A point's links (`g_skidLinks`, 8 bytes, laid out as the game's).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Link {
    pub next: u16,
    pub previous: u16,
    pub angle: u16,
    pub car: u16,
}

/// A point's middle: (left - right) >> 1 (signed) + right, for x, y and z.
pub fn middle(p: &Point) -> [u32; 3] {
    std::array::from_fn(|i| {
        ((p[i].wrapping_sub(p[3 + i]) as i32 >> 1) as u32).wrapping_add(p[3 + i])
    })
}

/// A point's edges across `centre` by `offset` (x, z): the left less it, the right plus
/// it, y the centre's.
pub fn edges(centre: [u32; 3], offset: (u32, u32)) -> Point {
    [
        centre[0].wrapping_sub(offset.0),
        centre[1],
        centre[2].wrapping_sub(offset.1),
        centre[0].wrapping_add(offset.0),
        centre[1],
        centre[2].wrapping_add(offset.1),
    ]
}

/// Halfway from the last point's angle to the new one (its low word, the turn's sign
/// kept), the new one's high word.
pub fn between(angle: u32, last: u16) -> u32 {
    let half = ((angle as u16).wrapping_sub(last) as i16 >> 1) as u16;
    (angle & 0xFFFF_0000) | half.wrapping_add(last) as u32
}

/// The ring's next point after `n`.
pub fn next_point(n: u32) -> u32 {
    let next = n.wrapping_add(1);
    if next == POINTS { 0 } else { next }
}

/// The ring's points and their links.
pub struct Ring<'a> {
    pub points: &'a mut [Point],
    pub links: &'a mut [Link],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Added {
    /// The angle from the last point (`skid_angle`; 0 without one).
    pub angle: u32,
    /// The edges' offset last worked out (`skid_offset_x`, `skid_offset_z`).
    pub offset: (u32, u32),
    /// The last point's middle (`skid_from` ..), with a last point.
    pub from: Option<[u32; 3]>,
    /// The halfway angle and the last point's middle (`skid_turned_angle`, `skid_turned` ..),
    /// when the last point is not the strip's first.
    pub turned: Option<(u32, [u32; 3])>,
    /// Whether the last point was given new edges (the registers then hold them).
    pub last_moved: bool,
    /// The last edges written: the right edge.
    pub right: [u32; 3],
    /// The new point's next link as read (without the last point moved).
    pub next_read: u16,
}

/// `race::add_skid_point`: point `n` (the ring's next) across the wheel's `place` at the
/// angle from the `last` point's middle (`angle_of(dx, dz)`; 0 without a last point),
/// its old links cut, linked after the last, marked with `car`; the last point turned to
/// the new angle (its own edges moved when it starts its strip, else made again across
/// its middle halfway between its angle and the new one); `offset_of(angle, y)` the edges'
/// offset. Refuses (before anything) a point or a link outside the ring.
pub fn add_skid_point(
    ring: Ring,
    n: u32,
    last: u32,
    place: [u32; 3],
    car: u16,
    mut angle_of: impl FnMut(u32, u32) -> u32,
    mut offset_of: impl FnMut(u32, u32) -> (u32, u32),
) -> Result<Added, Invalid> {
    let Ring { points, links } = ring;
    let ring = |i: u32| i < POINTS && (i as usize) < points.len() && (i as usize) < links.len();
    let named = |w: u16| w == NONE || ring(w as u32);
    if !ring(n) || !named(links[n as usize].previous) || !named(links[n as usize].next) {
        return Err(Invalid("a skid point or its links outside the ring"));
    }
    if last != u32::MAX && !ring(last) {
        return Err(Invalid("the last skid point outside the ring"));
    }
    let (nu, has_last) = (n as usize, last != u32::MAX);
    let mut added = Added {
        angle: 0,
        offset: (0, 0),
        from: None,
        turned: None,
        last_moved: false,
        right: [0; 3],
        next_read: 0,
    };
    if has_last {
        let m = middle(&points[last as usize]);
        added.from = Some(m);
        added.angle = angle_of(place[0].wrapping_sub(m[0]), place[2].wrapping_sub(m[2]));
        added.offset = offset_of(added.angle & 0xFFFF, m[2]);
    }
    links[nu].car = car;
    points[nu] = edges(place, added.offset);
    let previous = links[nu].previous;
    if previous != NONE {
        links[previous as usize].next = NONE;
    }
    added.next_read = links[nu].next;
    if added.next_read != NONE {
        links[added.next_read as usize].previous = NONE;
    }
    if has_last {
        links[last as usize].next = n as u16;
    }
    links[nu].previous = last as u16;
    links[nu].next = NONE;
    links[nu].angle = added.angle as u16;
    if has_last {
        let l = last as usize;
        added.last_moved = true;
        if links[l].previous == NONE {
            links[l].angle = added.angle as u16;
            let p = points[l];
            let (ox, oz) = added.offset;
            points[l] = [
                p[0].wrapping_sub(ox),
                p[1],
                p[2].wrapping_sub(oz),
                p[3].wrapping_add(ox),
                p[4],
                p[5].wrapping_add(oz),
            ];
        } else {
            let turned = between(added.angle, links[l].angle);
            let m = middle(&points[l]);
            added.turned = Some((turned, m));
            added.offset = offset_of(turned, m[1]);
            points[l] = edges(m, added.offset);
        }
        let p = points[l];
        added.right = [p[3], p[4], p[5]];
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!(middle(&[10, 20, 30, 0, 0, 0]), [5, 10, 15]);
        assert_eq!(middle(&[0, 0, 0, 10, 20, 31]), [5, 10, 15]);
        assert_eq!(edges([10, 5, 10], (2, 3)), [8, 5, 7, 12, 5, 13]);
        assert_eq!(between(0x0001_0010, 0x0008), 0x0001_000C);
        assert_eq!(between(0x0000_0002, 0xFFFE), 0x0000_0000);
        assert_eq!(next_point(0x3FF), 0);
        assert_eq!(next_point(5), 6);
    }

    #[test]
    fn strips() {
        let mut points = vec![[0u32; 6]; POINTS as usize];
        let mut links = vec![
            Link {
                next: NONE,
                previous: NONE,
                angle: 0,
                car: 0
            };
            POINTS as usize
        ];
        // A strip's first point: no angle, no offset.
        let a = add_skid_point(
            Ring {
                points: &mut points,
                links: &mut links,
            },
            0,
            u32::MAX,
            [100, 5, 100],
            1,
            |_, _| unreachable!(),
            |_, _| unreachable!(),
        )
        .unwrap();
        assert_eq!((a.angle, a.last_moved), (0, false));
        assert_eq!(points[0], [100, 5, 100, 100, 5, 100]);
        // The second: linked after the first, which turns to its angle.
        let a = add_skid_point(
            Ring {
                points: &mut points,
                links: &mut links,
            },
            1,
            0,
            [200, 5, 100],
            1,
            |dx, _| dx,
            |_, _| (0, 7),
        )
        .unwrap();
        assert_eq!(a.angle, 100);
        assert_eq!(links[0].next, 1);
        assert_eq!(links[1].previous, 0);
        assert_eq!(links[0].angle, 100);
        assert_eq!(points[0], [100, 5, 93, 100, 5, 107]);
        assert_eq!(a.right, [100, 5, 107]);
        // The third: the second, not first in its strip, turned halfway.
        let a = add_skid_point(
            Ring {
                points: &mut points,
                links: &mut links,
            },
            2,
            1,
            [300, 5, 100],
            1,
            |_, _| 200,
            |_, _| (1, 1),
        )
        .unwrap();
        assert_eq!(a.turned.map(|t| t.0), Some(150));
        // Reusing point 0 cuts it from the strip.
        add_skid_point(
            Ring {
                points: &mut points,
                links: &mut links,
            },
            0,
            u32::MAX,
            [0, 0, 0],
            2,
            |_, _| 0,
            |_, _| (0, 0),
        )
        .unwrap();
        assert_eq!(links[1].previous, NONE);
        assert!(
            add_skid_point(
                Ring {
                    points: &mut points,
                    links: &mut links
                },
                POINTS,
                u32::MAX,
                [0; 3],
                0,
                |_, _| 0,
                |_, _| (0, 0)
            )
            .is_err()
        );
        assert!(
            add_skid_point(
                Ring {
                    points: &mut points,
                    links: &mut links
                },
                3,
                0x1_FFFF,
                [0; 3],
                0,
                |_, _| 0,
                |_, _| (0, 0)
            )
            .is_err()
        );
    }
}
