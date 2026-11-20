//! The skidmarks' points: a ring of `POINTS` points (each the mark's left and right edges)
//! linked in strips (next, previous, angle, car's bit, words). The angle between two places
//! and the edges' offset for an angle are asked through closures.

use crate::Invalid;

/// The points in the ring.
pub const POINTS: u32 = 0x400;
/// A link to no point.
pub const NONE: u16 = 0xFFFF;
/// Half a mark's width (the edges' offset before turning).
pub const HALF_WIDTH: i32 = 0x1555;

/// A point: its left edge's x, y, z, then its right edge's.
pub type Point = [u32; 6];

/// A point's links in its strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Link {
    pub next: u16,
    /// The previous point in the strip (`NONE` for the first).
    pub previous: u16,
    /// The mark's angle (low word).
    pub angle: u16,
    /// The car's bit.
    pub car: u16,
}

/// A point's middle: (left - right) >> 1 (signed) + right, per axis.
pub fn middle(p: &Point) -> [u32; 3] {
    std::array::from_fn(|i| {
        ((p[i].wrapping_sub(p[3 + i]) as i32 >> 1) as u32).wrapping_add(p[3 + i])
    })
}

/// Whether a wheel at `place` is where the strip's last point already is (on the ground plane).
/// A point laid there would have no direction (the angle between two equal places is 0), so its
/// edges, and the strip's last point's, would turn across the strip: a twisted quad that draws as
/// a gap. A game drawn faster than the physics steps asks again before the wheel has moved.
pub fn is_repeat(last: &Point, place: [u32; 3]) -> bool {
    let m = middle(last);
    m[0] == place[0] && m[2] == place[2]
}

/// A point's edges across `centre` by `offset` (x, z): left less it, right plus it, y the
/// centre's.
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

/// Halfway from the last point's angle to the new one (low word, turn's sign kept), the new
/// one's high word.
pub fn between(angle: u32, last: u16) -> u32 {
    let half = ((angle as u16).wrapping_sub(last) as i16 >> 1) as u16;
    (angle & 0xFFFF_0000) | half.wrapping_add(last) as u32
}

/// The ring's next point after `n`.
pub fn next_point(n: u32) -> u32 {
    let next = n.wrapping_add(1);
    if next == POINTS { 0 } else { next }
}

/// The ring's tables, read and written one entry at a time, so a caller whose tables are
/// stored elsewhere need not copy them out. The indexes given are below `capacity()`.
pub trait Ring {
    /// How many points (and links) the tables hold.
    fn capacity(&self) -> usize;
    fn point(&self, index: usize) -> Point;
    fn set_point(&mut self, index: usize, point: Point);
    fn link(&self, index: usize) -> Link;
    fn set_link(&mut self, index: usize, link: Link);
}

#[cfg(test)]
/// The ring over two slices.
pub(crate) struct SliceRing<'a> {
    pub points: &'a mut [Point],
    pub links: &'a mut [Link],
}

#[cfg(test)]
impl Ring for SliceRing<'_> {
    fn capacity(&self) -> usize {
        self.points.len().min(self.links.len())
    }
    fn point(&self, index: usize) -> Point {
        self.points[index]
    }
    fn set_point(&mut self, index: usize, point: Point) {
        self.points[index] = point;
    }
    fn link(&self, index: usize) -> Link {
        self.links[index]
    }
    fn set_link(&mut self, index: usize, link: Link) {
        self.links[index] = link;
    }
}

/// What adding a point worked out along the way, besides the ring's changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Added {
    /// The angle from the last point (0 without one).
    pub angle: u32,
    /// The edges' offset last worked out.
    pub offset: (u32, u32),
    /// The last point's middle, if there is a last point.
    pub from: Option<[u32; 3]>,
    /// Halfway angle and the last point's middle, when it is not the strip's first.
    pub turned: Option<(u32, [u32; 3])>,
    /// Whether the last point was given new edges.
    pub last_moved: bool,
    /// The right edge last written.
    pub right: [u32; 3],
    /// The new point's next link as read, before the last point moved.
    pub next_read: u16,
}

/// Adds point `n` across the wheel's `place` at the angle from the `last` point's middle
/// (`angle_of(dx, dz)`; 0 without one): old links cut, linked after the last, marked with
/// `car`. The last point turns to the new angle (its edges move when it starts its strip,
/// else are made again across its middle halfway); `offset_of(angle, y)` is the edges'
/// offset. Refuses a point or link outside the ring before changing anything.
pub fn add_skid_point(
    mut ring: impl Ring,
    n: u32,
    last: u32,
    place: [u32; 3],
    car: u16,
    mut angle_of: impl FnMut(u32, u32) -> u32,
    mut offset_of: impl FnMut(u32, u32) -> (u32, u32),
) -> Result<Added, Invalid> {
    let len = ring.capacity();
    let inside = |i: u32| i < POINTS && (i as usize) < len;
    let named = |w: u16| w == NONE || inside(w as u32);
    if !inside(n) {
        return Err(Invalid("a skid point or its links outside the ring"));
    }
    let (nu, has_last) = (n as usize, last != u32::MAX);
    let mut own = ring.link(nu);
    if !named(own.previous) || !named(own.next) {
        return Err(Invalid("a skid point or its links outside the ring"));
    }
    if has_last && !inside(last) {
        return Err(Invalid("the last skid point outside the ring"));
    }
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
        let m = middle(&ring.point(last as usize));
        added.from = Some(m);
        added.angle = angle_of(place[0].wrapping_sub(m[0]), place[2].wrapping_sub(m[2]));
        added.offset = offset_of(added.angle & 0xFFFF, m[2]);
    }
    ring.set_point(nu, edges(place, added.offset));
    // The new point's old neighbours lose their links to it. A neighbour may be the point
    // itself or the last point, so each entry is read where it is changed.
    own.car = car;
    ring.set_link(nu, own);
    let previous = own.previous;
    if previous != NONE {
        let mut neighbour = ring.link(previous as usize);
        neighbour.next = NONE;
        ring.set_link(previous as usize, neighbour);
    }
    added.next_read = ring.link(nu).next;
    if added.next_read != NONE {
        let mut neighbour = ring.link(added.next_read as usize);
        neighbour.previous = NONE;
        ring.set_link(added.next_read as usize, neighbour);
    }
    if has_last {
        let mut before = ring.link(last as usize);
        before.next = n as u16;
        ring.set_link(last as usize, before);
    }
    let mut own = ring.link(nu);
    own.previous = last as u16;
    own.next = NONE;
    own.angle = added.angle as u16;
    ring.set_link(nu, own);
    if has_last {
        let l = last as usize;
        added.last_moved = true;
        let mut before = ring.link(l);
        if before.previous == NONE {
            before.angle = added.angle as u16;
            ring.set_link(l, before);
            let p = ring.point(l);
            let (ox, oz) = added.offset;
            ring.set_point(
                l,
                [
                    p[0].wrapping_sub(ox),
                    p[1],
                    p[2].wrapping_sub(oz),
                    p[3].wrapping_add(ox),
                    p[4],
                    p[5].wrapping_add(oz),
                ],
            );
        } else {
            let turned = between(added.angle, before.angle);
            let m = middle(&ring.point(l));
            added.turned = Some((turned, m));
            added.offset = offset_of(turned, m[1]);
            ring.set_point(l, edges(m, added.offset));
        }
        let p = ring.point(l);
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
        let p = edges([100, 7, 200], (5, 9));
        assert!(is_repeat(&p, [100, 8, 200]));
        assert!(!is_repeat(&p, [101, 7, 200]));
        assert!(!is_repeat(&p, [100, 7, 199]));
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
        // First point of a strip: no angle, no offset.
        let a = add_skid_point(
            SliceRing {
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
        // The second links after the first, which turns to its angle.
        let a = add_skid_point(
            SliceRing {
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
        // The third turns the second (not first in its strip) halfway.
        let a = add_skid_point(
            SliceRing {
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
            SliceRing {
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
                SliceRing {
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
                SliceRing {
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
