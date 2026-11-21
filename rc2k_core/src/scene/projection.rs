//! A point's place in the view and on the screen. Records are 0x20 bytes: the point's
//! number, the x on the screen, the point in the view (three floats), the y on
//! the screen; point n of the scene has record n & 0xFFF, point -n of an object record
//! 0x1000 + (n & 0xFFF). Intermediates are `f64`, rounded to `f32` where stored (as in
//! `math::geometry`). Also an object's points and normals.

/// A point nearer than this is not put on the screen, and counts as outside a polygon's depths.
pub const NEAR: f32 = 4096.0;
/// What a lowered record's depth loses.
pub const LOWERED_BY: f32 = 262144.0;

/// Why a point's number has no record: past the object's points or past the scene's and
/// the 0x40 after them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooHigh {
    Object,
    Scene,
}

/// The offset of a point's record: `object_points` the objects' count, `scene_points` the
/// scene's.
pub fn record_offset(point: u32, object_points: u32, scene_points: u32) -> Result<u32, TooHigh> {
    if (point as i32) < 0 {
        let number = point.wrapping_neg();
        if number >= object_points {
            return Err(TooHigh::Object);
        }
        Ok(((number & 0xFFF) + 0x1000) << 5)
    } else if point >= scene_points.wrapping_add(0x40) {
        Err(TooHigh::Scene)
    } else {
        Ok((point & 0xFFF) << 5)
    }
}

/// A point's three whole numbers through the view's 4 x 4 matrix: row c of the result is
/// factors c, 4 + c, 8 + c on x, y, z (the y and z products added first), then 12 + c.
pub fn rotate(point: [i32; 3], m: &[f32; 16]) -> [f32; 3] {
    let [x, y, z] = point.map(|v| v as f64);
    std::array::from_fn(|c| {
        let k = |n: usize| m[n + c] as f64;
        (k(0) * x + (k(4) * y + k(8) * z) + k(12)) as f32
    })
}

/// A normal's three floats through the matrix's first three rows (`rotate` without the
/// fourth).
pub fn turn_normal(normal: [f32; 3], m: &[f32; 16]) -> [f32; 3] {
    let [x, y, z] = normal.map(|v| v as f64);
    std::array::from_fn(|c| {
        let k = |n: usize| m[n + c] as f64;
        (k(0) * x + (k(4) * y + k(8) * z)) as f32
    })
}

/// The view's screen: its scale, centre and, when it is flat, the whole number the depth
/// divided by is made from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub scale: f32,
    pub centre: [f32; 2],
    pub flat: Option<i32>,
}

/// A place in the view onto the screen: x times (the scale over the depth) plus the centre's x,
/// the centre's y less y times the same.
pub fn to_screen(view: [f32; 3], s: &Screen) -> [f32; 2] {
    let depth = match s.flat {
        Some(d) => d as f64 + 1.0,
        None => view[2] as f64,
    };
    let k = s.scale as f64 / depth;
    [
        (view[0] as f64 * k + s.centre[0] as f64) as f32,
        (-(view[1] as f64 * k) + s.centre[1] as f64) as f32,
    ]
}

/// A record made anew: the place in the view, on the screen (0, 0 nearer than `NEAR`), and
/// what goes to the scratch (the depth when flat and on the screen, then `LOWERED_BY` when
/// lowered), if anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Made {
    pub view: [f32; 3],
    pub screen: [f32; 2],
    pub checked: Option<f32>,
}

/// A scene point's record: through the matrix, onto the screen (`screen()`, asked only then)
/// unless nearer than `NEAR` (or no number), and, `lowered`, the depth less `LOWERED_BY`.
pub fn make_record(
    point: [i32; 3],
    m: &[f32; 16],
    screen: impl FnOnce() -> Screen,
    lowered: bool,
) -> Made {
    let mut view = rotate(point, m);
    let mut on_screen = [0.0, 0.0];
    let mut checked = None;
    if view[2] >= NEAR {
        let s = screen();
        if s.flat.is_some() {
            checked = Some(view[2]);
        }
        on_screen = to_screen(view, &s);
    }
    if lowered {
        view[2] = (view[2] as f64 - LOWERED_BY as f64) as f32;
        checked = Some(LOWERED_BY);
    }
    Made {
        view,
        screen: on_screen,
        checked,
    }
}

/// Whether a projected point's depth is nearer than `NEAR` or not below the farthest, each
/// compare taking "no number" for "below".
pub fn outside_depths(depth: f32, far: f32) -> bool {
    let below = |a: f32, b: f32| a.partial_cmp(&b).is_none_or(std::cmp::Ordering::is_lt);
    below(depth, NEAR) || !below(depth, far)
}

#[cfg(test)]
mod tests;
