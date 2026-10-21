//! The tyre-change screen's layout (converted 2026-10-21 from `drive::tyre_screen`
//! 0x41C5FA, the layout and the wheel's place, and `drive::tyre_screen_parts` 0x41C790,
//! its ten fixed quads): rectangles as (x, y, w, h) whole numbers, a texture's and the
//! screen's. A quad's corners (converted 2026-10-21 from `drive::tyre_quad` 0x41DD3B).

/// A rectangle (x, y, w, h).
pub type Rect = [u32; 4];

/// The layout tables (0x20 bytes each from `unk_5BEB30`): one table (mode 0), two for the
/// wheel further left (mode 2), two more (other modes); by car 0 or not. (The table's
/// number from `unk_5BEB30`, the wheel's shift left.)
pub fn layout(mode: u32, car: u32) -> (u32, u32) {
    match mode {
        0 => (0, 0),
        2 => (if car != 0 { 2 } else { 1 }, 0xAA),
        _ => (if car != 0 { 4 } else { 3 }, 0),
    }
}

/// The wheel's place: the panel's middle less the wheel's half (each by an arithmetic
/// shift of the difference), less the shift.
pub fn wheel_place(panel: Rect, wheel: [u32; 2], shift: u32) -> [u32; 2] {
    let half = |x: u32, y: u32| ((x.wrapping_sub(y) as i32) >> 1) as u32;
    [
        half(panel[2], wheel[0])
            .wrapping_add(panel[0])
            .wrapping_sub(shift),
        half(panel[3], wheel[1]).wrapping_add(panel[1]),
    ]
}

/// The ten fixed quads (texture, screen) from the second rectangle of the layout
/// (`dword_5BEBD0` ..), the wheel's rectangle (`dword_5BEAE0` ..) and whether the car is
/// not the first (its half of the texture).
pub fn fixed_parts(table: Rect, wheel: Rect, second_car: bool) -> [(Rect, Rect); 10] {
    let mut out = [([0; 4], [0; 4]); 10];
    out[0] = ([0, 0x80, 0x100, 0x19], [table[0], table[1], 0x100, 0x19]);
    let rect = [
        wheel[0].wrapping_sub(0x4E),
        wheel[1].wrapping_sub(0xA),
        0x80,
        0x19,
    ];
    out[1] = ([0x80, 0, 0x80, 0x19], rect);
    out[2] = (
        [0x80, 0x19, 0x80, 0x19],
        [
            wheel[0].wrapping_add(wheel[2]).wrapping_sub(0x32),
            rect[1],
            0x80,
            0x19,
        ],
    );
    let half = if second_car { 0x80 } else { 0 };
    for k in 0..3u32 {
        let x = table[2].wrapping_add(0x78 * k);
        out[3 + k as usize] = (
            [half, 0xB2 + 0x19 * k, 0x80, 0x19],
            [x, table[3].wrapping_sub(0x32), 0x80, 0x19],
        );
        out[6 + k as usize] = (
            [0x80, 0x32 + 0x19 * k, 0x80, 0x19],
            [x, table[3], 0x80, 0x19],
        );
    }
    out[9] = (
        [0, 0x99, 0x100, 0x19],
        [
            table[2].wrapping_add(0x3C),
            table[3].wrapping_add(0x1E),
            0x100,
            0x19,
        ],
    );
    out
}

/// `drive::tyre_quad`: a screen rectangle's four corners as the vertices' (x, y)
/// floats, in the vertices' order: (x, y), (x + w, y), (x, y + h), (x + w, y + h). Each
/// sum `fild` / `fiadd` (exact) and stored once (`fst`): rounded once to the float at any
/// precision.
pub fn quad_corners(rect: Rect) -> [[f32; 2]; 4] {
    let n = |v: u32| i64::from(v as i32);
    let (x, y) = (n(rect[0]) as f32, n(rect[1]) as f32);
    let right = (n(rect[0]) + n(rect[2])) as f32;
    let bottom = (n(rect[1]) + n(rect[3])) as f32;
    [[x, y], [right, y], [x, bottom], [right, bottom]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners() {
        let c = quad_corners([10, 20, 5, (-30i32) as u32]);
        assert_eq!(
            c,
            [[10.0, 20.0], [15.0, 20.0], [10.0, -10.0], [15.0, -10.0]]
        );
        // A sum past 2^24 rounded once.
        let c = quad_corners([0x7FFF_FFFF, 0, 0x7FFF_FFFF, 0]);
        assert_eq!(c[1][0], 4294967294.0f64 as f32);
    }

    #[test]
    fn layouts() {
        assert_eq!(layout(0, 1), (0, 0));
        assert_eq!(layout(2, 1), (2, 0xAA));
        assert_eq!(layout(3, 0), (3, 0));
        assert_eq!(wheel_place([100, 50, 200, 80], [60, 20], 0), [170, 80]);
        assert_eq!(
            wheel_place([100, 50, 200, 80], [60, 20], 0xAA),
            [170 - 0xAA, 80]
        );
        let p = fixed_parts([10, 20, 300, 400], [500, 600, 64, 64], true);
        assert_eq!(p[0], ([0, 0x80, 0x100, 0x19], [10, 20, 0x100, 0x19]));
        assert_eq!(p[2].1, [500 + 64 - 0x32, 600 - 0xA, 0x80, 0x19]);
        assert_eq!(
            p[5],
            (
                [0x80, 0xB2 + 0x32, 0x80, 0x19],
                [300 + 0xF0, 400 - 0x32, 0x80, 0x19]
            )
        );
        assert_eq!(p[8].0, [0x80, 0x32 + 0x32, 0x80, 0x19]);
        assert_eq!(p[9].1, [300 + 0x3C, 400 + 0x1E, 0x100, 0x19]);
    }
}
