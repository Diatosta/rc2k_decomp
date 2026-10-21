//! The drive's forces under the wheels' step (converted 2026-10-21 from
//! `physics::ease_low_speed_forces` 0x440CF3, `physics::cap_reverse_inputs` 0x46514A,
//! `physics::add_drive_forces` 0x440C7B, `physics::cap_drive_speed` 0x440DF3,
//! `physics::drive_torque_share` 0x440D56 and `physics::set_downforce` 0x43F508). What
//! the figures are is a reading (the hooks' headers say which).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores (`physics::x87`): the same bits at the game's double precision.

use super::x87::{below, loaded};

/// `physics::ease_low_speed_forces`: the factor (`flt_61475C`) 1, or when |`speed`| is
/// below `limit` (20; or unordered) (|speed| / limit + `add`) x `times`, and the two
/// forces times it then (None: the forces left).
pub fn low_speed_factor(speed: f32, limit: f32, add: f32, times: f32) -> Option<f32> {
    if !below(speed.abs() as f64, limit as f64) {
        return None;
    }
    Some(((speed.abs() as f64 / limit as f64 + add as f64) * times as f64) as f32)
}

/// `physics::ease_low_speed_forces`' forces: each times the factor.
pub fn eased(forces: [f32; 2], factor: f32) -> [f32; 2] {
    forces.map(|f| (f as f64 * factor as f64) as f32)
}

/// `physics::cap_reverse_inputs`: in reverse (`gear` -1) while `speed` is not below `zero`
/// (nor unordered), the two values held to at most `cap` (2.0; below it or unordered
/// left). None when nothing is looked at.
pub fn cap_reverse(
    gear: u32,
    speed: f32,
    zero: f32,
    values: [f32; 2],
    cap: f32,
) -> Option<[f32; 2]> {
    if gear != u32::MAX || below(speed as f64, zero as f64) {
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

/// What `physics::add_drive_forces` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DriveForces {
    /// The first factor kept in `dword_62400C` with drive kind not 0.
    pub kept: Option<f32>,
    /// `flt_6140F4`: the factor times the four's sum.
    pub drive: f32,
    /// `flt_614248`, `flt_61424C` with their shares added.
    pub forces: [f32; 2],
}

/// `physics::add_drive_forces`: the factor (`factors[0]`, with drive kind not 0
/// `factors[1]`) times the sum of the four, then each force plus its input x that x
/// `scale`.
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

/// What `physics::cap_drive_speed` makes: (the allowance `flt_614168`, the speed plus it
/// kept in `dword_62400C`, the drive speed).
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

/// What `physics::drive_torque_share` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorqueShare {
    /// n (`dword_613EEC` with drive kind 1, else `dword_613EF0`), kept in `dword_62400C`.
    pub n: u32,
    /// |speed| / `dword_613EF4`, held to at most 1 (`flt_62401C`).
    pub speed_share: f32,
    /// n less the word (the original's eax), then (that) / n (`dword_624020`).
    pub less: u32,
    pub drive_share: f32,
    /// `flt_614758` = d + (1 - d) s; `flt_614750` times it.
    pub share: f32,
    pub drive: f32,
}

/// `physics::drive_torque_share`.
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

/// `physics::set_downforce`: (`speed`² `factor` + `offset`) `gravity` (`flt_614174`) and
/// its negative (`flt_6147F4`): by the look of it the car's weight with downforce
/// growing with the speed's square (a reading).
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
        assert_eq!(
            cap_reverse(u32::MAX, 1.0, 0.0, [3.0, 1.0], 2.0),
            Some([2.0, 1.0])
        );
        assert_eq!(cap_reverse(u32::MAX, -1.0, 0.0, [3.0, 1.0], 2.0), None);
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
