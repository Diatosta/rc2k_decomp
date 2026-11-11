//! The car's contacts with the ground and with objects' points (converted
//! from `physics::crash_contact` 0x45EDB5, `ground_contact` 0x45EFF3, `point_hits_car`
//! 0x45E800 and `object_hits_car` 0x45E755, its walk over the object's points; their
//! x87 steps, the turns and the force, stay the hooks'): when the body's deepest point
//! touches, which face normal a contact takes, and whether an object's point lies inside
//! the car's box.

use crate::Invalid;

/// The body's points' room (`probe_x` .., 0x40 each).
pub const POINT_ROOM: u32 = 0x40;

/// The deepest point's number (`lowest_point`) the tables hold.
pub fn check_body_point(n: u32) -> Result<(), Invalid> {
    if n >= POINT_ROOM {
        return Err(Invalid("a body point past 0x3F (the tables' room)"));
    }
    Ok(())
}

/// Whether the deepest point (`lowest_depth`, a whole number) makes a contact: below the
/// limit (`crash_depth` for the crash, 0x1000 for the ground) and, unless all four wheels
/// are off the ground (`all_wheels_off`), below 0; each by the sign of the `cmp`.
pub fn touches(depth: u32, limit: u32, all_off: bool) -> bool {
    let below = |a: u32, b: u32| (a.wrapping_sub(b) as i32) < 0;
    below(depth, limit) && (all_off || below(depth, 0))
}

/// The ground contact's limit.
pub const GROUND_LIMIT: u32 = 0x1000;

/// The face normal's offsets a contact takes (`take_contact`): +0x1C, +0x34, +0x3C with
/// the point's flip (`probe_face_flip`[n]`) at 1, else +0x18, +0x30, +0x38.
pub fn normal_offsets(flip: u32) -> [u32; 3] {
    if flip == 1 {
        [0x1C, 0x34, 0x3C]
    } else {
        [0x18, 0x30, 0x38]
    }
}

/// `physics::point_hits_car`: one part of an object's point from the car: the point's
/// part, plus the object's place, plus the offset, less the car's place (wrapping).
pub fn point_part(point: u32, object: u32, offset: u32, car: u32) -> u32 {
    point
        .wrapping_add(object)
        .wrapping_add(offset)
        .wrapping_sub(car)
}

/// Whether the point turned into the car's frame (whole numbers x, z) lies in its box:
/// x below the half width (`dword_614BC8`) and not below minus it, z below the front
/// (`spec_ints`) and not below minus the back (`dword_614BC4`); each by the sign of
/// the difference, as the original's `cmp`.
pub fn inside_box(x: u32, z: u32, half_width: u32, front: u32, back: u32) -> bool {
    let sign = |v: u32| (v as i32) < 0;
    sign(x.wrapping_sub(half_width))
        && !sign(x.wrapping_add(half_width))
        && sign(z.wrapping_sub(front))
        && !sign(z.wrapping_add(back))
}

/// Whether a point inside makes a hit of the contact kind (`contact_kind`): 10, 1, 5, 6.
pub fn kind_hits(kind: u32) -> bool {
    matches!(kind, 10 | 1 | 5 | 6)
}

/// The objects' tables' room (`g_objectFirstPoint`, `g_objectPointCount2`).
pub const OBJECT_ROOM: u32 = 0x100;

/// The object hit (`face_object`) the tables hold.
pub fn check_object(object: u32) -> Result<(), Invalid> {
    if object >= OBJECT_ROOM {
        return Err(Invalid("an object past 0xFF (the tables' room)"));
    }
    Ok(())
}

/// How `object_hits_car`'s walk ended: whether a point was inside, the point's number, the points left and its address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointWalk {
    pub inside: bool,
    pub number: u32,
    pub left: u32,
    pub address: u32,
}

/// `physics::object_hits_car`: the object's `count` points from `first` (12 bytes each
/// from `points`), `hits(address)` each in turn until one is inside. With no points the address
/// is not set.
pub fn walk_points(
    first: u32,
    count: u32,
    points: u32,
    edx: u32,
    mut hits: impl FnMut(u32) -> bool,
) -> PointWalk {
    let mut walk = PointWalk {
        inside: false,
        number: first,
        left: count,
        address: if count == 0 {
            edx
        } else {
            first.wrapping_mul(12).wrapping_add(points)
        },
    };
    while walk.left != 0 {
        if hits(walk.address) {
            walk.inside = true;
            return walk;
        }
        walk.address = walk.address.wrapping_add(12);
        walk.number = walk.number.wrapping_add(1);
        walk.left -= 1;
    }
    walk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_points() {
        assert!(check_object(0xFF).is_ok() && check_object(0x100).is_err());
        let w = walk_points(2, 0, 0x1000, 77, |_| panic!());
        assert_eq!(
            w,
            PointWalk {
                inside: false,
                number: 2,
                left: 0,
                address: 77
            }
        );
        let w = walk_points(2, 3, 0x1000, 77, |a| a == 0x1000 + 36);
        assert_eq!(
            w,
            PointWalk {
                inside: true,
                number: 3,
                left: 2,
                address: 0x1024
            }
        );
        let w = walk_points(2, 2, 0x1000, 77, |_| false);
        assert_eq!(
            (w.inside, w.number, w.left, w.address),
            (false, 4, 0, 0x1030)
        );
    }

    #[test]
    fn contacts() {
        let n = |v: i32| v as u32;
        assert!(touches(n(-5), 0x1000, false));
        assert!(!touches(5, 0x1000, false));
        assert!(touches(5, 0x1000, true));
        assert!(!touches(0x1000, 0x1000, true));
        // The sign of the difference: far apart wraps.
        assert!(!touches(n(i32::MIN), 0x1000, true));
        assert!(check_body_point(0x3F).is_ok() && check_body_point(0x40).is_err());
        assert_eq!(normal_offsets(1), [0x1C, 0x34, 0x3C]);
        assert_eq!(normal_offsets(2), [0x18, 0x30, 0x38]);
    }

    #[test]
    fn boxes() {
        let n = |v: i32| v as u32;
        assert!(inside_box(0, 0, 10, 20, 30));
        assert!(inside_box(n(-10), n(-30), 10, 20, 30));
        assert!(!inside_box(10, 0, 10, 20, 30));
        assert!(!inside_box(0, 20, 10, 20, 30));
        assert!(!inside_box(0, n(-31), 10, 20, 30));
        assert_eq!(point_part(5, 10, 1, 20), n(-4));
        assert!(kind_hits(6) && !kind_hits(2));
    }
}
