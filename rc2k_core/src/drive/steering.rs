//! The work car's steering from its input word (from
//! `drive::steer_inputs` 0x439DB1): by keys (stepped toward the side the word says, or
//! back toward 0) or by device (the word scaled), then the trim.
//!
//! Floats: each figure is one whole number times a float, `fistp`ed: the same at the
//! game's double precision (at extended a device product of more than 53 bits may round
//! one off).

use crate::physics::x87::{below, fistp64};

/// The steering's ends.
pub const LEFT: u32 = 0xE000_0000;
pub const RIGHT: u32 = 0x2000_0000;

fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// What `steer_inputs` reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SteerInputs {
    /// `dword_613890`: the input word (0 none, below 0 left).
    pub word: u32,
    /// `unk_71A234`: 2 when driven by device.
    pub mode: u32,
    /// `unk_71A440`: the keys (bit 0x1000 steering).
    pub keys: u32,
    /// `flt_614C08`, `flt_614C0C`: the device's and the keys' factors.
    pub device_factor: f32,
    pub key_factor: f32,
    /// `unk_71A3E0`: the trim's share; `unk_719FE8` bit 0x10 turns it.
    pub trim: f32,
    pub flags: u32,
    /// `unk_71A308`, `flt_5C5D14`: the trims apply below the limit (or unordered).
    pub figure: f32,
    pub limit: f32,
}

/// What `steer_inputs` makes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Steered {
    /// `unk_71A61C`, `unk_71A620`.
    pub values: [u32; 2],
    /// `unk_71A1E0`.
    pub trim: u32,
    /// `unk_71A624`, `unk_71A628`.
    pub trimmed: [u32; 2],
    /// `dword_62400C` as left (the last whole number put through it).
    pub scratch: u32,
}

/// One steering value moved by the key step toward the word's side (held at the ends),
/// or with the word 0 back toward 0 at twice the step (stopping at 0; each by the sign of
/// the result, as the original).
fn by_keys(v: u32, word: u32, step: u32) -> u32 {
    if word != 0 {
        if sign(word) {
            let n = v.wrapping_sub(step);
            if sign(n.wrapping_sub(LEFT)) { LEFT } else { n }
        } else {
            let n = v.wrapping_add(step);
            if sign(n.wrapping_sub(RIGHT)) {
                n
            } else {
                RIGHT
            }
        }
    } else {
        let step = step.wrapping_add(step);
        if v == 0 {
            v
        } else if sign(v) {
            let n = v.wrapping_add(step);
            if sign(n) { n } else { 0 }
        } else {
            let n = v.wrapping_sub(step);
            if sign(n) { 0 } else { n }
        }
    }
}

/// `drive::steer_inputs` on the two steering values (`unk_71A61C`, `unk_71A620`).
pub fn steer(s: &SteerInputs, values: [u32; 2]) -> Steered {
    let [mut first, mut second] = values;
    let mut keys = s.word == 0;
    if !keys && s.mode != 2 {
        keys = s.keys & 0x1000 != 0;
    }
    if keys {
        let step = fistp64(0xA0_0000 as f64 * s.key_factor as f64) as u32;
        second = by_keys(second, s.word, step);
        first = by_keys(first, s.word, step);
    } else {
        let product = i64::from(s.word as i32) * i64::from(RIGHT as i32);
        let mut v = fistp64((product >> 8) as u32 as i32 as f64 * s.device_factor as f64) as u32;
        if sign(v.wrapping_sub(LEFT)) {
            v = LEFT;
        }
        if !sign(v.wrapping_sub(RIGHT)) {
            v = RIGHT;
        }
        first = v;
        second = v;
    }
    let mut trim = fistp64(RIGHT as f64 * s.trim as f64) as u32;
    if s.flags & 0x10 != 0 {
        trim = trim.wrapping_neg();
    }
    let trimmed = if below(s.figure as f64, s.limit as f64) {
        [
            first.wrapping_add(trim),
            second.wrapping_add(trim.wrapping_mul(4)),
        ]
    } else {
        [0, 0]
    };
    Steered {
        values: [first, second],
        trim,
        trimmed,
        scratch: RIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> SteerInputs {
        SteerInputs {
            word: 0,
            mode: 1,
            keys: 0,
            device_factor: 0.5,
            key_factor: 1.0,
            trim: 0.0,
            flags: 0,
            figure: 1.0,
            limit: 100.0,
        }
    }

    #[test]
    fn keys() {
        // Back toward 0 at twice the step, stopping there.
        let s = steer(&inputs(), [0x150_0000, 0xFFF0_0000]);
        assert_eq!(s.values, [0x10_0000, 0]);
        assert_eq!(s.scratch, RIGHT);
        // Right, held at the end.
        let right = SteerInputs {
            word: 0x80,
            keys: 0x1000,
            ..inputs()
        };
        let s = steer(&right, [0x1FF0_0000, 0]);
        assert_eq!(s.values, [RIGHT, 0xA0_0000]);
    }

    #[test]
    fn device_and_trim() {
        let d = SteerInputs {
            word: 0x100,
            mode: 2,
            trim: 0.25,
            ..inputs()
        };
        let s = steer(&d, [5, 6]);
        // 0x100 x 0x20000000 >> 8 = 0x20000000, x 0.5.
        assert_eq!(s.values, [0x1000_0000, 0x1000_0000]);
        assert_eq!(s.trim, 0x0800_0000);
        assert_eq!(s.trimmed, [0x1800_0000, 0x3000_0000]);
        let off = SteerInputs {
            figure: f32::NAN,
            flags: 0x10,
            ..d
        };
        let s = steer(&off, [5, 6]);
        assert_eq!(s.trim, 0xF800_0000);
        assert_eq!(s.trimmed, [0x0800_0000, 0xF000_0000]);
        let held = SteerInputs { figure: 200.0, ..d };
        assert_eq!(steer(&held, [5, 6]).trimmed, [0, 0]);
    }
}
