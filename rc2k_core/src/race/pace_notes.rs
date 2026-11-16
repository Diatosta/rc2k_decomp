//! The pace notes' tables and texts. `Notes` works on an image of the tables; the stage
//! (checkpoint faces, distances, angles, slot offsets, distance words, face bytes) comes
//! through `Stage`. Every place read is checked; an out-of-image table place, a checkpoint at
//! 0x1418 or more, or a face outside the course window is refused. The tables lie one after
//! the other in the image, so an entry past a table's end reads the next table's.
//!
//! Slips kept as is:
//! - the last group is walked from the place after the last note;
//! - the last group's text has no NUL;
//! - a corner after a corner in one walk reads the tables at the first one's division
//!   remainder in place of the note's number (`pending_note` in `write_calls`);
//! - 'b' moves the text on without writing.

use crate::Invalid;
use crate::race::pace::{
    B, C, Corner, D, JOIN, code_place, corner, corner_start, distance_tens, ends_note, next_slot,
    replaces_join, starts_note,
};

/// Room for checkpoints (faces, distances and angles).
pub const MOST_CHECKPOINTS: u32 = 0x1418;

/// The tables' places in the image.
pub const IMAGE_START: u32 = 0x615E10;
pub const IMAGE_END: u32 = 0x61FE18;
pub const COUNT: u32 = 0x615E10; // PaceNotes.note_count
pub const GROUPS: u32 = 0x615E14; // PaceNotes.group_count
pub const STARTS: u32 = 0x615E18; // PaceNotes.note_starts[0]
pub const ENDS: u32 = 0x616E18; // PaceNotes.note_ends[0]
pub const KINDS: u32 = 0x617E18; // PaceNotes.note_end_codes[0]
pub const GAPS: u32 = 0x618E18; // PaceNotes.note_gaps
pub const GROUP_CHECKPOINTS: u32 = 0x619E18; // PaceNotes.group_checkpoints[0]
pub const GROUP_TEXTS: u32 = 0x61AE18; // PaceNotes.group_texts[0]
pub const TEXT: u32 = 0x61BE18; // PaceNotes.text
pub const TEXT_LEN: u32 = 0x4000;

/// The tables' image length in bytes.
pub const IMAGE_LEN: usize = (IMAGE_END - IMAGE_START) as usize;

/// What the walk asks of the stage; `k` is below `MOST_CHECKPOINTS`, `slot` below 8.
pub trait Stage {
    /// Checkpoint k's face number.
    fn checkpoint_face(&self, k: u32) -> u32;
    /// Checkpoint k's distance along the line.
    fn distance(&self, k: u32) -> u32;
    /// Checkpoint k's angle.
    fn angle(&self, k: u32) -> u32;
    /// A slot's byte offset in the face.
    fn slot_offset(&self, slot: u32) -> u32;
    /// The distance word for `tens` (at most 40).
    fn word(&self, tens: u32) -> u8;
    /// A byte of the course's faces (inside `Course::window`).
    fn face_byte(&self, at: u32) -> u8;
}

/// Where the course is: the faces, the readable window and the checkpoint count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Course {
    pub faces: u32,
    pub window: (u32, u32),
    pub count: u32,
}

/// The pace notes' tables being made: the walk's slot, the tables' image and the text's end.
pub struct Notes<'a, S: Stage> {
    /// The slot the walk is at; kept between calls.
    pub slot: u32,
    /// The tables' image (`IMAGE_LEN` bytes, from `IMAGE_START`).
    pub image: Vec<u8>,
    /// Where the next text byte goes (offset from `TEXT`).
    text_end: u32,
    course: Course,
    stage: &'a S,
}

impl<'a, S: Stage> Notes<'a, S> {
    /// The notes on the tables' image (`IMAGE_LEN` bytes).
    pub fn new(slot: u32, image: Vec<u8>, course: Course, stage: &'a S) -> Self {
        assert_eq!(image.len(), IMAGE_LEN);
        Self {
            slot,
            image,
            text_end: 0,
            course,
            stage,
        }
    }

    fn place(&self, at: u32, len: u32) -> Result<usize, Invalid> {
        let off = at.wrapping_sub(IMAGE_START);
        if off > IMAGE_END - IMAGE_START - len {
            return Err(Invalid(
                "a pace notes table place outside 0x615E10 .. 0x61FE18",
            ));
        }
        Ok(off as usize)
    }

    fn word(&self, at: u32) -> Result<u32, Invalid> {
        let o = self.place(at, 4)?;
        Ok(u32::from_le_bytes(self.image[o..o + 4].try_into().unwrap()))
    }

    fn set_word(&mut self, at: u32, v: u32) -> Result<(), Invalid> {
        let o = self.place(at, 4)?;
        self.image[o..o + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn byte(&self, at: u32) -> Result<u8, Invalid> {
        Ok(self.image[self.place(at, 1)?])
    }

    fn set_byte(&mut self, at: u32, v: u8) -> Result<(), Invalid> {
        let o = self.place(at, 1)?;
        self.image[o] = v;
        Ok(())
    }

    /// Entry `i` of the table starting at `table` (`STARTS`, `ENDS`, `KINDS`, `GAPS`,
    /// `GROUP_CHECKPOINTS`, `GROUP_TEXTS`), read by place and wrapping.
    pub fn entry(&self, table: u32, i: u32) -> Result<u32, Invalid> {
        self.word(table.wrapping_add(i.wrapping_mul(4)))
    }

    fn set_entry(&mut self, table: u32, i: u32, v: u32) -> Result<(), Invalid> {
        self.set_word(table.wrapping_add(i.wrapping_mul(4)), v)
    }

    /// The notes' count plus one (entry 0 is the start).
    fn count(&self) -> Result<u32, Invalid> {
        self.word(COUNT)
    }

    /// Writes a byte at the text's end and moves the end on.
    fn push_text(&mut self, byte: u8) -> Result<(), Invalid> {
        self.set_byte(TEXT.wrapping_add(self.text_end), byte)?;
        self.text_end = self.text_end.wrapping_add(1);
        Ok(())
    }

    fn by_checkpoint(&self, k: u32) -> Result<u32, Invalid> {
        if k >= MOST_CHECKPOINTS {
            return Err(Invalid("a pace note place past g_checkpointFaces' room"));
        }
        Ok(k)
    }

    /// Checkpoint k's face: its number and address (number x 0x40 from the faces).
    fn face(&self, k: u32) -> Result<(u32, u32), Invalid> {
        let n = self.stage.checkpoint_face(self.by_checkpoint(k)?);
        Ok((n, (n << 6).wrapping_add(self.course.faces)))
    }

    fn face_byte(&self, at: u32) -> Result<u32, Invalid> {
        let (lo, hi) = self.course.window;
        if at < lo || at >= hi {
            return Err(Invalid("a pace note face outside the course memory"));
        }
        Ok(self.stage.face_byte(at) as u32)
    }

    fn distance(&self, k: u32) -> Result<u32, Invalid> {
        Ok(self.stage.distance(self.by_checkpoint(k)?))
    }

    fn angle(&self, k: u32) -> Result<u32, Invalid> {
        Ok(self.stage.angle(self.by_checkpoint(k)?))
    }

    /// The code in the face at `face` for the current slot.
    fn code(&self, face: u32) -> Result<u32, Invalid> {
        let place = code_place(face, self.slot, |slot| self.stage.slot_offset(slot))?;
        self.face_byte(place)
    }

    /// The code at the current slot of the checkpoint's face.
    fn code_at(&self, checkpoint: u32) -> Result<u32, Invalid> {
        self.code(self.face(checkpoint)?.1)
    }

    /// Writes the distance word for the distance (16.16) in tens, at most 40, unless it is 0.
    fn write_distance_word(&mut self, distance: u32) -> Result<(), Invalid> {
        let word = self.stage.word(distance_tens(distance));
        if word != 0 {
            self.push_text(word)?;
        }
        Ok(())
    }
    /// Writes the codes of the note starting at `start` up to its end (0 or 'd'); 'b' codes move
    /// the text on unwritten and a joint is taken back by the codes that replace one. `previous`
    /// is the note before; a corner reads the tables at its successor.
    fn write_calls(&mut self, start: u32, previous: u32) -> Result<(), Invalid> {
        // Slip kept: a corner leaves its division's remainder where the note's number was, so a
        // second corner in this walk reads the tables at that remainder.
        let mut pending_note = previous;
        let mut place = start;
        loop {
            self.slot = place & 7;
            let mut code = self.code_at(place >> 3)?;
            if code == 0 || code == D {
                return Ok(());
            }
            if code == C {
                let corner = self.corner(pending_note)?;
                pending_note = corner.remainder;
                code = corner.code;
            }
            let byte = code as u8;
            if byte as u32 != B {
                if replaces_join(byte)
                    && self.byte(TEXT.wrapping_sub(1).wrapping_add(self.text_end))? == JOIN
                {
                    self.text_end = self.text_end.wrapping_sub(1);
                }
                self.set_byte(TEXT.wrapping_add(self.text_end), byte)?;
            }
            place = place.wrapping_add(1);
            self.text_end = self.text_end.wrapping_add(1);
        }
    }

    /// A corner: its code from the length and turn of the note after `note`.
    fn corner(&self, note: u32) -> Result<Corner, Invalid> {
        let end = self.entry(ENDS, note.wrapping_add(1))? >> 3;
        let start = self.entry(STARTS, note.wrapping_add(1))?;
        let length = self.distance(end)?.wrapping_sub(self.distance(start >> 3)?);
        let at_end = self.angle(end)?;
        let turn = at_end.wrapping_sub(self.angle(corner_start(start))?);
        Ok(corner(length, turn))
    }

    /// From checkpoint x 8 + slot on, the next code `wanted` accepts with its place; both move
    /// on past it. `None` at the checkpoint count. Refused: a walk past the count.
    fn next_code(
        &mut self,
        checkpoint: &mut u32,
        wanted: impl Fn(u32) -> bool,
    ) -> Result<Option<(u32, u32)>, Invalid> {
        let count = self.course.count;
        loop {
            let code = self.code_at(*checkpoint)?;
            let place = (*checkpoint << 3).wrapping_add(self.slot);
            (self.slot, *checkpoint) = next_slot(self.slot, *checkpoint);
            if wanted(code) {
                return Ok(Some((code, place)));
            }
            if *checkpoint == count {
                return Ok(None);
            }
            if *checkpoint > count {
                return Err(Invalid(
                    "the pace notes' search past the checkpoints' count",
                ));
            }
        }
    }

    /// The first checkpoint whose face is of kind 2, or the count.
    fn first_of_kind_2(&self) -> Result<u32, Invalid> {
        let count = self.course.count;
        let mut checkpoint = 0;
        loop {
            let (_, face) = self.face(checkpoint)?;
            if self.face_byte(face.wrapping_add(0x26))? == 2 {
                return Ok(checkpoint);
            }
            checkpoint = checkpoint.wrapping_add(1);
            if checkpoint == count {
                return Ok(checkpoint);
            }
        }
    }

    /// Makes the tables: notes, gaps and groups with their texts.
    pub fn init(&mut self) -> Result<(), Invalid> {
        self.set_word(GROUPS, 0)?;
        self.set_word(COUNT, 0)?;
        if self.course.count < 2 {
            return Ok(());
        }
        self.find_notes()?;
        self.measure_gaps()?;
        self.write_groups()
    }

    /// The notes: entry 0 is the start (first kind-2 checkpoint + 2); then each note's start,
    /// end and end code, walking the faces' slots.
    fn find_notes(&mut self) -> Result<(), Invalid> {
        let start = self.first_of_kind_2()?.wrapping_add(2) << 3;
        self.set_entry(STARTS, 0, start)?;
        self.set_entry(ENDS, 0, start.wrapping_add(1))?;
        let mut checkpoint = start.wrapping_add(1) >> 3;
        self.set_word(GROUPS, 0)?;
        self.set_word(COUNT, 1)?;
        self.slot = 0;
        loop {
            let note = self.count()?;
            let Some((_, start)) = self.next_code(&mut checkpoint, starts_note)? else {
                return Ok(());
            };
            self.set_entry(STARTS, note, start)?;
            let Some((code, end)) = self.next_code(&mut checkpoint, ends_note)? else {
                return Ok(());
            };
            self.set_entry(KINDS, note, code)?;
            self.set_entry(ENDS, note, end)?;
            let count = self.count()?;
            self.set_word(COUNT, count.wrapping_add(1))?;
        }
    }

    /// Each note's distance from its end to the next note's start (the last one's to the place
    /// after it).
    fn measure_gaps(&mut self) -> Result<(), Invalid> {
        let mut note = 0u32;
        loop {
            let next_start = self.distance(self.entry(STARTS, note.wrapping_add(1))? >> 3)?;
            let end = self.distance(self.entry(ENDS, note)? >> 3)?;
            self.set_entry(GAPS, note, next_start.wrapping_sub(end))?;
            note = note.wrapping_add(1);
            if note == self.count()? {
                return Ok(());
            }
        }
    }

    /// The groups: a note starts a new one when the gap before it exceeds 0x1E0000 or the note
    /// before ended with 'b'; a group's text is the distance word, its notes' calls joined by
    /// ')', and a NUL (not after the last group).
    fn write_groups(&mut self) -> Result<(), Invalid> {
        self.text_end = 0;
        let mut group = 0u32;
        let mut note = 0u32;
        loop {
            let end = self.entry(ENDS, note)? >> 3;
            self.set_entry(GROUP_CHECKPOINTS, group, end)?;
            self.set_entry(GROUP_TEXTS, group, self.text_end)?;
            let gap = self.entry(GAPS, note)?;
            self.write_distance_word(gap)?;
            let next_start = self.entry(STARTS, note.wrapping_add(1))?;
            self.write_calls(next_start, note)?;
            loop {
                note = note.wrapping_add(1);
                if note == self.count()? {
                    self.set_word(GROUPS, group)?;
                    self.set_word(GROUPS, self.word(GROUPS)?.wrapping_add(1))?;
                    return Ok(());
                }
                if self.entry(GAPS, note)? > 0x1E_0000 || self.entry(KINDS, note)? == B {
                    break;
                }
                self.push_text(JOIN)?;
                let next_start = self.entry(STARTS, note.wrapping_add(1))?;
                self.write_calls(next_start, note)?;
            }
            self.push_text(0)?;
            group = group.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stage of `n` checkpoints: face k is k, 64 bytes each from 0x1000; slot s's byte is s.
    struct Test {
        codes: Vec<[u8; 8]>,
        kinds: Vec<u8>,
    }

    const FACES: u32 = 0x1000;

    impl Stage for Test {
        fn checkpoint_face(&self, k: u32) -> u32 {
            k
        }
        fn distance(&self, k: u32) -> u32 {
            k * 0x10_0000
        }
        fn angle(&self, k: u32) -> u32 {
            k * 0x100
        }
        fn slot_offset(&self, slot: u32) -> u32 {
            slot
        }
        fn word(&self, tens: u32) -> u8 {
            0xA0 + tens as u8
        }
        fn face_byte(&self, at: u32) -> u8 {
            let off = at - FACES;
            let (face, byte) = ((off / 64) as usize, off % 64);
            match byte {
                0..8 => self.codes[face][byte as usize],
                0x26 => self.kinds[face],
                _ => 0,
            }
        }
    }

    fn course(n: u32) -> Course {
        Course {
            faces: FACES,
            window: (FACES, FACES + 64 * n),
            count: n,
        }
    }

    #[test]
    fn tables_and_text() {
        let mut codes = vec![[0u8; 8]; 6];
        codes[2][1] = 0x10; // a note from 2.1 ..
        codes[2][3] = D as u8; // .. to 2.3
        codes[3][0] = 0x20; // a second, joined (gap 0x100000) ..
        codes[3][2] = B as u8; // .. ended by 'b'
        let stage = Test {
            codes,
            kinds: vec![2, 0, 0, 0, 0, 0],
        };
        let mut m = Notes::new(5, vec![0; IMAGE_LEN], course(6), &stage);
        m.init().unwrap();
        // Entry 0 the start (checkpoint 0 + 2, slot 0) and its end (+1).
        assert_eq!(m.entry(STARTS, 0).unwrap(), 2 << 3);
        assert_eq!(m.entry(ENDS, 0).unwrap(), (2 << 3) + 1);
        assert_eq!(m.count().unwrap(), 3);
        assert_eq!(m.entry(STARTS, 1).unwrap(), (2 << 3) + 1);
        assert_eq!(m.entry(ENDS, 1).unwrap(), (2 << 3) + 3);
        assert_eq!(m.entry(KINDS, 1).unwrap(), D);
        assert_eq!(m.entry(STARTS, 2).unwrap(), 3 << 3);
        assert_eq!(m.entry(ENDS, 2).unwrap(), (3 << 3) + 2);
        assert_eq!(m.word(GROUPS).unwrap(), 2);
        // First group: gap word (tens 0), first note's code, ')', second's, NUL; the second group
        // (walked from the place after the last note) only its gap word (wrapped: 40), no NUL.
        assert_eq!(m.entry(GROUP_TEXTS, 0).unwrap(), 0);
        assert_eq!(m.entry(GROUP_TEXTS, 1).unwrap(), 5);
        assert_eq!(m.entry(GROUP_CHECKPOINTS, 1).unwrap(), 3);
        assert_eq!(
            &m.image[(TEXT - IMAGE_START) as usize..][..7],
            &[0xA0, 0x10, JOIN, 0x20, 0, 0xA0 + 40, 0]
        );
    }

    #[test]
    fn refusals() {
        let stage = Test {
            codes: vec![[0x10; 8]; 3],
            kinds: vec![0; 3],
        };
        let mut m = Notes::new(0, vec![0; IMAGE_LEN], course(3), &stage);
        // No kind-2 face: the start is the count + 2, past the window.
        assert!(m.init().is_err());
        assert!(m.word(IMAGE_END - 2).is_err());
        assert!(m.set_byte(IMAGE_START - 1, 0).is_err());
        let few = Test {
            codes: vec![[0; 8]],
            kinds: vec![0],
        };
        let mut m = Notes::new(0, vec![0xAA; IMAGE_LEN], course(1), &few);
        m.init().unwrap();
        assert_eq!((m.count(), m.word(GROUPS)), (Ok(0), Ok(0)));
    }
}
