//! The marks' view (converted 2026-10-21 from `marks::turn_kept_vectors` 0x4C13CA,
//! `turn_kept_xz` 0x4C1411, `turn_kept_zy` 0x4C14D2, `scale_kept_vectors` 0x4C15F4,
//! `scale_vectors_by` 0x4D16A3, `view_step` 0x4C1593, `aim_vector` 0x4C1294,
//! `tilt_shape` 0x4C1194, `view_motion` 0x4C4D40 and `view_heading` 0x4C4CEC): the three
//! kept vectors turned by two angles and scaled, the view's step, its aim made of length
//! 1, a shape's tilt, the view's motion and heading.
//!
//! As in `math::vector`, what the x87 keeps on its stack is an `f64`, rounded to `f32`
//! where the original stores it (the same bits at the game's double precision, but for
//! the sines and cosines: `scene::trig`).

use crate::math::vector;
use crate::scene::trig;

/// An angle's sine and cosine (`fsin`, `fcos`), stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SineCosine {
    pub sine: f32,
    pub cosine: f32,
}

impl SineCosine {
    pub fn of(angle: f32) -> Self {
        let a = angle as f64;
        Self {
            sine: trig::sine(a) as f32,
            cosine: trig::cosine(a) as f32,
        }
    }
}

/// (y, z) turned by a stored sine and cosine: (z s + y c, z c - y s).
pub fn turn_yz(y: f32, z: f32, t: SineCosine) -> (f32, f32) {
    let (y, z, s, c) = (y as f64, z as f64, t.sine as f64, t.cosine as f64);
    ((z * s + y * c) as f32, (z * c - y * s) as f32)
}

/// The three kept vectors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeptVectors(pub [[f32; 3]; 3]);

impl KeptVectors {
    /// Each vector's parts `b` and `a` turned as (y, z) are by `turn_yz`.
    fn turn(&mut self, a: usize, b: usize, t: SineCosine) {
        for v in &mut self.0 {
            (v[b], v[a]) = turn_yz(v[b], v[a], t);
        }
    }

    /// `marks::turn_kept_xz`: x and z by the second angle's.
    pub fn turn_xz(&mut self, second: SineCosine) {
        self.turn(0, 2, second);
    }

    /// `marks::turn_kept_zy`: z and y by the first angle's.
    pub fn turn_zy(&mut self, first: SineCosine) {
        self.turn(2, 1, first);
    }
}

/// `marks::turn_kept_vectors`: the two angles' sines and cosines (stored, then read back),
/// x and z turned by the second's, then z and y by the first's.
pub fn turn_kept_vectors(v: &mut KeptVectors, angles: [f32; 2]) -> [SineCosine; 2] {
    let turns = angles.map(SineCosine::of);
    v.turn_xz(turns[1]);
    v.turn_zy(turns[0]);
    turns
}

/// A vector times a factor (`fld; fmul; fstp` each).
pub fn scaled(v: [f32; 3], factor: f32) -> [f32; 3] {
    v.map(|x| (x as f64 * factor as f64) as f32)
}

/// `marks::view_step`: the step from the last place to this one, each part `-(now -
/// last)` as the original has it.
pub fn step(now: [f32; 3], last: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| -(now[k] as f64 - last[k] as f64) as f32)
}

/// A vector made of length 1 and its length, as `marks::aim_vector` and `tilt_shape` do it
/// (the length stored with `fst`, each part times one over the unrounded length).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit {
    pub unit: [f32; 3],
    pub length: f32,
}

pub fn unit_and_length(v: [f32; 3]) -> Unit {
    Unit {
        unit: vector::normalise(v),
        length: vector::length(v) as f32,
    }
}

/// `marks::aim_vector`'s vector: each part `-t0 - t1 + t2 + t3`, from its four terms.
pub fn aim(terms: [[f32; 4]; 3]) -> [f32; 3] {
    terms.map(|t| {
        let t = t.map(f64::from);
        (-t[0] - t[1] + t[2] + t[3]) as f32
    })
}

/// What `marks::tilt_shape` makes: the tilt's sine and cosine, the point's y and z turned
/// by them, and the point (turned) made of length 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tilt {
    pub turn: SineCosine,
    pub y: f32,
    pub z: f32,
    pub unit: Unit,
}

pub fn tilt(angle: f32, point: [f32; 3]) -> Tilt {
    let turn = SineCosine::of(angle);
    let (y, z) = turn_yz(point[1], point[2], turn);
    Tilt {
        turn,
        y,
        z,
        unit: unit_and_length([point[0], y, z]),
    }
}

/// What `marks::view_motion` makes: the step since the last frame (over the frame's
/// ticks), how much the view faces along it, and the place kept as the last.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    /// The step as first stored (the place less the last), then over the ticks.
    pub step: [f32; 3],
    pub per_tick: [f32; 3],
    pub facing: f32,
    pub last: [f32; 3],
}

/// `marks::view_motion`: the view's place (the origin through its matrix `m`, the game's
/// 4 x 4 layout as `vector::rotate_point` takes it, unrounded) less the last place; the
/// third row's dot product with that step made of length 1; the place kept; the step
/// over the ticks (`fidiv`).
pub fn view_motion(m: &[f32; 15], last: [f32; 3], ticks: i32) -> Motion {
    let place = vector::rotate_point(m, [0.0; 3]);
    let step: [f32; 3] = std::array::from_fn(|k| (place[k] - last[k] as f64) as f32);
    let facing = vector::dot([m[8], m[9], m[10]], vector::normalise(step)) as f32;
    Motion {
        step,
        per_tick: step.map(|s| (s as f64 / ticks as f64) as f32),
        facing,
        last: place.map(|p| p as f32),
    }
}

/// `marks::view_heading`'s last step: the angle (a float) as it is when the level
/// direction's x (normalised) is negative, else 2 pi less it.
pub fn heading(angle: f32, x: f32) -> f32 {
    if x.is_sign_negative() {
        angle
    } else {
        (2.0 * std::f64::consts::PI - angle as f64) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kept_turns() {
        let quarter = SineCosine {
            sine: 1.0,
            cosine: 0.0,
        };
        assert_eq!(turn_yz(1.0, 0.0, quarter), (0.0, -1.0));
        let mut v = KeptVectors([[1.0, 2.0, 3.0], [0.0; 3], [0.0, 0.0, 1.0]]);
        v.turn_xz(quarter);
        // z' = x s + z c, x' = x c - z s.
        assert_eq!(v.0[0], [-3.0, 2.0, 1.0]);
        v.turn_zy(quarter);
        // y' = z s + y c, z' = z c - y s.
        assert_eq!(v.0[0], [-3.0, 1.0, -2.0]);
        let mut w = KeptVectors([[1.0, 2.0, 3.0]; 3]);
        let t = turn_kept_vectors(&mut w, [0.0, 0.0]);
        assert_eq!(
            t[0],
            SineCosine {
                sine: 0.0,
                cosine: 1.0
            }
        );
        assert_eq!(w.0[2], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn steps_and_aims() {
        assert_eq!(scaled([1.0, -2.0, 0.5], 2.0), [2.0, -4.0, 1.0]);
        assert_eq!(step([1.0, 2.0, 3.0], [0.5, 2.0, 4.0]), [-0.5, -0.0, 1.0]);
        assert_eq!(aim([[1.0, 1.0, 5.0, 0.0]; 3]), [3.0; 3]);
        let u = unit_and_length([0.0, 3.0, 4.0]);
        assert_eq!(u.length, 5.0);
        assert_eq!(u.unit, [0.0, 0.6, 0.8]);
        let t = tilt(0.0, [0.0, 3.0, 4.0]);
        assert_eq!((t.y, t.z), (3.0, 4.0));
        assert_eq!(t.unit, u);
    }

    #[test]
    fn motion_and_heading() {
        let mut m = [0.0f32; 15];
        (m[8], m[9], m[10]) = (0.0, 0.0, 1.0);
        m[12..15].copy_from_slice(&[1.0, 2.0, 7.0]);
        let v = view_motion(&m, [1.0, 2.0, 3.0], 2);
        assert_eq!(v.step, [0.0, 0.0, 4.0]);
        assert_eq!(v.per_tick, [0.0, 0.0, 2.0]);
        assert_eq!(v.facing, 1.0);
        assert_eq!(v.last, [1.0, 2.0, 7.0]);
        assert_eq!(heading(1.0, -0.5), 1.0);
        assert_eq!(heading(1.0, 0.0), (std::f64::consts::TAU - 1.0) as f32);
    }
}
