//! Whether and how an object of the scene is drawn: the race's tests that hide an object, the
//! menus' kind bytes, the texture set and the way on.

/// The objects' tables' length.
pub const OBJECTS: u32 = 0x100;
/// The texture set of the objects drawn with the other set.
pub const OTHER_SET: u32 = 0x108;

/// What the race's tests read, each asked for where they read it: the object, the kind
/// test's switches, an object's kind, the other way, whether the special objects are shown,
/// which of two, the only object (plus 1) and an object's face count.
pub trait RaceObject {
    fn object(&self) -> u32;
    fn kind_switch(&self) -> bool;
    fn kind_mode(&self) -> u32;
    fn kind(&self, object: u32) -> u32;
    fn other_way(&self) -> u32;
    fn shown(&self) -> u32;
    fn which(&self) -> u32;
    fn only(&self) -> u32;
    fn faces(&self, object: u32) -> u32;
}

/// The race's two tests, whether the object is drawn: hidden by kind (0x30, 0x4E with either
/// switch); objects 0x38 and 0x3A ..= 0x3F hidden on the other way when the special objects
/// are not shown (0x3E unless `which` is 0, 0x3D unless 1); on the other way every object
/// but the only one; and an object without faces.
pub fn race_shown(o: &impl RaceObject) -> bool {
    let object = o.object();
    if (o.kind_switch() || o.kind_mode() == 2) && matches!(o.kind(object), 0x30 | 0x4E) {
        return false;
    }
    let other_way = o.other_way() != 0;
    let which = o.which();
    let special = object == 0x38 || (0x3A..0x40).contains(&object);
    if special
        && (other_way
            || o.shown() == 0
            || (object == 0x3E && which != 0)
            || (object == 0x3D && which != 1))
    {
        return false;
    }
    if other_way {
        // The only object is numbered from 1; 0 means all.
        let only = o.only();
        if only != 0 && only - 1 != which {
            return false;
        }
    }
    o.faces(object) != 0
}

/// The menus' kind bytes: 0x100 kind dwords, 0 but the seventh, eighth and tenth, 999.
pub fn kind_bytes() -> [u32; 0x100] {
    let mut bytes = [0u32; 0x100];
    (bytes[6], bytes[7], bytes[9]) = (999, 999, 999);
    bytes
}

/// Why the texture set cannot be chosen: an object past the tables, a car without its block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetError {
    TooHigh,
    NoBlock,
}

/// The texture set: outside a race the other set for an object of kind 0x23A; in a race for
/// objects 0x50 ..= 0xEF (the cars, 0x14 objects each) whose car's word is 0x21
/// (`car_word(car)`, `None` for a car without its block).
pub fn texture_set(
    object: u32,
    in_race: bool,
    kind: impl FnOnce(u32) -> u32,
    car_word: impl FnOnce(u32) -> Option<u32>,
) -> Result<u32, SetError> {
    if object >= OBJECTS {
        return Err(SetError::TooHigh);
    }
    let other = if !in_race {
        kind(object) == 0x23A
    } else {
        let car_object = object.wrapping_sub(0x50);
        car_object < 0xA0 && car_word(car_object / 0x14).ok_or(SetError::NoBlock)? == 0x21
    };
    Ok(if other { OTHER_SET } else { 0 })
}

/// The two ways an object is drawn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// Drawn as a shown object.
    Shown,
    /// Drawn as a described object.
    Described,
}

/// The way on by the scene's mode: none for an object without faces.
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
mod tests;
