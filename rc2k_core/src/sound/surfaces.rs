//! The sounds of what a car's wheels run on, their choices (the floats that pick the wheel and
//! make the volume are the caller's).
//!
//! The four wheels are records of five dwords, one after another in the object (the fifth dword the
//! surface). The counting wheel is the front or back pair by the sign of one difference, then the
//! pair's second by the other's. A surface's set comes from a table of 16, the set from seven
//! pairs (one for each of the two sounds); a set is four sounds, or eight (the first is 1) for a car
//! whose half flag picks the half.

use crate::Invalid;

/// The wheels' records in the object.
pub const WHEELS: [u32; 4] = [0x44, 0x58, 0x6C, 0x80];
/// Surfaces in the table, pairs of sets.
pub const SURFACES: u32 = 16;
pub(crate) const SET_PAIRS: u32 = 7;

/// The wheel whose surface counts and the other of its pair (indexes into `WHEELS`), from the two
/// differences' signs (the float's sign bit read as an integer's).
pub fn wheel(across: f32, along: f32) -> (usize, usize) {
    let (mut first, second) = if !across.is_sign_negative() {
        (1, 3)
    } else {
        (0, 2)
    };
    if along.is_sign_negative() {
        first = second;
    }
    (first, second)
}

/// Surfaces 7 and 8 have their volume turned down.
pub fn quieter(surface: u32) -> bool {
    surface == 7 || surface == 8
}

/// The pair of sets of `surface` (`sets_of` the table, asked for a surface inside it). Refuses a
/// surface past the table and a pair past the seven.
pub fn set_pair(surface: u32, sets_of: impl FnOnce(u32) -> u32) -> Result<u32, Invalid> {
    if surface >= SURFACES {
        return Err(Invalid("a surface past the 16 surface sounds"));
    }
    let pair = sets_of(surface);
    if pair >= SET_PAIRS {
        return Err(Invalid("a surface set past the seven pairs"));
    }
    Ok(pair)
}

/// `sound::surface_sound`: the sound's place in the set (from 0) and the object's dword it
/// replaces (one for sets of eight, the next otherwise). `object` is the object's first
/// dword, `half` its half flag (asked only for a set of eight). Refuses a half over 1.
pub fn pick(eight: bool, object: u32, half: impl FnOnce() -> u32) -> Result<(u32, u32), Invalid> {
    let mut n = object & 3;
    if eight {
        let half = half();
        if half > 1 {
            return Err(Invalid("a set of eight with +0xC over 1"));
        }
        n += half * 4;
    }
    Ok((n, if eight { 0x144 } else { 0x148 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choices() {
        assert_eq!(wheel(0.0, 0.0), (1, 3));
        assert_eq!(wheel(-0.0, 0.0), (0, 2));
        assert_eq!(wheel(f32::from_bits(0x8000_0001), -1.0), (2, 2));
        assert_eq!(wheel(1.0, -0.0), (3, 3));
        assert!(quieter(7) && quieter(8) && !quieter(9));
        assert_eq!(set_pair(15, |s| s - 9), Ok(6));
        assert!(set_pair(16, |_| unreachable!()).is_err());
        assert!(set_pair(3, |_| 7).is_err());
        assert_eq!(pick(false, 7, || unreachable!()), Ok((3, 0x148)));
        assert_eq!(pick(true, 6, || 1), Ok((6, 0x144)));
        assert!(pick(true, 6, || 2).is_err());
    }
}
