//! How a course face goes to the polygon drawer: as one quad or as its two triangles, the
//! corners each takes, the polygon mode its textures pick, which corners are in sight
//! (from `race::draw_whole_quad` 0x452F3D, `draw_face_triangles`
//! 0x452CF3, `draw_face_polygons` 0x452CCF, `draw_face` 0x452C6F, `clip_and_draw_face`
//! 0x454187, `face_out_of_view` 0x45246F, `draw_tinted_face` 0x44F9F3, `draw_back_face`
//! 0x452B58 and `draw_face_sides` 0x452393).

/// The quad's polygon entries from the face's corners, (entry, corner) in the original's
/// order: entries 1, 2, 3, 0 from corners 0, 1, 3, 2.
pub const QUAD_ENTRIES: [(u32, u32); 4] = [(1, 0), (2, 1), (3, 3), (0, 2)];
/// The first triangle's entries (corners 0, 1, 2) and the second's (3, 2, 1).
pub const FIRST_TRIANGLE: [(u32, u32); 3] = [(0, 0), (1, 1), (2, 2)];
pub const SECOND_TRIANGLE: [(u32, u32); 3] = [(0, 3), (1, 2), (2, 1)];

/// The polygon mode of a face with one texture.
pub const PLAIN_MODE: u32 = 2;

/// `draw_face`'s polygon mode: the plain one, or with a second texture the mode its
/// record's +0x2C picks (`surface_kinds`[pick]`, asked only then).
pub fn face_mode(second_texture: u32, picked: impl FnOnce() -> u32) -> u32 {
    if second_texture != 0 {
        picked()
    } else {
        PLAIN_MODE
    }
}

/// `draw_face_polygons`: one quad is tried in a race with both triangles facing the view
/// (their two answers adding to 0).
pub fn tries_quad(in_race: u32, first_away: u32, second_away: u32) -> bool {
    in_race != 0 && first_away.wrapping_add(second_away) == 0
}

/// `draw_face_triangles`: the first triangle drawn unless it faces away; the second when
/// the face has one (a fourth corner) and it does not face away.
pub fn draws_first(first_away: u32) -> bool {
    first_away == 0
}

pub fn draws_second(fourth: u32, second_away: u32) -> bool {
    fourth != crate::race::faces::NO_CORNER && second_away == 0
}

/// `draw_whole_quad` after the projection: by the vertices outside, Some(true) to draw as
/// triangles (some outside), Some(false) to draw nothing (all four), None to test the
/// screen's inside.
pub fn quad_by_outside(outside: u32) -> Option<bool> {
    match outside {
        4 => Some(false),
        0 => None,
        _ => Some(true),
    }
}

/// The screen's inside the quad must keep to: (left, right, top, bottom) each one in
/// from the edges (`SceneDraw.clip_edges[2]`, `SceneDraw.clip_edges[3]`, `clip_edges`, `SceneDraw.clip_edges[1]`).
pub fn screen_inside(left: u32, right: u32, top: u32, bottom: u32) -> [u32; 4] {
    [
        left.wrapping_add(1),
        right.wrapping_sub(1),
        top.wrapping_add(1),
        bottom.wrapping_sub(1),
    ]
}

/// A rounded coordinate inside `low` .. `high` (by the signs of the differences).
pub fn inside(n: u32, low: u32, high: u32) -> bool {
    (n.wrapping_sub(low) as i32) >= 0 && (n.wrapping_sub(high) as i32) < 0
}

/// `clip_and_draw_face`: a polygon is drawn with three vertices or more.
pub fn draws_polygon(vertices: u32) -> bool {
    vertices >= 3
}

/// How a corner of `face_out_of_view` is in sight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InSight {
    /// |x| below the near limit (the original keeps |x| in `scratch`).
    Near,
    /// |x| times the slope below the depth.
    Slope,
}

/// A corner in sight: its depth (a float's bits) not negative (sign bit clear), and |x|
/// below the near limit, or |x| times the slope below the depth (each "below" true when
/// unordered, as `fcomp` / `jb`). Exact in `f64`: the product of two floats is.
pub fn corner_in_sight(depth: u32, x: u32, near: f32, slope: f32) -> Option<InSight> {
    if depth & 0x8000_0000 != 0 {
        return None;
    }
    use crate::physics::x87::below;
    let f = |b: u32| f32::from_bits(b) as f64;
    let x = f(x & 0x7FFF_FFFF);
    if below(x, near as f64) {
        return Some(InSight::Near);
    }
    if below(x * slope as f64, f(depth)) {
        return Some(InSight::Slope);
    }
    None
}

/// `draw_tinted_face`: the polygon colour times the object's tint byte by byte
/// ((a x b) >> 8): all four bytes with MMX, the low three without (the top byte 0).
pub fn tinted(tint: u32, colour: u32, mmx: bool) -> u32 {
    let mut out = 0;
    for k in 0..if mmx { 4 } else { 3 } {
        let p = (((tint >> (8 * k)) & 0xFF) * ((colour >> (8 * k)) & 0xFF)) >> 8;
        out |= p << (8 * k);
    }
    out
}

/// An object's tint is used when it is not 0.
pub fn tints(tint: u32) -> bool {
    tint != 0
}

/// `draw_back_face`: a two-sided face (`back_a`, the first kind's bit 3) has a back.
pub fn has_back(two_sided: u32) -> bool {
    two_sided != 0
}

/// The corners turned over: the first two swapped always, the last two when there is a
/// fourth.
pub fn turns_last_pair(fourth: u32) -> bool {
    fourth != crate::race::faces::NO_CORNER
}

/// The first texture's order turned over: the turned order of its low three bits
/// (`turned_corner_order`[order & 7]`), its bit 3 kept.
pub fn turned_order(order: u32, turned: impl FnOnce(u32) -> u8) -> u32 {
    turned(order & 7) as u32 | (order & 8)
}

/// `draw_face_sides` in a race: a face is drawn only with an object on either triangle
/// (+1 or +0x21).
pub fn has_objects(first: u8, second: u8) -> bool {
    first | second != 0
}

/// A two-sided face (its kind's bit 3).
pub fn two_sided(kind: u8) -> bool {
    kind & 8 != 0
}

/// A side is drawn unless both triangles face away (their answers and'ed).
pub fn side_drawn(first_away: u32, second_away: u32) -> bool {
    first_away & second_away == 0
}

/// The facings flipped for the back (`xor 1`).
pub fn flipped(away: u32) -> u32 {
    away ^ 1
}

/// A projected corner: its depth in the view and its place on the screen (floats' bits, a
/// point record's +0x18, +0xC, +0x1C).
#[derive(Clone, Copy, Debug)]
pub struct ScreenCorner {
    pub depth: u32,
    pub x: u32,
    pub y: u32,
}

/// The screen's clip edges (`clip_screen`: top, bottom, left, right).
#[derive(Clone, Copy, Debug)]
pub struct ScreenEdges {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Whether a face whose polygons all lie within its corners' hull leaves nothing on the
/// screen (not the original's: it finds this only at the end, cutting each polygon to the
/// screen): every corner projected (its depth at least `near`, so no polygon is cut at the
/// near plane and each lies on the same side of a screen edge as its corners), and every
/// corner strictly past the same edge (the clipper's passes then keep none of its
/// vertices). Ties never count as past, and a corner with an unordered depth or place keeps
/// the face (the clipper counts such a place as inside), so a face this answers for would
/// have been cut away whole.
pub fn off_screen(corners: &[ScreenCorner], edges: ScreenEdges, near: f32) -> bool {
    let f = |b: u32| f32::from_bits(b) as f64;
    let mut past = [true; 4];
    for c in corners {
        // Unordered (NaN) counts as nearer.
        if f(c.depth)
            .partial_cmp(&(near as f64))
            .is_none_or(std::cmp::Ordering::is_lt)
        {
            return false;
        }
        let (x, y) = (f(c.x), f(c.y));
        // The clipper keeps an unordered place inside every edge: a face with one is drawn.
        if x.is_nan() || y.is_nan() {
            return false;
        }
        past[0] &= y < edges.top as f64;
        past[1] &= y > edges.bottom as f64;
        past[2] &= x < edges.left as f64;
        past[3] &= x > edges.right as f64;
    }
    !corners.is_empty() && past.contains(&true)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_face_off_the_screen_only_when_wholly_past_one_edge() {
        let b = |v: f32| v.to_bits();
        let e = ScreenEdges {
            top: 0.0,
            bottom: 100.0,
            left: 0.0,
            right: 200.0,
        };
        let c = |d: f32, x: f32, y: f32| ScreenCorner {
            depth: b(d),
            x: b(x),
            y: b(y),
        };
        // All above the top.
        assert!(off_screen(
            &[
                c(5000.0, 10.0, -1.0),
                c(5000.0, 300.0, -5.0),
                c(9e3, -9.0, -0.5)
            ],
            e,
            4096.0
        ));
        // Past two different edges (above and right), so not wholly past one.
        assert!(!off_screen(
            &[c(5000.0, 10.0, -1.0), c(5000.0, 300.0, 50.0)],
            e,
            4096.0
        ));
        // On an edge is not past it.
        assert!(!off_screen(
            &[c(5000.0, 10.0, 0.0), c(5000.0, 20.0, -1.0)],
            e,
            4096.0
        ));
        assert!(!off_screen(
            &[c(5000.0, 200.0, 10.0), c(5000.0, 300.0, 10.0)],
            e,
            4096.0
        ));
        // A corner nearer than the near plane, or unordered: kept.
        assert!(!off_screen(
            &[c(4000.0, 10.0, -1.0), c(5000.0, 10.0, -1.0)],
            e,
            4096.0
        ));
        assert!(!off_screen(&[c(f32::NAN, 10.0, -1.0)], e, 4096.0));
        assert!(!off_screen(
            &[c(5000.0, f32::NAN, -1.0), c(5000.0, 10.0, -1.0)],
            e,
            4096.0
        ));
        assert!(!off_screen(&[], e, 4096.0));
        // Each edge alone.
        assert!(off_screen(&[c(5e3, 10.0, 101.0)], e, 4096.0));
        assert!(off_screen(&[c(5e3, -1.0, 10.0)], e, 4096.0));
        assert!(off_screen(&[c(5e3, 201.0, 10.0)], e, 4096.0));
    }

    use super::*;

    #[test]
    fn tints_backs_sides() {
        assert_eq!(tinted(0xFF_80_40_FF, 0x80_80_FF_02, true), 0x7F_40_3F_01);
        assert_eq!(tinted(0xFF_80_40_FF, 0x80_80_FF_02, false), 0x00_40_3F_01);
        assert!(tints(1) && !tints(0));
        assert!(has_back(8) && !has_back(0));
        assert!(turns_last_pair(5) && !turns_last_pair(u32::MAX));
        assert_eq!(
            turned_order(0xB, |n| [7, 6, 5, 4, 3, 2, 1, 0][n as usize]),
            8 | 4
        );
        assert!(has_objects(0, 3) && !has_objects(0, 0));
        assert!(two_sided(0x18) && !two_sided(0x10));
        assert!(side_drawn(1, 0) && !side_drawn(1, 1) && side_drawn(2, 1));
        assert_eq!(flipped(1), 0);
        assert_eq!(flipped(2), 3);
    }

    #[test]
    fn quads_and_triangles() {
        assert_eq!(face_mode(0, || unreachable!()), 2);
        assert_eq!(face_mode(5, || 4), 4);
        assert!(tries_quad(1, 0, 0) && !tries_quad(0, 0, 0) && !tries_quad(1, 1, 0));
        assert!(tries_quad(1, 1, u32::MAX));
        assert!(draws_first(0) && !draws_first(1));
        assert!(draws_second(7, 0) && !draws_second(u32::MAX, 0) && !draws_second(7, 1));
        assert_eq!(quad_by_outside(4), Some(false));
        assert_eq!(quad_by_outside(0), None);
        assert_eq!(quad_by_outside(2), Some(true));
        assert_eq!(screen_inside(0, 640, 0, 480), [1, 639, 1, 479]);
        assert!(inside(1, 1, 639) && !inside(639, 1, 639) && !inside(0, 1, 639));
        assert!(!inside(0x8000_0001, 1, 639));
        assert!(draws_polygon(3) && !draws_polygon(2));
    }

    #[test]
    fn corners_in_sight() {
        let b = |v: f32| v.to_bits();
        assert_eq!(corner_in_sight(b(-1.0), b(0.0), 1.0, 1.0), None);
        assert_eq!(corner_in_sight(b(-0.0), b(0.0), 1.0, 1.0), None);
        assert_eq!(
            corner_in_sight(b(5.0), b(-0.5), 1.0, 1.0),
            Some(InSight::Near)
        );
        assert_eq!(
            corner_in_sight(b(5.0), b(2.0), 1.0, 2.0),
            Some(InSight::Slope)
        );
        assert_eq!(corner_in_sight(b(4.0), b(2.0), 1.0, 2.0), None);
        assert_eq!(
            corner_in_sight(b(4.0), f32::NAN.to_bits(), 1.0, 2.0),
            Some(InSight::Near)
        );
        assert_eq!(
            corner_in_sight(f32::NAN.to_bits() & 0x7FFF_FFFF, b(2.0), 1.0, 2.0),
            Some(InSight::Slope)
        );
    }
}
