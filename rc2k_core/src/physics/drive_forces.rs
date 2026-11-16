//! The drive's forces under the wheels' step.
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where it is stored.

use super::x87::{below, loaded};

/// The low-speed factor: None when |`speed`| is not below `limit` (so the forces are left),
/// else (|speed| / limit + `add`) x `times`.
pub fn low_speed_factor(speed: f32, limit: f32, add: f32, times: f32) -> Option<f32> {
    if !below(speed.abs() as f64, limit as f64) {
        return None;
    }
    Some(((speed.abs() as f64 / limit as f64 + add as f64) * times as f64) as f32)
}

/// The two forces, each times the factor.
pub fn eased(forces: [f32; 2], factor: f32) -> [f32; 2] {
    forces.map(|f| (f as f64 * factor as f64) as f32)
}

/// In reverse (`gear` -1) while `speed` is not below `zero`: the two values held to at most
/// `cap`. None when not in reverse or the speed is below `zero`.
pub fn cap_reverse(
    gear: i32,
    speed: f32,
    zero: f32,
    values: [f32; 2],
    cap: f32,
) -> Option<[f32; 2]> {
    if gear != -1 || below(speed as f64, zero as f64) {
        return None;
    }
    Some(values.map(|v| {
        if below(v as f64, cap as f64) {
            v
        } else {
            loaded(cap)
        }
    }))
}

/// What adding the drive forces makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DriveForces {
    /// The first factor, kept aside with drive kind not 0.
    pub kept: Option<f32>,
    /// The factor times the four's sum.
    pub drive: f32,
    /// The forces with their shares added.
    pub forces: [f32; 2],
}

/// The drive forces: the factor (`factors[0]`, or `factors[1]` with drive kind not 0) times the
/// sum of the four; each force plus its input x that x `scale`.
pub fn add_drive_forces(
    kind: u32,
    factors: [f32; 2],
    four: [f32; 4],
    inputs: [f32; 2],
    scale: f32,
    forces: [f32; 2],
) -> DriveForces {
    let (kept, factor) = if kind != 0 {
        (Some(loaded(factors[0])), factors[1])
    } else {
        (None, factors[0])
    };
    let sum = four[0] as f64 + four[1] as f64 + four[2] as f64 + four[3] as f64;
    let drive = (sum * factor as f64) as f32;
    let add =
        |input: f32, force: f32| (input as f64 * drive as f64 * scale as f64 + force as f64) as f32;
    DriveForces {
        kept,
        drive,
        forces: [add(inputs[0], forces[0]), add(inputs[1], forces[1])],
    }
}

/// The allowance, the speed plus it, and the drive speed capped at that.
pub fn cap_drive_speed(
    over: f32,
    speed: f32,
    add: f32,
    less: f32,
    zero: f32,
    drive: f32,
) -> (f32, f32, f32) {
    let mut allowance = (over as f64 / (speed.abs() as f64 + add as f64) - less as f64) as f32;
    if below(allowance as f64, zero as f64) {
        allowance = loaded(zero);
    }
    let cap = (allowance as f64 + speed as f64) as f32;
    let drive = if below(drive as f64, cap as f64) {
        drive
    } else {
        loaded(cap)
    };
    (allowance, cap, drive)
}

/// What the drive-torque share makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorqueShare {
    /// `ns[0]` with drive kind 1, else `ns[1]`.
    pub n: u32,
    /// |speed| / `per`, held to at most 1.
    pub speed_share: f32,
    /// n less the word, and that / n.
    pub less: u32,
    pub drive_share: f32,
    /// d + (1 - d) s, and the drive times it.
    pub share: f32,
    pub drive: f32,
}

/// The drive torque's share from the drive kind, the speed and the word.
pub fn torque_share(
    kind: u32,
    ns: [u32; 2],
    speed: f32,
    per: i32,
    one: f32,
    word: u32,
    drive: f32,
) -> TorqueShare {
    let n = if kind == 1 { ns[0] } else { ns[1] };
    let mut speed_share = (speed.abs() as f64 / per as f64) as f32;
    if !below(speed_share as f64, one as f64) {
        speed_share = loaded(one);
    }
    let less = n.wrapping_sub(word);
    let drive_share = (less as i32 as f64 / n as i32 as f64) as f32;
    let d = drive_share as f64;
    let share = ((1.0 - d) * speed_share as f64 + d) as f32;
    TorqueShare {
        n,
        speed_share,
        less,
        drive_share,
        share,
        drive: (drive as f64 * share as f64) as f32,
    }
}

/// Downforce: (`speed`² `factor` + `offset`) `gravity`, and its negative.
pub fn downforce(speed: f32, factor: f32, offset: f32, gravity: f32) -> (f32, f32) {
    let s = speed as f64;
    let load = ((s * s * factor as f64 + offset as f64) * gravity as f64) as f32;
    (load, -load)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_speeds_and_reverse() {
        assert_eq!(low_speed_factor(-10.0, 20.0, 0.5, 2.0), Some(2.0));
        assert_eq!(low_speed_factor(20.0, 20.0, 0.5, 2.0), None);
        assert_eq!(
            low_speed_factor(f32::NAN, 20.0, 0.5, 2.0).map(f32::is_nan),
            Some(true)
        );
        assert_eq!(eased([2.0, -3.0], 0.5), [1.0, -1.5]);
        assert_eq!(cap_reverse(-1, 1.0, 0.0, [3.0, 1.0], 2.0), Some([2.0, 1.0]));
        assert_eq!(cap_reverse(-1, -1.0, 0.0, [3.0, 1.0], 2.0), None);
        assert_eq!(cap_reverse(1, 1.0, 0.0, [3.0, 1.0], 2.0), None);
    }

    #[test]
    fn downforces() {
        assert_eq!(downforce(2.0, 0.5, 1.0, -2.0), (-6.0, 6.0));
        let (load, minus) = downforce(f32::NAN, 1.0, 0.0, 1.0);
        assert_eq!(minus.to_bits(), load.to_bits() ^ 0x8000_0000);
    }

    #[test]
    fn forces() {
        let f = add_drive_forces(
            1,
            [2.0, 0.5],
            [1.0, 2.0, 3.0, 4.0],
            [1.0, 2.0],
            2.0,
            [1.0, 1.0],
        );
        assert_eq!(
            f,
            DriveForces {
                kept: Some(2.0),
                drive: 5.0,
                forces: [11.0, 21.0]
            }
        );
        assert_eq!(
            add_drive_forces(0, [2.0, 0.5], [1.0; 4], [0.0; 2], 1.0, [0.0; 2]).drive,
            8.0
        );
        assert_eq!(
            cap_drive_speed(10.0, -4.0, 1.0, 1.0, 0.0, 9.0),
            (1.0, -3.0, -3.0)
        );
        assert_eq!(
            cap_drive_speed(1.0, 9.0, 1.0, 1.0, 0.0, 2.0),
            (0.0, 9.0, 2.0)
        );
        let s = torque_share(1, [8, 32], -16.0, 32, 1.0, 4, 2.0);
        assert_eq!(
            (s.n, s.speed_share, s.less, s.drive_share),
            (8, 0.5, 4, 0.5)
        );
        assert_eq!((s.share, s.drive), (0.75, 1.5));
        assert_eq!(
            torque_share(0, [8, 32], 64.0, 32, 1.0, 0, 2.0).speed_share,
            1.0
        );
    }
}
