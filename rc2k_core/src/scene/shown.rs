//! Whether and how an object of the scene is drawn (converted 2026-10-21 from
//! `scene::draw_shown_object` 0x4503A5 with its inlined tests `scene::object_hidden_by_kind`
//! 0x4522EA and `scene::object_hidden` 0x4503DD, and from `scene::draw_scene_object`
//! 0x451E3E): the race's tests that hide an object, the menus' kind bytes, the texture set
//! and the way on.

/// The objects' tables' length.
pub const OBJECTS: u32 = 0x100;
/// The texture set of the objects drawn with the other set (`dword_639954`).
pub const OTHER_SET: u32 = 0x108;

/// What the race's tests read, each asked for where they read it: the object, the kind
/// test's switches (`dword_71C3E0`, `dword_62389C`), an object's kind (`g_objectKinds`), the
/// other way (`dword_621480`), whether the special objects are shown (`dword_609330`),
/// which of two (`dword_71B400`), the only object (`dword_635058`, plus 1) and an object's
/// face count.
pub trait RaceObject {
    fn object(&self) -> u32;
    fn kind_switch(&self) -> u32;
    fn kind_mode(&self) -> u32;
    fn kind(&self, object: u32) -> u32;
    fn other_way(&self) -> u32;
    fn shown(&self) -> u32;
    fn which(&self) -> u32;
    fn only(&self) -> u32;
    fn faces(&self, object: u32) -> u32;
}

/// What the race's tests decide, and the registers they leave (ebx the object once the
/// kind test is passed; eax the only object less 1 on the other way).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Shown {
    pub drawn: bool,
    pub ebx: Option<u32>,
    pub eax: Option<u32>,
}

/// The race's two tests: hidden by kind (0x30, 0x4E with either switch); objects 0x38 and
/// 0x3A .. 0x3F hidden on the other way, when the special objects are not shown, 0x3E
/// unless `which` is 0 and 0x3D unless it is 1; on the other way every object but the only
/// one; and an object without faces.
pub fn race_shown(o: &impl RaceObject) -> Shown {
    let mut s = Shown::default();
    let object = o.object();
    if o.kind_switch() != 0 || o.kind_mode() == 2 {
        let kind = o.kind(object);
        if kind == 0x30 || kind == 0x4E {
            return s;
        }
    }
    s.ebx = Some(object);
    let other_way = o.other_way() != 0;
    let which = o.which();
    let special = object == 0x38 || (0x3A..0x40).contains(&object);
    if special
        && (other_way
            || o.shown() == 0
            || (object == 0x3E && which != 0)
            || (object == 0x3D && which != 1))
    {
        return s;
    }
    if other_way {
        let only = o.only();
        s.eax = Some(only);
        if only != 0 {
            s.eax = Some(only - 1);
            if only - 1 != which {
                return s;
            }
        }
    }
    s.drawn = o.faces(object) != 0;
    s
}

/// `scene::clear_kind_bytes` (0x46219C): the menus' 0x100 kind dwords (`dword_5CD688`), 0
/// but the seventh, eighth and tenth, 999.
pub fn kind_bytes() -> [u32; 0x100] {
    let mut bytes = [0u32; 0x100];
    (bytes[6], bytes[7], bytes[9]) = (999, 999, 999);
    bytes
}

/// Why the texture set cannot be chosen: an object past the tables (fatal error
/// 0x80600A01), a car without its block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetError {
    TooHigh,
    NoBlock,
}

/// The texture set chosen and the registers the choosing leaves (eax what was last looked
/// at, edx the car's remainder when a car was looked at).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureSet {
    pub set: u32,
    pub eax: u32,
    pub edx: Option<u32>,
}

/// `draw_scene_object`'s texture set: outside a race the other set for an object of kind
/// 0x23A (`kind(object)`); in a race for objects 0x50 .. 0xEF (the cars, 0x14 objects each)
/// whose car's block has 0x21 at +0x24 (`car_word(car)`, `None` for a car without its
/// block). `eax` is what eax holds coming in.
pub fn texture_set(
    object: u32,
    in_race: bool,
    eax: u32,
    kind: impl FnOnce(u32) -> u32,
    car_word: impl FnOnce(u32) -> Option<u32>,
) -> Result<TextureSet, SetError> {
    if object >= OBJECTS {
        return Err(SetError::TooHigh);
    }
    let mut t = TextureSet {
        set: 0,
        eax,
        edx: None,
    };
    if !in_race {
        t.eax = kind(object);
        if t.eax == 0x23A {
            t.set = OTHER_SET;
        }
    } else {
        t.eax = object.wrapping_sub(0x50);
        if t.eax < 0xA0 {
            t.edx = Some(t.eax % 0x14);
            t.eax = car_word(t.eax / 0x14).ok_or(SetError::NoBlock)?;
            if t.eax == 0x21 {
                t.set = OTHER_SET;
            }
        }
    }
    Ok(t)
}

/// The two ways on from `draw_scene_object`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// `scene::draw_shown_object`.
    Shown,
    /// `scene::draw_described_object`.
    Described,
}

/// The way on by the scene's mode (`dword_6365C4`): none for an object without faces.
pub fn way(faces: u32, mode: u32) -> Option<Way> {
    if faces == 0 {
        return None;
    }
    match mode {
        0 => Some(Way::Shown),
        1 | 2 => Some(Way::Described),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The test's variables, all at hand.
    #[derive(Clone, Copy, Default)]
    struct Vars {
        object: u32,
        kind_switch: u32,
        kind_mode: u32,
        kind: u32,
        other_way: u32,
        shown: u32,
        which: u32,
        only: u32,
        faces: u32,
    }

    impl RaceObject for Vars {
        fn object(&self) -> u32 {
            self.object
        }
        fn kind_switch(&self) -> u32 {
            self.kind_switch
        }
        fn kind_mode(&self) -> u32 {
            self.kind_mode
        }
        fn kind(&self, _: u32) -> u32 {
            self.kind
        }
        fn other_way(&self) -> u32 {
            self.other_way
        }
        fn shown(&self) -> u32 {
            self.shown
        }
        fn which(&self) -> u32 {
            self.which
        }
        fn only(&self) -> u32 {
            self.only
        }
        fn faces(&self, _: u32) -> u32 {
            self.faces
        }
    }

    #[test]
    fn race_tests() {
        let o = Vars {
            object: 0x10,
            shown: 1,
            faces: 5,
            ..Vars::default()
        };
        assert_eq!(
            race_shown(&o),
            Shown {
                drawn: true,
                ebx: Some(0x10),
                eax: None
            }
        );
        let kind = Vars {
            kind_mode: 2,
            kind: 0x4E,
            ..o
        };
        assert_eq!(race_shown(&kind), Shown::default());
        assert!(
            race_shown(&Vars {
                kind_mode: 1,
                ..kind
            })
            .drawn
        );
        for (object, which, drawn) in [
            (0x3E, 0, true),
            (0x3E, 1, false),
            (0x3D, 1, true),
            (0x3D, 0, false),
            (0x39, 5, true),
        ] {
            assert_eq!(race_shown(&Vars { object, which, ..o }).drawn, drawn);
        }
        assert!(
            !race_shown(&Vars {
                object: 0x38,
                shown: 0,
                ..o
            })
            .drawn
        );
        let other = Vars {
            other_way: 1,
            only: 3,
            which: 2,
            ..o
        };
        assert_eq!(race_shown(&other).eax, Some(2));
        assert!(race_shown(&other).drawn);
        assert!(!race_shown(&Vars { which: 1, ..other }).drawn);
        assert_eq!(race_shown(&Vars { only: 0, ..other }).eax, Some(0));
        assert!(!race_shown(&Vars { faces: 0, ..o }).drawn);
        let b = kind_bytes();
        assert_eq!((b[5], b[6], b[7], b[8], b[9]), (0, 999, 999, 0, 999));
    }

    #[test]
    fn sets_and_ways() {
        let set = |object, in_race, word: Option<u32>| {
            texture_set(object, in_race, 7, |_| 0x23A, |_| word)
        };
        assert_eq!(set(0x100, false, None), Err(SetError::TooHigh));
        assert_eq!(set(3, false, None).map(|t| t.set), Ok(OTHER_SET));
        assert_eq!(
            set(0x4F, true, None),
            Ok(TextureSet {
                set: 0,
                eax: 0xFFFF_FFFF,
                edx: None
            })
        );
        assert_eq!(
            set(0x65, true, Some(0x21)),
            Ok(TextureSet {
                set: OTHER_SET,
                eax: 0x21,
                edx: Some(1)
            })
        );
        assert_eq!(set(0x65, true, None), Err(SetError::NoBlock));
        assert_eq!(set(0xF0, true, None).map(|t| (t.set, t.eax)), Ok((0, 0xA0)));
        assert_eq!(way(0, 0), None);
        assert_eq!(way(1, 0), Some(Way::Shown));
        assert_eq!(way(1, 2), Some(Way::Described));
        assert_eq!(way(1, 3), None);
    }
}
