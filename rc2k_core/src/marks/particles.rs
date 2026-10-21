//! The marks' particles' points moved by lot, and the splines' blended points (converted
//! 2026-10-21 from `marks::jitter_point` 0x4C6106, `spread_point` 0x4C61AB and
//! `blend_points` 0x4BA5E2). As in `math::vector`, what the x87 keeps on its stack is an
//! `f64`, rounded to `f32` where the original stores it: the same bits at the game's
//! double precision (`fprem` is exact, as `%` is).

/// A step of the marks' random (`g_marksSeed`, `g_marksSpreadRandom`): times 0x9D plus 0x47.
pub fn next_random(r: u32) -> u32 {
    r.wrapping_mul(0x9D).wrapping_add(0x47)
}

/// What `jitter_point` leaves: the point moved (in the stack's order, b c d as they came
/// in st(1) .. st(3)), the random and its last value shifted right 12 (ebx).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jittered {
    pub point: [f32; 3],
    pub seed: u32,
    pub last: i32,
}

/// `marks::jitter_point`: three steps of the random; each moves one of the point's
/// values (d first, then c, then b) by the step shifted right 12 (arithmetic) remainder
/// the spread (`fprem`), stored.
pub fn jitter(spread: f32, point: [f32; 3], seed: u32) -> Jittered {
    let mut moved = [point[2], point[1], point[0]];
    let (mut seed, mut last) = (seed, 0);
    for v in moved.iter_mut() {
        seed = next_random(seed);
        last = seed as i32 >> 12;
        *v = (last as f64 % spread as f64 + *v as f64) as f32;
    }
    Jittered {
        point: [moved[2], moved[1], moved[0]],
        seed,
        last,
    }
}

/// `marks::spread_point`: one step of the random, then it rotated right 12 and again; each
/// of the point's values (d by the third, c the second, b the first) grown by itself times
/// the spread times the value (signed) times `scale` (a double), stored. Returns the point
/// (b c d) and the third value (the random kept, and eax).
pub fn spread(spread: f32, point: [f32; 3], random: u32, scale: f64) -> ([f32; 3], u32) {
    let first = next_random(random);
    let second = first.rotate_right(12);
    let third = second.rotate_right(12);
    let grown = |r: u32, v: f32| {
        let v = v as f64;
        (r as i32 as f64 * scale * (v * spread as f64) + v) as f32
    };
    (
        [
            grown(first, point[0]),
            grown(second, point[1]),
            grown(third, point[2]),
        ],
        third,
    )
}

/// `marks::blend_points`: three points of whole numbers a, b, c blended by an entry of the
/// spline file (`weights` as there: c's, b's, a's): each of x, y, z a ka + (b kb + c kc),
/// unrounded.
pub fn blend(points: [[i32; 3]; 3], weights: [f32; 3]) -> [f64; 3] {
    let [a, b, c] = points;
    let [kc, kb, ka] = weights.map(f64::from);
    std::array::from_fn(|n| a[n] as f64 * ka + (b[n] as f64 * kb + c[n] as f64 * kc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jitters_and_spreads() {
        // Seed 0: 0x47 (>> 12 is 0), then 0x2BD2 (2), then 0x1AE011 (0x1AE, 1 remainder 3).
        let j = jitter(3.0, [10.0, 20.0, 30.0], 0);
        assert_eq!(j.seed, 0x1A_E011);
        assert_eq!(j.last, 0x1AE);
        assert_eq!(j.point, [11.0, 22.0, 30.0]);
        let (p, third) = spread(1.0, [1.0, 2.0, 0.0], 0, 0.5);
        assert_eq!(third, 0x47u32.rotate_right(24));
        assert_eq!(p[0], (71.0 * 0.5 * 1.0 + 1.0) as f32);
        assert_eq!(p[2], 0.0);
        assert_eq!(
            blend([[1, 2, 3], [4, 5, 6], [7, 8, 9]], [0.0, 0.5, 2.0]),
            [4.0, 6.5, 9.0]
        );
    }
}
