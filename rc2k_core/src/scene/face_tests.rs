//! Whether an object's face is drawn: the four tests of `scene::list_object_face`'s own
//! (from 0x4504CB and its parts 0x452292, 0x4505BD,
//! 0x450688, 0x4506B7). A face's group is the low five bits of its byte +7. The
//! game's variables come through `FaceWorld`, each asked only where the original reads it.

use crate::Invalid;
use crate::scene::kinds::{KNOWN_KINDS, OBJECTS, find_known};

/// The bytes of a face the tests read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Face {
    /// +6: its kind.
    pub kind: u8,
    /// +7: its group in the low five bits.
    pub group: u8,
    /// +8: bits 2 and 3 its see-through, bit 0 drawn first.
    pub flags: u8,
    /// +0xB: the line kind (kind 0x1C) or what a kind's byte must be.
    pub extra: u8,
}

impl Face {
    pub fn group(self) -> u32 {
        (self.group & 0x1F) as u32
    }
}

/// The game's variables the tests read and the two they write.
pub trait FaceWorld {
    /// Whether a race is on (a game flag that is not 0).
    fn in_race(&self) -> bool;
    /// `g_objectNumber`.
    fn object(&self) -> u32;
    /// The kind of an object (the object number under 0x100).
    fn object_kind(&self, object: u32) -> u32;
    /// The known kinds of car, by place.
    fn known_kind(&self, place: usize) -> u32;
    /// The two groups a car kind's record names (records 0x134 bytes apart, by place).
    fn kind_groups(&self, place: usize) -> [u32; 2];
    /// The two groups hidden by the chosen view.
    fn hidden_groups(&self) -> [u32; 2];
    fn set_hidden_groups(&mut self, groups: [u32; 2]);
    /// The two other groups.
    fn other_groups(&self) -> [u32; 2];
    /// The group mask.
    fn group_mask(&self) -> u32;
    /// The mask kind (the developers' mode that cycles it).
    fn mask_kind(&self) -> u32;
    /// The group table's entry for a group.
    fn group_table(&self, group: u32) -> u32;
    /// Non-zero when the second mask is not used.
    fn no_second_mask(&self) -> u32;
    /// The second mask.
    fn second_mask(&self) -> u32;
    /// The lone group and whether it is on.
    fn lone_group(&self) -> u32;
    fn lone_group_on(&self) -> u32;
    /// Which of the groups to show.
    fn groups_which(&self) -> u32;
    /// Whether see-through is on, and its value.
    fn see_through_on(&self) -> u32;
    fn see_through_value(&self) -> u32;
    /// `g_objectSeeThrough`.
    fn set_see_through(&mut self, value: u32);
    /// The dword the line kind picks at +0x6FC of the block of the car being drawn;
    /// refused when there is no block.
    fn line_value(&self, extra: u32) -> Result<u32, Invalid>;
    /// The byte the face kind selects in the kind table.
    fn kind_byte(&self, kind: u32) -> u32;
}

/// The tests in the original's order; Ok(true): the face is dropped. Refuses an object's
/// number past the tables (outside a race) and no block for a face of kind 0x1C.
pub fn dropped(w: &mut impl FaceWorld, face: Face) -> Result<bool, Invalid> {
    let group = face.group();
    let object = w.object();
    let in_race = w.in_race();
    // 1. The known kinds' hidden groups, outside a race.
    if !in_race {
        if object >= OBJECTS {
            return Err(Invalid("an object's number past the tables"));
        }
        let kind = w.object_kind(object);
        let place = find_known(|n| w.known_kind(n), kind);
        if place < KNOWN_KINDS {
            let groups = w.kind_groups(place as usize);
            w.set_hidden_groups(groups);
            if groups.contains(&group) {
                return Ok(true);
            }
        }
    }
    // 2. The masks of the objects from 0x50 whose number divides by 0x14.
    if object >= 0x50 && object.is_multiple_of(0x14) {
        let bit = |mask: u32| mask >> group & 1 != 0;
        if !bit(w.group_mask()) {
            return Ok(true);
        }
        if w.mask_kind() == 5 {
            if w.group_table(group) == 0 {
                return Ok(true);
            }
        } else if w.no_second_mask() == 0 && !bit(w.second_mask()) {
            return Ok(true);
        }
        if group == w.lone_group() && w.lone_group_on() == 0 {
            return Ok(true);
        }
        let which = w.groups_which();
        if w.hidden_groups().contains(&group) {
            if which == 1 {
                return Ok(true);
            }
        } else if w.other_groups().contains(&group) && which == 0 {
            return Ok(true);
        }
    }
    // 3. See-through faces.
    if (face.flags >> 2) & 3 != 0 && w.see_through_on() != 0 {
        let value = w.see_through_value();
        if value == 0 {
            return Ok(true);
        }
        w.set_see_through(value);
    }
    // 4. The kinds.
    let (kind, extra) = (face.kind as u32, face.extra as u32);
    if kind == 0x1C {
        if !in_race || extra == 0 || extra > 6 {
            return Ok(true);
        }
        let value = w.line_value(extra)?;
        if value == 0x8000_0000 || value < 0xA {
            return Ok(true);
        }
    } else if kind != 0 && extra != w.kind_byte(kind) {
        return Ok(true);
    }
    Ok(false)
}

/// The listed entry's key: the sort key plus bit 30 unless bit 0 of the face's +8 is set.
pub fn listed_key(face: Face, key: u32) -> u32 {
    ((((face.flags & 1) ^ 1) as u32) << 30).wrapping_add(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct World {
        in_race: bool,
        object: u32,
        hidden: [u32; 2],
        see_through: Option<u32>,
        values: [u32; 12],
        line: u32,
    }

    impl FaceWorld for World {
        fn in_race(&self) -> bool {
            self.in_race
        }
        fn object(&self) -> u32 {
            self.object
        }
        fn object_kind(&self, _: u32) -> u32 {
            0x31
        }
        fn known_kind(&self, place: usize) -> u32 {
            if place == 3 { 0x30 } else { 0 }
        }
        fn kind_groups(&self, _: usize) -> [u32; 2] {
            [4, 5]
        }
        fn hidden_groups(&self) -> [u32; 2] {
            self.hidden
        }
        fn set_hidden_groups(&mut self, groups: [u32; 2]) {
            self.hidden = groups;
        }
        fn other_groups(&self) -> [u32; 2] {
            [6, 7]
        }
        fn group_mask(&self) -> u32 {
            self.values[0]
        }
        fn mask_kind(&self) -> u32 {
            self.values[1]
        }
        fn group_table(&self, _: u32) -> u32 {
            self.values[2]
        }
        fn no_second_mask(&self) -> u32 {
            self.values[3]
        }
        fn second_mask(&self) -> u32 {
            self.values[4]
        }
        fn lone_group(&self) -> u32 {
            self.values[5]
        }
        fn lone_group_on(&self) -> u32 {
            self.values[6]
        }
        fn groups_which(&self) -> u32 {
            self.values[7]
        }
        fn see_through_on(&self) -> u32 {
            self.values[8]
        }
        fn see_through_value(&self) -> u32 {
            self.values[9]
        }
        fn set_see_through(&mut self, value: u32) {
            self.see_through = Some(value);
        }
        fn line_value(&self, _: u32) -> Result<u32, Invalid> {
            Ok(self.line)
        }
        fn kind_byte(&self, _: u32) -> u32 {
            self.values[10]
        }
    }

    #[test]
    fn tests() {
        let face = |kind, group, flags, extra| Face {
            kind,
            group,
            flags,
            extra,
        };
        // A known kind's hidden group, outside a race.
        let mut w = World::default();
        assert_eq!(dropped(&mut w, face(0, 4, 0, 0)), Ok(true));
        assert_eq!(w.hidden, [4, 5]);
        assert_eq!(dropped(&mut w, face(0, 0x23, 0, 0)), Ok(false));
        w.object = 0x100;
        assert!(dropped(&mut w, face(0, 3, 0, 0)).is_err());
        // The masks.
        let mut w = World {
            in_race: true,
            object: 0x64,
            ..World::default()
        };
        assert_eq!(dropped(&mut w, face(0, 3, 0, 0)), Ok(true));
        w.values[0] = 8;
        w.values[3] = 1;
        w.values[6] = 1;
        assert_eq!(dropped(&mut w, face(0, 3, 0, 0)), Ok(false));
        w.values[1] = 5;
        assert_eq!(dropped(&mut w, face(0, 3, 0, 0)), Ok(true));
        // See-through.
        let mut w = World {
            in_race: true,
            object: 1,
            ..World::default()
        };
        w.values[8] = 1;
        assert_eq!(dropped(&mut w, face(0, 0, 4, 0)), Ok(true));
        w.values[9] = 0x80;
        assert_eq!(dropped(&mut w, face(0, 0, 4, 0)), Ok(false));
        assert_eq!(w.see_through, Some(0x80));
        // Lines and kinds.
        w.line = 10;
        assert_eq!(dropped(&mut w, face(0x1C, 0, 0, 3)), Ok(false));
        assert_eq!(dropped(&mut w, face(0x1C, 0, 0, 7)), Ok(true));
        w.line = 0x8000_0000;
        assert_eq!(dropped(&mut w, face(0x1C, 0, 0, 3)), Ok(true));
        w.values[10] = 2;
        assert_eq!(dropped(&mut w, face(5, 0, 0, 2)), Ok(false));
        assert_eq!(dropped(&mut w, face(5, 0, 0, 1)), Ok(true));
        assert_eq!(listed_key(face(0, 0, 0, 0), 5), 0x4000_0005);
        assert_eq!(listed_key(face(0, 0, 1, 0), 5), 5);
    }
}
