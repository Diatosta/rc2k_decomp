use super::*;

#[derive(Clone, Copy, Default)]
struct Vars {
    object: u32,
    kind_switch: bool,
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
    fn kind_switch(&self) -> bool {
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
    assert!(race_shown(&o));
    let kind = Vars {
        kind_mode: 2,
        kind: 0x4E,
        ..o
    };
    assert!(!race_shown(&kind));
    assert!(race_shown(&Vars {
        kind_mode: 1,
        ..kind
    }));
    for (object, which, drawn) in [
        (0x3E, 0, true),
        (0x3E, 1, false),
        (0x3D, 1, true),
        (0x3D, 0, false),
        (0x39, 5, true),
    ] {
        assert_eq!(race_shown(&Vars { object, which, ..o }), drawn);
    }
    assert!(!race_shown(&Vars {
        object: 0x38,
        shown: 0,
        ..o
    }));
    let other = Vars {
        other_way: 1,
        only: 3,
        which: 2,
        ..o
    };
    assert!(race_shown(&other));
    assert!(!race_shown(&Vars { which: 1, ..other }));
    assert!(race_shown(&Vars { only: 0, ..other }));
    assert!(!race_shown(&Vars { faces: 0, ..o }));
    let b = kind_bytes();
    assert_eq!((b[5], b[6], b[7], b[8], b[9]), (0, 999, 999, 0, 999));
}

#[test]
fn sets_and_ways() {
    let set =
        |object, in_race, word: Option<u32>| texture_set(object, in_race, |_| 0x23A, |_| word);
    assert_eq!(set(0x100, false, None), Err(SetError::TooHigh));
    assert_eq!(set(3, false, None), Ok(OTHER_SET));
    assert_eq!(set(0x4F, true, None), Ok(0));
    assert_eq!(set(0x65, true, Some(0x21)), Ok(OTHER_SET));
    assert_eq!(set(0x65, true, None), Err(SetError::NoBlock));
    assert_eq!(set(0xF0, true, None), Ok(0));
    assert_eq!(way(0, 0), None);
    assert_eq!(way(1, 0), Some(Way::Shown));
    assert_eq!(way(1, 2), Some(Way::Described));
    assert_eq!(way(1, 3), None);
}
