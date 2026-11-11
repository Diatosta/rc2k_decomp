//! An object's polygon drawn whole or as a fan of triangles (from
//! `scene::draw_object_polygon` 0x451F69 and its test `scene::needs_clipping`): the work list
//! started, whether a vertex is off the screen or too near, and the fan's triangles.

/// The kept entries' room (`unk_976610`, up to the list): the most vertices.
pub const MOST_KEPT: u32 = 0x40;

/// Whether a polygon of `count` vertices can be drawn (1 .. 0x40).
pub fn fits(count: u32) -> bool {
    count != 0 && count <= MOST_KEPT
}

/// The list as the drawing starts it: 0, 1, .. `count` - 1 and a 0 after them.
pub fn first_list(count: u32) -> Vec<u32> {
    (0..count).chain([0]).collect()
}

/// The edges `scene::needs_clipping` tests against: the least (`flt_5C5D90`, for the depth, x and
/// y), the screen's right and bottom (`g_clipScreenRight`, `g_clipScreenBottom`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edges {
    pub least: f32,
    pub right: f32,
    pub bottom: f32,
}

/// A vertex's x, y on the screen and depth (its record's +0xC, +0x1C, +0x18).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
}

/// The x87's "below" (C0): also when one of the two is no number.
fn below(a: f32, b: f32) -> bool {
    a.partial_cmp(&b).is_none_or(std::cmp::Ordering::is_lt)
}

/// `scene::needs_clipping`: whether some vertex of a polygon of three or more is nearer than the
/// least, left of or above it, or at or past the right or bottom edge. `vertex(n)` is
/// asked for each in turn until one is.
pub fn needs_clipping<E>(
    count: u32,
    e: &Edges,
    mut vertex: impl FnMut(usize) -> Result<Vertex, E>,
) -> Result<bool, E> {
    if (count as i32).wrapping_sub(3) < 0 {
        return Ok(false);
    }
    for n in 0..count as usize {
        let v = vertex(n)?;
        if below(v.depth, e.least)
            || below(v.x, e.least)
            || !below(v.x, e.right)
            || below(v.y, e.least)
            || !below(v.y, e.bottom)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the polygon is drawn whole (nothing to clip, or a triangle) rather than as a
/// fan.
pub fn drawn_whole(count: u32, needs_clipping: bool) -> bool {
    !needs_clipping || count == 3
}

/// The fan's triangles after the first (entries 0, 1, 2 as they are): for n = 3 .. the
/// kept entries 0, n - 1 and n.
pub fn fan(count: u32) -> impl Iterator<Item = [usize; 3]> {
    (3..count as usize).map(|n| [0, n - 1, n])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fans() {
        assert!(!fits(0) && fits(1) && fits(0x40) && !fits(0x41));
        assert_eq!(first_list(3), [0, 1, 2, 0]);
        let e = Edges {
            least: 1.0,
            right: 640.0,
            bottom: 480.0,
        };
        let on = Vertex {
            x: 10.0,
            y: 10.0,
            depth: 5000.0,
        };
        let vs = [on, on, Vertex { x: 640.0, ..on }, on];
        let mut asked = vec![];
        let cut = needs_clipping::<()>(4, &e, |n| {
            asked.push(n);
            Ok(vs[n])
        });
        assert_eq!((cut, asked), (Ok(true), vec![0, 1, 2]));
        assert_eq!(needs_clipping::<()>(2, &e, |_| Err(())), Ok(false));
        assert_eq!(needs_clipping::<()>(3, &e, |_| Ok(on)), Ok(false));
        let nan = Vertex {
            depth: f32::NAN,
            ..on
        };
        assert_eq!(needs_clipping::<()>(3, &e, |_| Ok(nan)), Ok(true));
        assert!(drawn_whole(5, false) && drawn_whole(3, true) && !drawn_whole(4, true));
        assert_eq!(fan(5).collect::<Vec<_>>(), [[0, 2, 3], [0, 3, 4]]);
        assert_eq!(fan(3).count(), 0);
    }
}
