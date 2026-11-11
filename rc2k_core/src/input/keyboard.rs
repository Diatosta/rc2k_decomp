//! Keys made characters with a keyboard layout (from
//! `input::key_to_char` 0x407E43).
//!
//! A layout has a list of dead keys {key, plain, shifted} ended by a 0 key, and three
//! tables of a character a key: plain, shifted, and the one used while AltGr is down. A
//! dead key is kept pending; the next key is then looked up, by its plain character, in
//! the compose list {dead key, base, plain result, shifted result} ended by a 0. Keys whose
//! bit is set in the silent bitmap type nothing.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// A layout's lists and tables; `None` where the game's pointer is null.
pub struct Layout<'a> {
    pub dead_keys: Option<&'a dyn Bytes>,
    pub plain: Option<&'a dyn Bytes>,
    pub shifted: Option<&'a dyn Bytes>,
    pub alt_gr: Option<&'a dyn Bytes>,
}

/// Which layout layout mode 2 switches to: Alt or AltGr with the left shift the first,
/// with the right shift the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switch {
    First,
    Second,
}

pub fn switch(
    mode: u8,
    alt: bool,
    alt_gr: bool,
    left_shift: bool,
    right_shift: bool,
) -> Option<Switch> {
    if mode != 2 || !(alt || alt_gr) {
        return None;
    }
    if left_shift {
        Some(Switch::First)
    } else if right_shift {
        Some(Switch::Second)
    } else {
        None
    }
}

/// What a key typed: the low 16 bits as the game returns them (the low byte the character, 0 for
/// none; the next byte the dead key stored or used), and the pending dead key's new value, if it
/// changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Typed {
    pub code: u32,
    pub pending: Option<u8>,
}

/// Lists are read at most this far for their end.
const LIST_LIMIT: usize = 256;

fn at(bytes: &dyn Bytes, n: usize) -> Result<u8, Invalid> {
    bytes
        .byte(n)
        .ok_or(Invalid("a keyboard table past its end"))
}

/// The key (below 0x80) made a character. Refused (before anything changes): a dead key
/// pending with no plain table (the original reads through null), lists without an end
/// within 256 entries.
pub fn key_to_char(
    key: u8,
    layout: &Layout,
    compose: &dyn Bytes,
    silent: &dyn Bytes,
    shift: bool,
    alt_gr: bool,
    pending: u8,
) -> Result<Typed, Invalid> {
    // Is this key a dead key?
    let mut dead = 0u8;
    if let Some(list) = layout.dead_keys {
        let mut entry = 0;
        let mut ended = false;
        for _ in 0..LIST_LIMIT {
            if at(list, entry)? == key {
                dead = at(list, entry + if shift { 2 } else { 1 })?;
                ended = true;
                break;
            }
            entry += 3;
            if at(list, entry)? == 0 {
                ended = true;
                break;
            }
        }
        if !ended {
            return Err(Invalid("dead key list without an end"));
        }
    }
    let silent = dead == 0 && at(silent, key as usize / 8)? & (1 << (key % 8)) != 0;
    let composing = dead == 0 && !silent && pending != 0;
    let mut composed = None;
    if composing {
        let plain = layout
            .plain
            .ok_or(Invalid("a dead key is pending and there is no plain table"))?;
        let base = at(plain, key as usize)?;
        let mut entry = 0;
        let mut ended = false;
        for _ in 0..LIST_LIMIT {
            if at(compose, entry)? == pending && at(compose, entry + 1)? == base {
                composed = Some(at(compose, entry + if shift { 3 } else { 2 })?);
                ended = true;
                break;
            }
            entry += 4;
            if at(compose, entry)? == 0 {
                ended = true;
                break;
            }
        }
        if !ended {
            return Err(Invalid("compose list without an end"));
        }
    }
    if dead != 0 {
        return Ok(Typed {
            code: (dead as u32) << 8,
            pending: Some(dead),
        });
    }
    if silent {
        return Ok(Typed {
            code: 0,
            pending: None,
        });
    }
    let (high, pending) = if composing {
        ((pending as u32) << 8, Some(0))
    } else {
        (0, None)
    };
    if let Some(c) = composed {
        return Ok(Typed {
            code: high | c as u32,
            pending,
        });
    }
    let table = if alt_gr {
        layout.alt_gr
    } else if shift {
        layout.shifted
    } else {
        layout.plain
    };
    let code = match table {
        Some(table) => high | at(table, key as usize)? as u32,
        None => high,
    };
    Ok(Typed { code, pending })
}

/// A key's name on the keyboard pages (`input::append_key_name` 0x407F39): the layout's
/// string for the key's low seven bits, or none (0): "0x" and two hexadecimal digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyName {
    String(u32),
    Hex(u32),
}

pub fn key_name(key: u32, names: impl Fn(u32) -> u16) -> KeyName {
    let key = key & 0x7F;
    match names(key) {
        0 => KeyName::Hex(key),
        id => KeyName::String(id as u32),
    }
}

/// The answers to a yes/no question: strings 0xC3 (yes, 1), 0xC4 (no, -1), 0xC5 (2).
pub const ANSWERS: [(u32, u32); 3] = [(0xC3, 1), (0xC4, u32::MAX), (0xC5, 2)];

/// The answer a key's character (made lower case) gives (`input::yes_no_key` 0x407D71):
/// the first of the three whose string's first byte it is (`first(id)`; an empty string
/// is compared through what was there before), else 0.
pub fn answer(lower: u8, mut first: impl FnMut(u32) -> u8) -> u32 {
    ANSWERS
        .into_iter()
        .find(|&(id, _)| first(id) == lower)
        .map_or(0, |(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types() {
        let plain: Vec<u8> = (0..0x80u8).map(|k| k + 0x20).collect();
        let shifted: Vec<u8> = (0..0x80u8).map(|k| k + 0x40).collect();
        let dead = [0x1Au8, b'^', b'"', 0];
        let compose = [b'^', 0x25, 0xE2, 0xC2, 0];
        let silent = [0u8, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let layout = Layout {
            dead_keys: Some(&dead),
            plain: Some(&plain),
            shifted: Some(&shifted),
            alt_gr: None,
        };
        let typed = |key, shift, alt_gr, pending| {
            key_to_char(key, &layout, &compose, &silent, shift, alt_gr, pending).unwrap()
        };
        assert_eq!(
            typed(5, false, false, 0),
            Typed {
                code: 0x25,
                pending: None
            }
        );
        assert_eq!(typed(5, true, false, 0).code, 0x45);
        assert_eq!(
            typed(0x1A, false, false, 0),
            Typed {
                code: 0x5E00,
                pending: Some(b'^')
            }
        );
        assert_eq!(
            typed(5, false, false, b'^'),
            Typed {
                code: 0x5EE2,
                pending: Some(0)
            }
        );
        assert_eq!(
            typed(6, false, false, b'^'),
            Typed {
                code: 0x5E26,
                pending: Some(0)
            }
        );
        assert_eq!(typed(15, false, false, 0).code, 0); // silent
        assert_eq!(typed(5, false, true, 0).code, 0); // no AltGr table
        assert_eq!(switch(2, true, false, false, true), Some(Switch::Second));
        assert_eq!(switch(1, true, false, true, false), None);
    }

    #[test]
    fn names_and_answers() {
        assert_eq!(
            key_name(0x81, |k| if k == 1 { 0x55 } else { 0 }),
            KeyName::String(0x55)
        );
        assert_eq!(key_name(0x02, |_| 0), KeyName::Hex(2));
        let firsts = |id| match id {
            0xC3 => b's',
            0xC4 => b'n',
            _ => b'x',
        };
        assert_eq!((answer(b'n', firsts), answer(b'q', firsts)), (u32::MAX, 0));
    }
}
