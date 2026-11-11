//! The pace notes' rules (from `race::pace_note_code` 0x441A37,
//! `race::next_pace_slot` 0x441A4B,
//! `pace_distance_word` 0x441C78, and the codes and the corner of `race::pace_note_calls`
//! 0x441CB4 / `race::init_pace_notes` 0x441A66): a place on the line is a checkpoint and
//! one of its face's eight slots; a note runs from a code in 1 .. 0x63 other than 'b' to
//! the next 'b' or 'd'.

use crate::Invalid;

/// A note's end that is not joined to the next ('b').
pub const B: u32 = 0x62;
/// A corner ('c'): its code is worked out (`corner`).
pub const C: u32 = 0x63;
/// A note's end ('d').
pub const D: u32 = 0x64;
/// The text's joint between notes (')').
pub const JOIN: u8 = 0x29;

/// The slots a checkpoint's face has.
pub const SLOTS: u32 = 8;

/// `race::pace_note_code`: where the face's code for `slot` is: the face plus the slot's
/// offset (`pace_slot_offsets`[slot]`, asked only for a slot below 8). Refused past 7 (the
/// original reads past its table).
pub fn code_place(face: u32, slot: u32, offset: impl FnOnce(u32) -> u32) -> Result<u32, Invalid> {
    if slot >= SLOTS {
        return Err(Invalid("a pace note slot past 7"));
    }
    Ok(offset(slot).wrapping_add(face))
}

/// `race::next_pace_slot`: the slot on by one; at 8 or past (unsigned) back to 0 and the
/// checkpoint on by one. (slot, checkpoint).
pub fn next_slot(slot: u32, checkpoint: u32) -> (u32, u32) {
    let slot = slot.wrapping_add(1);
    if slot >= SLOTS {
        (0, checkpoint.wrapping_add(1))
    } else {
        (slot, checkpoint)
    }
}

/// The last word of the distances (`byte_614D38` has 41).
pub const MOST_TENS: u32 = 40;

/// `race::pace_distance_word`: the distance's (16.16) tens, at most 40: the word's place.
pub fn distance_tens(distance: u32) -> u32 {
    ((distance >> 16) / 10).min(MOST_TENS)
}

/// `race::init_pace_notes`: whether a code starts a note.
pub fn starts_note(code: u32) -> bool {
    code != 0 && code != B && code < D
}

/// `race::init_pace_notes`: whether a code ends a note.
pub fn ends_note(code: u32) -> bool {
    code == B || code == D
}

/// `race::pace_note_calls`: whether a code takes the place of a joint just written
/// ('/', '<', '4' .. '8').
pub fn replaces_join(code: u8) -> bool {
    code == 0x2F || code == 0x3C || (0x34..=0x38).contains(&code)
}

/// A corner's code and what the original leaves of its sums.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Corner {
    pub code: u32,
    /// + 8 for a note of 0x780000 or longer, + 0x10 turning one way.
    pub flags: u32,
    /// The length, at least 0x3C0000.
    pub length: u32,
    /// The division's remainder.
    pub remainder: u32,
}

/// `race::pace_note_calls`: the checkpoint a corner's turn is measured from: the next
/// note's start less one, held to 0 ..= 0x1869F (signed), a checkpoint (>> 3).
pub fn corner_start(start: u32) -> u32 {
    let mut s = start.wrapping_sub(1);
    if (s as i32) < 0 {
        s = 0;
    }
    if (s.wrapping_sub(0x1869F) as i32) >= 0 {
        s = 0x1869F;
    }
    s >> 3
}

/// `race::pace_note_calls`: a corner from the note's length (the distance between its
/// end and start) and its turn (the angle at its end less that at `corner_start`): the
/// turn's size x 60 over the length (held to 0x3C0000 at least) graded at 0xCCC, 0x1998,
/// 0x2664 and 0x3330. The sign tests as the original's (`cmp` / `jns`, wrapping).
pub fn corner(length: u32, turn: u32) -> Corner {
    let mut flags = 0u32;
    if (length.wrapping_sub(0x78_0000) as i32) >= 0 {
        flags += 8;
    }
    let length = if (length.wrapping_sub(0x3C_0000) as i32) < 0 {
        0x3C_0000
    } else {
        length
    };
    if (turn as i32) >= 0 {
        flags += 0x10;
    }
    let size = if (turn as i32) < 0 {
        turn.wrapping_neg()
    } else {
        turn
    };
    let product = size as u64 * 60;
    // The length is 0x3C0000 or more, or past 0x80000000 (unsigned): the quotient fits.
    let quotient = (product / length as u64) as u32;
    let grade = match quotient {
        q if q > 0x3330 => 5,
        q if q > 0x2664 => 4,
        q if q > 0x1998 => 3,
        q if q > 0xCCC => 2,
        _ => 1,
    };
    Corner {
        code: grade + flags,
        flags,
        length,
        remainder: (product % length as u64) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_places() {
        assert_eq!(code_place(0x100, 3, |s| s * 4), Ok(0x10C));
        assert_eq!(code_place(4, 7, |_| u32::MAX), Ok(3));
        assert!(code_place(0, 8, |_| unreachable!()).is_err());
    }

    #[test]
    fn places_and_words() {
        assert_eq!(next_slot(6, 3), (7, 3));
        assert_eq!(next_slot(7, 3), (0, 4));
        assert_eq!(next_slot(u32::MAX, 3), (0, 3));
        assert_eq!(next_slot(9, u32::MAX), (0, 0));
        assert_eq!(distance_tens(23 << 16), 2);
        assert_eq!(distance_tens(u32::MAX), 40);
        assert!(starts_note(0x61) && !starts_note(B) && !starts_note(0) && !starts_note(D));
        assert!(ends_note(B) && ends_note(D) && !ends_note(C));
        assert!(replaces_join(b'5') && !replaces_join(b'9'));
    }

    #[test]
    fn corners() {
        assert_eq!(corner_start(0), 0);
        assert_eq!(corner_start(0x20), 3);
        assert_eq!(corner_start(0x20_0000), 0x1869F >> 3);
        let c = corner(0x10_0000, 0x1000);
        assert_eq!((c.length, c.flags), (0x3C_0000, 0x10));
        assert_eq!(c.code, 1 + 0x10);
        let c = corner(0x78_0000, (-0x60_0000i32) as u32);
        // 0x600000 x 60 / 0x780000 = 48: grade 1.
        assert_eq!((c.flags, c.code, c.remainder), (8, 9, 0));
        let c = corner(0x3C_0000, 0x3400_0000);
        assert_eq!(c.code, 5 + 0x10);
    }
}
