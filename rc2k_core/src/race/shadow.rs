//! The car's shadow, its whole-number parts (from `race::shadow_hull`
//! 0x45836F, the cells of `race::shadow_faces` 0x457618 with 0x4576C9, and the near
//! test of `race::clip_shadow_to_face` 0x457960). A shadow point is a 0x20-byte record
//! (+0 x, +4 y, +8 z as whole numbers, +0xC the angle, ...). The cars' shadows' passes
//! from `race::cast_car_shadow` 0x4575C1 and `race::cast_car_shadows`
//! 0x45754C.

use crate::Invalid;

/// A shadow point's record, as dwords.
pub type Record = [u32; 8];

/// The most shadow points.
pub const MOST_POINTS: u32 = 0x40;

/// The removal that ended or last changed the hull.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// The point taken out (the records after it moved down over it).
    pub at: u32,
    /// The records moved.
    pub moved: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hull {
    /// The points left (`hull_count`).
    pub count: u32,
    /// The point the walk was at.
    pub at: u32,
    /// The point tested last (whose turn decided).
    pub tested: u32,
    /// The last turn's two products' difference (a 64-bit value: not negative takes a point out).
    pub difference: i64,
    /// The last turn's first product (`product`).
    pub product: i64,
    /// The last removal (none: no point was taken out).
    pub cut: Option<Cut>,
    /// Whether the walk ended on a removal (no point left to test) rather than at the end.
    pub ended_on_cut: bool,
}

/// The records `hull` reads and moves for `count` points: the points, the first's copy
/// after them and the two after that (moved down with the rest).
pub fn hull_records(count: u32) -> usize {
    count as usize + 3
}

/// `race::shadow_hull`: the first record copied after the last; then, walking the points,
/// each one where the turn from the one before to the one after is not to the left (the
/// 64-bit cross product of whole numbers, (z1 - z0)(x2 - x0) taken from (x1 - x0)(z2 - z0),
/// not negative) is taken out (the rest moved down, starting again from the first) until
/// none is. Refuses (before anything) fewer than two points, more than `MOST_POINTS`, and
/// records shorter than `hull_records`.
pub fn hull(records: &mut [Record], count: u32) -> Result<Hull, Invalid> {
    if !(2..=MOST_POINTS).contains(&count) {
        return Err(Invalid("fewer than two shadow points or more than 0x40"));
    }
    if records.len() < hull_records(count) {
        return Err(Invalid("the shadow's records are too short"));
    }
    records[count as usize] = records[0];
    let mut last = count - 1;
    let mut cut = None;
    let side = |a: Record, b: Record| {
        (
            b[0].wrapping_sub(a[0]) as i32 as i64,
            b[2].wrapping_sub(a[2]) as i32 as i64,
        )
    };
    loop {
        let mut at = 0u32;
        loop {
            let n = at as usize;
            let (p0, p1, p2) = (records[n], records[n + 1], records[n + 2]);
            let (dx1, dz1) = side(p0, p1);
            let (dx2, dz2) = side(p0, p2);
            let product = dz1 * dx2;
            let difference = (dx1 * dz2).wrapping_sub(product);
            if difference >= 0 {
                let moved = last - at + 2;
                let from = n + 2;
                records.copy_within(from..from + moved as usize, n + 1);
                cut = Some(Cut { at: at + 1, moved });
                last -= 1;
                if last != 0 {
                    break;
                }
                return Ok(Hull {
                    count: last + 1,
                    at,
                    tested: at,
                    difference,
                    product,
                    cut,
                    ended_on_cut: true,
                });
            }
            at += 1;
            if (at.wrapping_sub(last) as i32) >= 0 {
                return Ok(Hull {
                    count: last + 1,
                    at,
                    tested: at - 1,
                    difference,
                    product,
                    cut,
                    ended_on_cut: false,
                });
            }
        }
    }
}

/// A course record's cell (its first dword: x in bits 0 .. 10, z in bits 11 .. 21; -1 for
/// none).
pub fn record_cell(word: u32) -> Option<(u32, u32)> {
    (word != u32::MAX).then_some((word & 0x7FF, word >> 11 & 0x7FF))
}

/// Whether a cell is one of the 3 x 3 round the car's place (x and z, whole numbers: a
/// cell is 2^21 across, the car's place moved half a cell on).
pub fn cell_near_car(car: (u32, u32), cell: (u32, u32)) -> bool {
    let off =
        |p: u32, c: u32| ((p.wrapping_add(0x10_0000).wrapping_sub(c << 21)) as i32 >> 21) as u32;
    off(car.0, cell.0).wrapping_add(1) < 3 && off(car.1, cell.1).wrapping_add(1) < 3
}

/// Whether a course face takes the shadow (0x4576C9): its kind below 0x10 and bit 7 of
/// its switches' word (asked for only then) clear.
pub fn takes_shadow(kind: u8, switches: impl FnOnce() -> u16) -> bool {
    kind < 0x10 && switches() & 0x80 == 0
}

/// A face's two triangles for the shadow: its corners' offsets in the face record.
pub const SHADOW_TRIANGLES: [[u32; 3]; 2] = [[4, 8, 0xC], [0x10, 0xC, 8]];

/// How far a face's corner may be from the car (each of x and z) to be near.
pub const NEAR: u32 = 0x10_0000;

/// What `face_near` found: whether the face is near, the corner asked about last and
/// the last distance worked out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Near {
    pub near: bool,
    pub corner: usize,
    pub distance: u32,
}

/// `race::clip_shadow_to_face`'s test: the face's corners in turn (`corner(n)`: its x and
/// z) until one is more than `NEAR` off the car in x (not near) or within it in z (near);
/// none such, not near.
pub fn face_near(car: (u32, u32), mut corner: impl FnMut(usize) -> (u32, u32)) -> Near {
    let off = |a: u32, b: u32| (a.wrapping_sub(b) as i32).wrapping_abs() as u32;
    let mut found = Near {
        near: false,
        corner: 0,
        distance: 0,
    };
    for n in 0..3 {
        let (x, z) = corner(n);
        found.corner = n;
        found.distance = off(x, car.0);
        if found.distance > NEAR {
            break;
        }
        found.distance = off(z, car.1);
        if found.distance < NEAR {
            found.near = true;
            break;
        }
    }
    found
}

/// 0x457A85's walk: the polygon's `count` sides (`entry(k)` and `entry(k + 1)`, the
/// first entry again after the last) cut by an edge: for each side whether each end is on
/// the inner side (`inside`, asked of the first then the second), the first kept (`add`)
/// when it is, and where the two differ the crossing (`cross`, giving the new entry)
/// added after it. Gives what the last side added last (none: nothing).
pub fn clip_by_edge(
    count: u32,
    mut entry: impl FnMut(u32) -> u32,
    mut inside: impl FnMut(u32) -> bool,
    mut cross: impl FnMut(u32, u32) -> u32,
    mut add: impl FnMut(u32),
) -> Option<u32> {
    let mut last_added = None;
    for k in 0..count {
        let (a, b) = (entry(k), entry(k + 1));
        let (a_in, b_in) = (inside(a), inside(b));
        last_added = None;
        if a_in {
            add(a);
            last_added = Some(a);
        }
        if a_in != b_in {
            let c = cross(a, b);
            add(c);
            last_added = Some(c);
        }
    }
    last_added
}

/// `cast_car_shadow`: a shadow is cast with three corners or more (signed).
pub fn casts(corners: u32) -> bool {
    (corners.wrapping_sub(3) as i32) >= 0
}

/// The hull is drawn with three points or more.
pub fn hull_drawn(points: u32) -> bool {
    points >= 3
}

/// The hull's polygon list: its points in order.
pub fn hull_list(points: u32) -> impl Iterator<Item = u32> {
    0..points
}

/// The shadow's texture (`shadow_texture`), its strength (`shadow_strength`) and the sun's
/// height for the second pass (`sun_height`, put back after).
pub const SHADOW_TEXTURE: u32 = 0xC3;
pub const SHADOW_STRENGTH: u32 = 0x1999;
pub const SUN_PASS: u32 = 0x40;

/// `cast_car_shadows`' refusals: no cars or more than eight, a null block.
pub fn check_shadow_cars(cars: u32, block: impl Fn(u32) -> u32) -> Result<(), Invalid> {
    if cars == 0 || cars > 8 {
        return Err(Invalid("no race cars or more than eight"));
    }
    if (0..cars).any(|n| block(n) == 0) {
        return Err(Invalid("a null car block"));
    }
    Ok(())
}

/// A car casts a shadow when its +0x6FC is set.
pub fn casts_shadow(word: u32) -> bool {
    word != 0
}

/// A car's passes: one with `RaceVars.settings_words[2]` set, then one with the sun at `SUN_PASS`.
pub fn first_pass(switch: u32) -> bool {
    switch != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes() {
        // By the sign of corners - 3.
        assert!(casts(3) && !casts(2) && casts(0x8000_0002));
        assert!(!casts(0x8000_0003));
        assert!(hull_drawn(3) && !hull_drawn(2));
        assert_eq!(hull_list(3).collect::<Vec<_>>(), [0, 1, 2]);
        assert!(check_shadow_cars(0, |_| 1).is_err());
        assert!(check_shadow_cars(9, |_| 1).is_err());
        assert!(check_shadow_cars(2, |n| n).is_err());
        assert!(check_shadow_cars(2, |_| 1).is_ok());
        assert!(casts_shadow(1) && !casts_shadow(0));
        assert!(first_pass(1) && !first_pass(0));
    }

    fn points(shape: &[(i32, i32)]) -> Vec<Record> {
        let mut v: Vec<Record> = shape
            .iter()
            .enumerate()
            .map(|(k, &(x, z))| [x as u32, 9, z as u32, k as u32, 0, 0, 0, 0])
            .collect();
        v.resize(shape.len() + 3, [0xEE; 8]);
        v
    }

    #[test]
    fn hulls() {
        // A square going round the other way keeps all four.
        let mut r = points(&[(0, 0), (0, 100), (100, 100), (100, 0)]);
        let h = hull(&mut r, 4).unwrap();
        assert_eq!((h.count, h.cut, h.ended_on_cut), (4, None, false));
        assert_eq!(r[4], r[0]);
        // A line loses its points one by one.
        let mut r = points(&[(0, 0), (10, 10), (20, 20), (30, 30)]);
        let h = hull(&mut r, 4).unwrap();
        assert!(h.ended_on_cut);
        assert_eq!(h.count, 1);
        // A dent is taken out: (50, 10) between (0, 0) and (100, 0).
        let mut r = points(&[(0, 0), (0, 100), (100, 100), (100, 0), (50, 10)]);
        let h = hull(&mut r, 5).unwrap();
        assert!(h.count < 5);
        assert!(hull(&mut points(&[(0, 0)]), 1).is_err());
        assert!(hull(&mut [[0; 8]; 4], 2).is_err());
    }

    #[test]
    fn cells_and_faces() {
        assert_eq!(record_cell(u32::MAX), None);
        assert_eq!(record_cell(3 | 5 << 11), Some((3, 5)));
        let car = (3 << 21, 5 << 21);
        assert!(cell_near_car(car, (3, 5)));
        assert!(cell_near_car(car, (2, 6)));
        assert!(!cell_near_car(car, (1, 5)));
        assert!(!cell_near_car(car, (3, 7)));
        assert!(takes_shadow(0xF, || 0x7F));
        assert!(!takes_shadow(0x10, || unreachable!()));
        assert!(!takes_shadow(0, || 0x80));
        let n = face_near((0, 0), |n| [(0x20_0000, 0), (0, 0), (0, 0)][n]);
        assert_eq!((n.near, n.corner, n.distance), (false, 0, 0x20_0000));
        let n = face_near((0, 0), |n| [(5, 0x20_0000), (5, 7), (0, 0)][n]);
        assert_eq!((n.near, n.corner, n.distance), (true, 1, 7));
        let polygon = [1u32, 2, 3, 1];
        let mut list = Vec::new();
        let last = clip_by_edge(
            3,
            |k| polygon[k as usize],
            |e| e != 2,
            |a, b| 10 * a + b,
            |e| list.push(e),
        );
        assert_eq!(list, [1, 12, 23, 3]);
        assert_eq!(last, Some(3));
        let mut list = Vec::new();
        let last = clip_by_edge(1, |_| 2, |_| false, |_, _| unreachable!(), |e| list.push(e));
        assert_eq!((list.len(), last), (0, None));
        let n = face_near((0, 0), |_| (0, NEAR));
        assert_eq!((n.near, n.corner, n.distance), (false, 2, NEAR));
    }
}
