//! A force record's push and twist on a car (converted 2026-10-21 from
//! `physics::apply_force` 0x45FAE1 and `physics::apply_torque` 0x45FB82; the records'
//! counts of `apply_forces` 0x43F566, `apply_torques` 0x43F6BF and `clear_records`
//! 0x43F3A6, the last emptying the first six dwords of each). A force record (0x20 bytes): the force at +0, the point it acts at at
//! +0xC. A reading: the push moves
//! the car's motion, the twist its spin, `flt_613FA8` .. the inverse inertia.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::loaded;
use crate::math::vector::{rotate_point, scale};

use crate::Invalid;

/// The most records a run takes (the game's lists have 0x17 and 7).
pub const MOST_RECORDS: u32 = 0x100;
/// The dwords `physics::clear_records` empties in each record (the first six of eight).
pub const CLEARED_DWORDS: u32 = 6;

/// A run over `count` records: 0 refused (the originals' `dec; jnz` would loop 2^32
/// times), as is a count past `MOST_RECORDS`; `why` says whose.
pub fn record_count(count: u32, why: &'static str) -> Result<u32, Invalid> {
    if count == 0 || count > MOST_RECORDS {
        return Err(Invalid(why));
    }
    Ok(count)
}

/// A vector turned by a matrix (`physics::turn_point_by`: `rotate_point`, each part
/// stored).
pub fn turned(m: &[f32; 15], v: [f32; 3]) -> [f32; 3] {
    rotate_point(m, v.map(f64::from)).map(|x| x as f32)
}

/// What `apply_force` stores, in its order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Push {
    /// `flt_614B30` ..: the force times the step, turned into the world.
    pub work: [f32; 3],
    /// `dword_62400C`: the step (left in st(0), stored).
    pub step: f32,
    /// The car's motion (+0x198 ..) with the force added.
    pub motion: [f32; 3],
    /// +0x1A4 ..: the motion turned by the car's inverse matrix (the last `g_carSpeed`).
    pub local: [f32; 3],
}

/// `physics::apply_force`: the force times the step, turned by `crash` (`g_crashMatrix`),
/// added to the car's motion; the motion turned by `inverse` (the car's +0x9CC).
pub fn apply_force(
    force: [f32; 3],
    step: f32,
    crash: &[f32; 15],
    motion: [f32; 3],
    inverse: &[f32; 15],
) -> Push {
    let work = turned(crash, scale(force, step as f64));
    let motion = [0, 1, 2].map(|k| (work[k] as f64 + motion[k] as f64) as f32);
    let local = rotate_point(inverse, motion.map(f64::from)).map(|x| x as f32);
    Push {
        work,
        step: loaded(step),
        motion,
        local,
    }
}

/// `scene::cross_product` of `b` and `c` (ebx, ecx): c2 b1 - b2 c1, c0 b2 - b0 c2,
/// c1 b0 - b1 c0, each stored.
pub fn cross(b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    [(2, 1), (0, 2), (1, 0)]
        .map(|(x, y)| (c[x] as f64 * b[y] as f64 - b[x] as f64 * c[y] as f64) as f32)
}

/// What `apply_torque` stores, in its order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Twist {
    /// `flt_614274` ..: the cross product of the record's two vectors.
    pub cross: [f32; 3],
    /// `dword_6D30B4` ..: the spin, in the car's frame, less the twist, turned back (also
    /// the car's new spin, +0x184 ..).
    pub spin: [f32; 3],
}

/// `physics::apply_torque`: the car's spin turned into its frame (`inverse`, +0x9CC),
/// each part less the cross product of the record's force and point times the step and
/// that part's inverse inertia (`fld c; fmul; fmul; fchs; fadd; fstp`), turned back
/// (`matrix`, +0xB4).
pub fn apply_torque(
    record: [f32; 6],
    spin: [f32; 3],
    step: f32,
    inertia: [f32; 3],
    inverse: &[f32; 15],
    matrix: &[f32; 15],
) -> Twist {
    let local = turned(inverse, spin);
    let cross = cross(
        [record[0], record[1], record[2]],
        [record[3], record[4], record[5]],
    );
    let less = [0, 1, 2]
        .map(|k| (-(cross[k] as f64 * step as f64 * inertia[k] as f64) + local[k] as f64) as f32);
    Twist {
        cross,
        spin: turned(matrix, less),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: [f32; 15] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
    ];

    #[test]
    fn pushes() {
        let p = apply_force([2.0, 4.0, -6.0], 0.5, &IDENTITY, [1.0, 1.0, 1.0], &IDENTITY);
        assert_eq!(p.work, [1.0, 2.0, -3.0]);
        assert_eq!(p.motion, [2.0, 3.0, -2.0]);
        assert_eq!(p.local, p.motion);
        assert_eq!(p.step, 0.5);
        let mut moved = IDENTITY;
        moved[12] = 10.0;
        let p = apply_force([0.0; 3], 1.0, &IDENTITY, [1.0, 2.0, 3.0], &moved);
        assert_eq!(p.local, [11.0, 2.0, 3.0]);
    }

    #[test]
    fn twists() {
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        let t = apply_torque(
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            0.5,
            [1.0, 1.0, 1.0],
            &IDENTITY,
            &IDENTITY,
        );
        assert_eq!(t.cross, [0.0, 0.0, 1.0]);
        assert_eq!(t.spin, [0.0, 0.0, 0.5]);
    }

    #[test]
    fn counts() {
        assert!(record_count(0, "x").is_err());
        assert!(record_count(0x101, "x").is_err());
        assert_eq!(record_count(0x17, "x"), Ok(0x17));
    }
}
