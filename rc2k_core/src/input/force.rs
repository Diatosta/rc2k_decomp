//! A device's force-feedback effects as DirectInput is told of them
//! (from `input::effect_found` 0x40953B and `input::make_effect` 0x40992F's parameters).
//!
//! A device record has six effects, each of a kind (1 constant, 2 ramp, 3 periodic, 4
//! condition) and maybe a GUID it wants. While DirectInput lists the device's effects,
//! each of the six takes the last listed of its kind, unless it has already found the one
//! it wants. Its `DIEFFECT` is then filled the same for all six, with its kind's
//! type-specific parameters.

/// A GUID's 16 bytes.
pub type Guid = [u8; 16];

/// What `EnumEffects` tells of one of the device's effects (`DIEFFECTINFOA`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Listed {
    pub guid: Guid,
    pub eff_type: u32,
    pub static_params: u32,
    pub dynamic_params: u32,
}

/// What an effect has found (its +0xC .. +0x28).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Found {
    /// 1 when one was found.
    pub found: u32,
    pub listed: Listed,
}

/// `input::effect_found` for one effect of kind `kind`: whether it takes `listed` (of the
/// type's low byte its kind), unless what it found is the GUID it wants (`wanted`, asked
/// only then). Taken: found 1 and the listed effect's GUID, type and parameters.
pub fn take_listed(kind: u32, f: &mut Found, listed: &Listed, wanted: impl Fn() -> Guid) -> bool {
    if listed.eff_type & 0xFF != kind {
        return false;
    }
    if f.found == 1 && f.listed.guid == wanted() {
        return false;
    }
    *f = Found {
        found: 1,
        listed: *listed,
    };
    true
}

/// `DIEFFECT`'s figures, the same for the six: its size, flags (cartesian, by object
/// offsets), for ever, no sample period, full gain, no trigger, one axis.
pub const EFFECT_SIZE: u32 = 0x34;
pub const EFFECT_FLAGS: u32 = 0x12;
pub const FOR_EVER: u32 = u32::MAX;
pub const FULL_GAIN: u32 = 10000;
pub const NO_BUTTON: u32 = u32::MAX;
pub const AXES_COUNT: u32 = 1;
/// The axes' offsets (only the first, x, counts) and the direction.
pub const AXES: [u32; 2] = [0, 4];
pub const DIRECTION: u32 = 0;
/// `DIENVELOPE`'s size; its four figures 0.
pub const ENVELOPE_SIZE: u32 = 0x14;

/// The type-specific parameters by kind: a constant force of 10000; a ramp from 10000 to
/// 10000; a periodic one (magnitude 10000, offset 0, phase 0, period 50000); a condition
/// (offset 0, both coefficients and saturations 10000, dead band 0); others none.
pub fn type_specific(kind: u32) -> &'static [u32] {
    match kind {
        1 => &[10000],
        2 => &[10000, 10000],
        3 => &[10000, 0, 0, 50000],
        4 => &[0, 10000, 10000, 10000, 10000, 0],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing() {
        let one = Listed {
            guid: [1; 16],
            eff_type: 0x203,
            static_params: 5,
            dynamic_params: 6,
        };
        let two = Listed {
            guid: [2; 16],
            ..one
        };
        let mut f = Found::default();
        assert!(!take_listed(1, &mut f, &one, || unreachable!()));
        assert!(take_listed(3, &mut f, &one, || unreachable!()));
        assert_eq!(f.listed, one);
        // wanting the first: kept; else the last listed
        assert!(!take_listed(3, &mut f, &two, || [1; 16]));
        assert!(take_listed(3, &mut f, &two, || [9; 16]));
        assert_eq!((f.found, f.listed.guid), (1, [2; 16]));
    }

    #[test]
    fn parameters() {
        assert_eq!(type_specific(3), &[10000, 0, 0, 50000]);
        assert_eq!(type_specific(4).len(), 6);
        assert!(type_specific(5).is_empty());
    }
}
