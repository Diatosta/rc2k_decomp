//! A point's place in the view and on the screen (from
//! `scene::project_point` 0x44D004, its matrix step `scene::rotate_point` and its screen step, and
//! from `scene::project_polygon` 0x44E1F7). The records are 0x20 bytes: +0 the point's
//! number, +0xC the x on the screen, +0x10 .. +0x18 the point in the view, +0x1C the y on
//! the screen; a point n of the scene has record n & 0xFFF, a point -n of an object record
//! 0x1000 + (n & 0xFFF). As in `math::geometry`, what the x87 keeps on its stack is an
//! `f64`, rounded to `f32` where the original stores it: the same bits at the game's double
//! precision. Also an object's points and normals (from `scene::place_object_points`
//! 0x45BE05, its normal step 0x45C1E7).

/// A point nearer than this (`flt_5C5E18`) is not put on the screen, and counts as outside
/// a polygon's depths.
pub const NEAR: f32 = 4096.0;
/// What a lowered record's depth loses (`flt_5C5E1C`).
pub const LOWERED_BY: f32 = 262144.0;

/// Why a point's number has no record: past the object's points (fatal error 0x80600902)
/// or past the scene's and the 0x40 after them (0x80600903).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooHigh {
    Object,
    Scene,
}

/// The offset of a point's record: `object_points` the objects' count (`dword_5CD680`),
/// `scene_points` the scene's (`dword_5CCD20`).
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

/// `scene::rotate_point`: a point's three whole numbers through the view's matrix (`flt_5EA2CC`, 4
/// x 4: row c of the result is factors c, 4 + c, 8 + c on x, y, z, the y and z products added
/// first, then 12 + c).
pub fn rotate(point: [i32; 3], m: &[f32; 16]) -> [f32; 3] {
    let [x, y, z] = point.map(|v| v as f64);
    std::array::from_fn(|c| {
        let k = |n: usize| m[n + c] as f64;
        (k(0) * x + (k(4) * y + k(8) * z) + k(12)) as f32
    })
}

/// 0x45C1E7: a normal's three floats through the matrix's first three rows (as
/// `rotate`, without the fourth).
pub fn turn_normal(normal: [f32; 3], m: &[f32; 16]) -> [f32; 3] {
    let [x, y, z] = normal.map(|v| v as f64);
    std::array::from_fn(|c| {
        let k = |n: usize| m[n + c] as f64;
        (k(0) * x + (k(4) * y + k(8) * z)) as f32
    })
}

/// The view's screen: its scale (`flt_624088`), centre (`flt_62409C`, `flt_6240A0`) and,
/// when it is flat (`dword_60932C`), the whole number the depth divided by is made from
/// (`dword_5CCD80`, plus 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub scale: f32,
    pub centre: [f32; 2],
    pub flat: Option<i32>,
}

/// A place in the view onto the screen: x times (the scale over the depth) plus the
/// centre's x, the centre's y less y times the same.
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

/// A record made anew: the place in the view, on the screen (0, 0 nearer than `NEAR`),
/// and what goes to `dword_62400C` (the depth when flat and on the screen, then
/// `LOWERED_BY` when lowered), if anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Made {
    pub view: [f32; 3],
    pub screen: [f32; 2],
    pub checked: Option<f32>,
}

/// A scene point's record made (`project_point`, 0x44CF4E): through the matrix, onto
/// the screen (`screen()`, asked only then) unless nearer than `NEAR` (or no number), and,
/// `lowered` (`dword_6365C4` 1), the depth less `LOWERED_BY`.
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

/// `project_polygon`'s test of a projected point's depth: nearer than `NEAR` or not below
/// the farthest (`flt_6241C8`), each compare taking "no number" for "below".
pub fn outside_depths(depth: f32, far: f32) -> bool {
    let below = |a: f32, b: f32| a.partial_cmp(&b).is_none_or(std::cmp::Ordering::is_lt);
    below(depth, NEAR) || !below(depth, far)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY_FAR: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 9000.0, 0.0,
    ];

    #[test]
    fn offsets() {
        assert_eq!(record_offset(3, 6, 8), Ok(0x60));
        assert_eq!(record_offset(0x47, 6, 8), Ok(0x47 << 5));
        assert_eq!(record_offset(0x48, 6, 8), Err(TooHigh::Scene));
        assert_eq!(record_offset(-5i32 as u32, 6, 8), Ok((0x1005) << 5));
        assert_eq!(record_offset(-6i32 as u32, 6, 8), Err(TooHigh::Object));
        // 0x80000000 is an object's number past any count
        assert_eq!(record_offset(0x8000_0000, 6, 8), Err(TooHigh::Object));
    }

    #[test]
    fn records() {
        let s = Screen {
            scale: 640.0,
            centre: [400.0, 300.5],
            flat: None,
        };
        let m = make_record([100, -50, 1000], &IDENTITY_FAR, || s, false);
        assert_eq!(m.view, [100.0, -50.0, 10000.0]);
        assert_eq!(m.screen, [406.4, 303.7]);
        assert_eq!(m.checked, None);
        // too near: not on the screen; lowered
        let m = make_record([1, 2, -6000], &IDENTITY_FAR, || unreachable!(), true);
        assert_eq!((m.view[2], m.screen), (3000.0 - 262144.0, [0.0, 0.0]));
        assert_eq!(m.checked, Some(LOWERED_BY));
        // flat: the depth divided by is 7999 + 1, the real one checked
        let flat = Screen {
            flat: Some(7999),
            ..s
        };
        let m = make_record([80, 0, 0], &IDENTITY_FAR, || flat, false);
        assert_eq!((m.screen, m.checked), ([406.4, 300.5], Some(9000.0)));
        // the order of the sums: x' = m0 x + (m4 y + m8 z) + m12
        let mut r = [0.0f32; 16];
        (r[0], r[4], r[8], r[12]) = (1.0, 1.0, -1.0, 0.5);
        assert_eq!(rotate([0x4000_0000, 1, -0x4000_0000], &r)[0], 2147483648.0);
        assert_eq!(turn_normal([0.5, 2.0, 1.0], &r)[0], 1.5);
        assert_eq!(
            turn_normal([0.6, -0.8, 0.0], &IDENTITY_FAR),
            [0.6, -0.8, 0.0]
        );
    }

    #[test]
    fn depths() {
        assert!(outside_depths(100.0, 9000.0));
        assert!(!outside_depths(4096.0, 9000.0));
        assert!(outside_depths(9000.0, 9000.0));
        assert!(outside_depths(f32::NAN, 9000.0));
        assert!(!outside_depths(5000.0, f32::NAN));
    }
}
