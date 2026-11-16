//! Object kinds: the 0x22 known kinds of car, each with an object kind, a first picture and an
//! icon number, and the kind of each of the scene's 0x100 objects.

use crate::Invalid;

/// The known kinds of car.
pub const KNOWN_KINDS: u32 = 0x22;
/// The scene's objects.
pub const OBJECTS: u32 = 0x100;
/// The object kinds a hiding setting hides.
pub const HIDDEN_KINDS: [u32; 2] = [0x30, 0x4E];

/// The entry of a known kind's tables a number asks for: held to 0 .. 0x21, signed.
pub fn known_place(which: u32) -> usize {
    (which as i32).clamp(0, KNOWN_KINDS as i32 - 1) as usize
}

/// The place of the first known kind whose object kind (`known(place)`), or that plus 1, is
/// `kind`; `KNOWN_KINDS` for none.
pub fn find_known(known: impl Fn(usize) -> u32, kind: u32) -> u32 {
    (0..KNOWN_KINDS)
        .find(|&n| {
            let k = known(n as usize);
            kind == k || kind == k.wrapping_add(1)
        })
        .unwrap_or(KNOWN_KINDS)
}

/// The first object (1 .. 0xFF; object 0 is never looked at) whose kind (`kind_of`) is
/// `kind`, 0 for none.
pub fn first_of_kind(kind_of: impl Fn(usize) -> u32, kind: u32) -> u32 {
    (1..OBJECTS)
        .find(|&n| kind_of(n as usize) == kind)
        .unwrap_or(0)
}

/// The two settings under which some kinds are not drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HideSettings {
    pub switch: bool,
    pub mode: u32,
}

impl HideSettings {
    /// Whether kinds are hidden at all: the switch on, or the mode 2.
    pub fn on(self) -> bool {
        self.switch || self.mode == 2
    }
}

/// Whether `object` is hidden by its kind (`kind_of`, asked only with the settings on), and
/// the number looked at. Refuses an object past the table with the settings on.
pub fn kind_is_hidden(
    settings: HideSettings,
    object: u32,
    kind_of: impl FnOnce(usize) -> u32,
) -> Result<(bool, u32), Invalid> {
    if !settings.on() {
        return Ok((false, object));
    }
    if object >= OBJECTS {
        return Err(Invalid("an object past g_objectKinds (0x100)"));
    }
    let kind = kind_of(object as usize);
    Ok((HIDDEN_KINDS.contains(&kind), kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known() {
        assert_eq!(
            [0, 5, 0x21, 0x22, 0x8000_0000, u32::MAX, 0x7FFF_FFFF].map(known_place),
            [0, 5, 0x21, 0x21, 0, 0, 0x21]
        );
        let table = |n: usize| 10 * n as u32;
        assert_eq!(find_known(table, 30), 3);
        assert_eq!(find_known(table, 31), 3);
        assert_eq!(find_known(table, 32), KNOWN_KINDS);
        assert_eq!(find_known(|_| u32::MAX, 0), 0);
    }

    #[test]
    fn objects() {
        let kinds = |n: usize| (n as u32) % 7;
        assert_eq!(first_of_kind(kinds, 0), 7);
        assert_eq!(first_of_kind(kinds, 3), 3);
        assert_eq!(first_of_kind(kinds, 9), 0);
        let off = HideSettings {
            switch: false,
            mode: 3,
        };
        let on = HideSettings {
            switch: false,
            mode: 2,
        };
        assert_eq!(
            kind_is_hidden(off, 0x123, |_| unreachable!()),
            Ok((false, 0x123))
        );
        assert!(kind_is_hidden(on, 0x100, |_| 0).is_err());
        assert_eq!(kind_is_hidden(on, 4, |_| 0x4E), Ok((true, 0x4E)));
        assert_eq!(
            kind_is_hidden(
                HideSettings {
                    switch: true,
                    mode: 0
                },
                4,
                |_| 0x31
            ),
            Ok((false, 0x31))
        );
    }
}
