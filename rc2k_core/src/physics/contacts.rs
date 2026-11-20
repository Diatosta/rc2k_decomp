//! The car's contacts with the ground and with objects' points: when the body's deepest point
//! touches, which face normal a contact takes, and whether an object's point lies inside the
//! car's box.

use crate::Invalid;

/// The room for the body's points (0x40 each).
pub const POINT_ROOM: u32 = 0x40;

/// Refuses a deepest-point number the tables do not hold.
pub fn check_body_point(n: u32) -> Result<(), Invalid> {
    if n >= POINT_ROOM {
        return Err(Invalid("a body point past 0x3F (the tables' room)"));
    }
    Ok(())
}

/// Whether the deepest point (a whole number) makes a contact: below `limit` and, unless all
/// four wheels are off the ground, below 0; each by sign.
pub fn touches(depth: u32, limit: u32, all_off: bool) -> bool {
    let below = |a: u32, b: u32| (a.wrapping_sub(b) as i32) < 0;
    below(depth, limit) && (all_off || below(depth, 0))
}

/// The depth limit for a ground contact.
pub const GROUND_LIMIT: u32 = 0x1000;

/// The face normal's offsets a contact takes: the first set when the point's flip is 1.
pub fn normal_offsets(flip: u32) -> [u32; 3] {
    if flip == 1 {
        [0x1C, 0x34, 0x3C]
    } else {
        [0x18, 0x30, 0x38]
    }
}

/// One part of an object's point from the car: point + object + offset - car (wrapping).
pub fn point_part(point: u32, object: u32, offset: u32, car: u32) -> u32 {
    point
        .wrapping_add(object)
        .wrapping_add(offset)
        .wrapping_sub(car)
}

/// Whether the point in the car's frame (whole numbers x, z) lies in its box: x within the
/// half width, z between minus the back and the front; each by the sign of the difference.
pub fn inside_box(x: u32, z: u32, half_width: u32, front: u32, back: u32) -> bool {
    let sign = |v: u32| (v as i32) < 0;
    sign(x.wrapping_sub(half_width))
        && !sign(x.wrapping_add(half_width))
        && sign(z.wrapping_sub(front))
        && !sign(z.wrapping_add(back))
}

/// Whether a point inside makes a hit of this contact kind (10, 1, 5 or 6).
pub fn kind_hits(kind: u32) -> bool {
    matches!(kind, 10 | 1 | 5 | 6)
}

/// The room of the objects' tables.
pub const OBJECT_ROOM: u32 = 0x100;

/// Refuses an object number the tables do not hold.
pub fn check_object(object: u32) -> Result<(), Invalid> {
    if object >= OBJECT_ROOM {
        return Err(Invalid("an object past 0xFF (the tables' room)"));
    }
    Ok(())
}

/// How a walk over an object's points ended: whether a point was inside, its number, the
/// points left and its address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointWalk {
    pub inside: bool,
    /// The number of the point reached.
    pub number: u32,
    /// The points left.
    pub left: u32,
    /// The reached point's address.
    pub address: u32,
}

/// Walks the object's `count` points from `first` (12 bytes each from `points`), calling
/// `hits(address)` until one is inside. With no points the address is `no_points_address`.
pub fn walk_points(
    first: u32,
    count: u32,
    points: u32,
    no_points_address: u32,
    mut hits: impl FnMut(u32) -> bool,
) -> PointWalk {
    let mut walk = PointWalk {
        inside: false,
        number: first,
        left: count,
        address: if count == 0 {
            no_points_address
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
mod tests;
