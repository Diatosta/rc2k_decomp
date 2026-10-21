//! The car's sound object placed (converted 2026-10-21 from `physics::place_sound_e1`
//! 0x45CEF9, `place_sound_e1_other` 0x45CE32, `place_sound_e5` 0x45CF95 and
//! `place_sound_object` 0x45CD60): a sound and its band set in one of the object's two
//! slots, and the place it is heard from: the car's place (with an offset for one) less
//! the listener's, turned by the view (`g_viewTurnMatrix`) and scaled (`flt_5C5DA0`,
//! 1/65536); then the object is told to play (+0x10 1). A reading: the hits' sounds.
//!
//! Floats: the place's parts are whole numbers (`fild` / `fiadd` / `fisub`), exact in an
//! `f64`; the turn is `math::vector::rotate_point`, kept on the stack, each part times the
//! scale and stored: the same bits at the game's double precision.
//!
//! The work car's engine sound object told the car's state (converted 2026-10-21 from
//! `physics::car_sound_state` 0x45CACE): its voices' shares, the volume, the place heard
//! from (`place`, the listener the view block's or the car's own) and its motion turned
//! by the view.

use crate::Invalid;
use crate::math::vector::rotate_point;

/// One of a sound object's two slots: where the sound, its band and its place go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub sound: u32,
    pub band: u32,
    pub place: u32,
}

/// The first slot (`place_sound_e1`, `_e1_other`, `place_sound_object`).
pub const FIRST: Slot = Slot {
    sound: 0x238,
    band: 0x23C,
    place: 0x240,
};
/// The second (`place_sound_e5`).
pub const SECOND: Slot = Slot {
    sound: 0x24C,
    band: 0x250,
    place: 0x254,
};
/// The object's play switch, set to `PLAY` last.
pub const PLAYING: u32 = 0x10;
pub const PLAY: u32 = 1;
/// The sounds of `place_sound_e1` / `_e1_other` and of `place_sound_e5`.
pub const SOUND_E1: u32 = 0xE1;
pub const SOUND_E5: u32 = 0xE5;

/// The place heard from: the car's (+ `offset`) less the listener's, turned by `view`,
/// each part times `scale`.
pub fn place(
    car: [i32; 3],
    listener: [i32; 3],
    offset: Option<[i32; 3]>,
    view: &[f32; 15],
    scale: f32,
) -> [f32; 3] {
    let part = |k: usize| match offset {
        Some(o) => o[k] as f64 + car[k] as f64 - listener[k] as f64,
        None => car[k] as f64 - listener[k] as f64,
    };
    rotate_point(view, [part(0), part(1), part(2)]).map(|x| (x * scale as f64) as f32)
}

/// The roadside objects (`g_objectKinds` has 0x100) and their kinds' material records
/// (0x4B0).
pub const OBJECTS: u32 = 0x100;
pub const KINDS: u32 = 0x4B0;

/// `physics::place_sound_object`'s sound: the object `thing`'s kind (`kind_of`), its
/// material record's byte +0x24 (`byte_of`), its low four bits into the sounds' table
/// (`sound_of`). An object or kind past its table refused.
pub fn object_sound(
    thing: u32,
    kind_of: impl FnOnce(u32) -> u32,
    byte_of: impl FnOnce(u32) -> u8,
    sound_of: impl FnOnce(u32) -> u32,
) -> Result<u32, Invalid> {
    if thing >= OBJECTS {
        return Err(Invalid("an object past 0x100"));
    }
    let kind = kind_of(thing);
    if kind >= KINDS {
        return Err(Invalid("a kind past the 0x4B0 records"));
    }
    Ok(sound_of(byte_of(kind) as u32 & 0xF))
}

/// `physics::car_sound_state`'s voices' figures (also kept at `g_car` +0x28C, +0x290,
/// +0x294).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voices {
    /// +0x28C: +0x288 over `flt_613EE0`; the voices' +0x50 .. are it times a scale.
    pub share: f32,
    /// +0x290: 1.0; the voices' +0x4C ...
    pub full: f32,
    /// +0x294: the motion's figure (as it was); the voices' +0x44 ...
    pub motion: f32,
}

/// The voices' figures: all three 0 with all four wheels off the ground (`unk_71A22C`).
pub fn voices(level: f32, divisor: f32, motion: f32, all_off: bool) -> Voices {
    if all_off {
        return Voices {
            share: 0.0,
            full: 0.0,
            motion: 0.0,
        };
    }
    Voices {
        share: (f64::from(level) / f64::from(divisor)) as f32,
        full: 1.0,
        motion,
    }
}

/// A voice's pitch: the share times the scale (`flt_5C5E2C`), one product rounded once.
pub fn pitch(share: f32, scale: f32) -> f32 {
    (f64::from(share) * f64::from(scale)) as f32
}

/// The object's volume (+0x18): the car's +0x638 held to 0 ..= 0x100 (signed).
pub fn volume(v: u32) -> u32 {
    (v as i32).clamp(0, 0x100) as u32
}

/// The car's motion (+0xD4 ..) turned by the view, each part stored as a float.
pub fn turned_motion(view: &[f32; 15], motion: [f32; 3]) -> [f32; 3] {
    rotate_point(view, motion.map(f64::from)).map(|x| x as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_state() {
        let v = voices(3.0, 2.0, 7.0, false);
        assert_eq!(
            v,
            Voices {
                share: 1.5,
                full: 1.0,
                motion: 7.0
            }
        );
        assert_eq!(voices(3.0, 2.0, 7.0, true).full, 0.0);
        assert_eq!(pitch(1.5, 256.0), 384.0);
        assert_eq!(volume(0x200), 0x100);
        assert_eq!(volume(u32::MAX), 0);
        assert_eq!(volume(0x80), 0x80);
    }

    #[test]
    fn places() {
        let mut view = [0.0; 15];
        view[0] = 1.0;
        view[5] = 1.0;
        view[10] = 1.0;
        let p = place(
            [0x30000, 5, -0x10000],
            [0x10000, 5, 0],
            None,
            &view,
            1.0 / 65536.0,
        );
        assert_eq!(p, [2.0, 0.0, -1.0]);
        let p = place(
            [i32::MAX, 0, 0],
            [i32::MIN, 0, 0],
            Some([i32::MAX, 0, 0]),
            &view,
            1.0,
        );
        assert_eq!(
            p[0],
            (3.0 * i32::MAX as f64 + 1.0) as f32,
            "no wrap: whole numbers"
        );
    }

    #[test]
    fn object_sounds() {
        assert!(object_sound(0x100, |_| 0, |_| 0, |_| 0).is_err());
        assert!(object_sound(3, |_| 0x4B0, |_| 0, |_| 0).is_err());
        assert_eq!(
            object_sound(3, |t| t + 1, |k| k as u8 | 0xF0, |s| s * 10),
            Ok(40)
        );
    }
}
