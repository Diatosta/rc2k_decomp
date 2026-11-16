//! A course face tried against the work car: which faces are objects' and which of those are
//! tried, the turn words, the object's reach, and what an inside contact does by contact kind.

use crate::Invalid;

/// The object byte that marks a face touched without a contact.
pub const TOUCH_ONLY: u32 = 0x3B;

/// The object material records' room (0x80 bytes each).
pub const MATERIALS: u32 = 0x4B0;

/// What a face's first two bytes make of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceObject {
    /// Its first byte is not 0xFF: not an object's.
    NotObject,
    /// Object byte 0x3B: the contact kind 0xFF and the face touched.
    TouchOnly,
    /// An object's face.
    Object(u32),
}

/// Classifies a face from its first byte and (asked only for an object's face) its object byte.
pub fn face_object(first: u32, object: impl FnOnce() -> u32) -> FaceObject {
    if first != 0xFF {
        return FaceObject::NotObject;
    }
    match object() {
        TOUCH_ONLY => FaceObject::TouchOnly,
        n => FaceObject::Object(n),
    }
}

/// A kind no object was loaded with. Course data names such slots on several stages (stage
/// 22's face 6494 names slot 0x65); a contact of kind 0 or 0xFF does nothing.
pub const UNSET_KIND: u32 = u32::MAX;

/// Refuses a kind past the material records unless it is `UNSET_KIND`.
pub fn check_material(kind: u32) -> Result<(), Invalid> {
    if kind >= MATERIALS && kind != UNSET_KIND {
        return Err(Invalid("an object kind past the 0x4B0 material records"));
    }
    Ok(())
}

/// Whether the object's face is tried, from its flags, the stage's direction and the object:
/// Err(a leftover value) when left (flag bit 0: 1; a direction in bits 1 .. 2 that is not the
/// stage's: that direction; an object 0x38 or more but not 0x3A: the object).
pub fn object_tried(flags: u32, direction: u32, object: u32) -> Result<(), u32> {
    if flags & 1 != 0 {
        return Err(1);
    }
    let way = (flags >> 1) & 3;
    if way != 0 && way - 1 != direction {
        return Err(way - 1);
    }
    if object != 0x3A && object >= 0x38 {
        return Err(object);
    }
    Ok(())
}

/// A turn word of the face: the byte << 8, low 16 bits.
pub fn turn_word(byte: u32) -> u32 {
    (byte << 8) & 0xFFFF
}

/// Whether the contact kind looks for the object's points when the scene point is not inside.
pub fn tries_points(kind: u32) -> bool {
    matches!(kind, 9 | 8 | 6 | 5)
}

/// The object's reach: its size x the scale, the 64-bit product's low 36 bits >> 4, + 0x28000,
/// and the product's high dword.
pub fn reach(size: u32, scale: u32) -> (u32, u32) {
    let product = u64::from(size) * u64::from(scale);
    let high = (product >> 32) as u32;
    (
        ((product as u32) >> 4 | high << 28).wrapping_add(0x28000),
        high,
    )
}

/// Whether a point at `distance` is out of `reach` (by the sign of the difference).
pub fn out_of_reach(reach: u32, distance: u32) -> bool {
    (reach.wrapping_sub(distance) as i32) < 0
}

/// What an inside contact does, by the contact kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inside {
    /// Kinds 6, 9, 5, 1, 10: the car put back, its ground felt, the face touched, pushed back.
    PushBack,
    /// Kinds 2, 7, 3, 8, 4: the face touched.
    Touch,
    /// Any other kind: nothing.
    Nothing,
}

/// The inside contact's effect for a contact kind.
pub fn inside(kind: u32) -> Inside {
    match kind {
        6 | 9 | 5 | 1 | 10 => Inside::PushBack,
        2 | 7 | 3 | 8 | 4 => Inside::Touch,
        _ => Inside::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unloaded_slot_is_let_through_and_other_wild_kinds_are_not() {
        assert!(check_material(0).is_ok());
        assert!(check_material(MATERIALS - 1).is_ok());
        assert!(check_material(UNSET_KIND).is_ok());
        assert!(check_material(MATERIALS).is_err());
        assert!(check_material(UNSET_KIND - 1).is_err());
        // The slot stage 22's face names is left whatever its flags,
        assert!(object_tried(0, 0, 0x65).is_err());
        // and the kinds those faces read do nothing.
        for kind in [0, 0xFF] {
            assert!(matches!(inside(kind), Inside::Nothing));
            assert!(!tries_points(kind));
        }
    }

    #[test]
    fn objects() {
        assert_eq!(face_object(0, || panic!()), FaceObject::NotObject);
        assert_eq!(face_object(0xFF, || 0x3B), FaceObject::TouchOnly);
        assert_eq!(face_object(0xFF, || 7), FaceObject::Object(7));
        assert!(check_material(0x4AF).is_ok());
        assert!(check_material(0x4B0).is_err());
        assert_eq!(object_tried(1, 0, 3), Err(1));
        assert_eq!(object_tried(4, 0, 3), Err(1));
        assert_eq!(object_tried(4, 1, 3), Ok(()));
        assert_eq!(object_tried(0, 5, 0x39), Err(0x39));
        assert_eq!(object_tried(0, 5, 0x3A), Ok(()));
        assert_eq!(turn_word(0x1FF), 0xFF00);
    }

    #[test]
    fn reaches() {
        assert_eq!(reach(0x100, 0x10), (0x28100, 0));
        assert_eq!(reach(0x8000_0000, 0x20), (0x28000, 0x10));
        assert!(out_of_reach(5, 6));
        assert!(!out_of_reach(5, 5));
        assert!(out_of_reach(0x7FFF_FFFF, 0xFFFF_FFFF));
        assert_eq!(inside(10), Inside::PushBack);
        assert_eq!(inside(4), Inside::Touch);
        assert_eq!(inside(0), Inside::Nothing);
        assert!(tries_points(5) && !tries_points(1));
    }
}
