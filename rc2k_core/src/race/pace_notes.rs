//! The pace notes' tables made and their texts written (converted 2026-10-21 from
//! `race::init_pace_notes` 0x441A66 and `race::pace_note_calls` 0x441CB4, with the codes'
//! rules of `race::pace`). The functions run on a machine of the registers the original
//! keeps its walk in and an image of the tables (0x615E10 .. 0x61FE18: the notes' count and
//! the groups', the notes' starts, ends, end codes and gaps, the groups' checkpoints and
//! text places, the text); the stage (the checkpoints' faces, distances, angles, the
//! slots' offsets, the distance words, the faces' bytes) is asked through `Stage`. Every
//! place read is checked: a table place outside the image, a checkpoint 0x1418 or past, a
//! face outside the course's window: refused (the original reads elsewhere). The
//! original's own slips are kept (`notes/bugs.md`): the last group is walked from the
//! place after the last note, the last group's text has no NUL, a corner after a corner
//! in one walk reads the tables at the first one's remainder (edx), 'b' moves the text on
//! without writing.

use crate::Invalid;
use crate::race::pace::{
    B, C, D, JOIN, code_place, corner, corner_start, distance_tens, ends_note, next_slot,
    replaces_join, starts_note,
};

/// `g_checkpointFaces`' room (and the distances' and angles').
pub const MOST_CHECKPOINTS: u32 = 0x1418;

/// The tables as IDA shows them.
pub const IMAGE_START: u32 = 0x615E10;
pub const IMAGE_END: u32 = 0x61FE18;
pub const COUNT: u32 = 0x615E10; // dword_615E10
pub const GROUPS: u32 = 0x615E14; // dword_615E14
pub const STARTS: u32 = 0x615E18; // dword_615E18
pub const ENDS: u32 = 0x616E18; // dword_616E18
pub const KINDS: u32 = 0x617E18; // dword_617E18
pub const GAPS: u32 = 0x618E18; // dword_618E18
pub const GROUP_CHECKPOINTS: u32 = 0x619E18; // dword_619E18
pub const GROUP_TEXTS: u32 = 0x61AE18; // dword_61AE18
pub const TEXT: u32 = 0x61BE18; // byte_61BE18
pub const TEXT_LEN: u32 = 0x4000;

/// The tables' image's length.
pub const IMAGE_LEN: usize = (IMAGE_END - IMAGE_START) as usize;

/// What the walk asks of the stage.
pub trait Stage {
    /// Checkpoint k's face number (`g_checkpointFaces`; k below 0x1418).
    fn checkpoint_face(&self, k: u32) -> u32;
    /// Checkpoint k's distance along the line (`g_checkpointDistances`; k below 0x1418).
    fn distance(&self, k: u32) -> u32;
    /// Checkpoint k's angle (`dword_9ED808`; k below 0x1418).
    fn angle(&self, k: u32) -> u32;
    /// A slot's byte in the face (`dword_614D14`; the slot below 8).
    fn slot_offset(&self, slot: u32) -> u32;
    /// The distance's word by tens (`byte_614D38`; at most 40).
    fn word(&self, tens: u32) -> u8;
    /// A byte of the course's faces (at a place inside `Course::window`).
    fn face_byte(&self, at: u32) -> u8;
}

/// Where the course is: the faces (`g_courseFaces`), where they may be read, the
/// checkpoints' count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Course {
    pub faces: u32,
    pub window: (u32, u32),
    pub count: u32,
}

/// The registers the walk keeps its state in (what the original leaves in them).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
}

/// The registers, the slot (`dword_614D34`) and the tables' image.
pub struct Notes<'a, S: Stage> {
    pub r: Registers,
    pub slot: u32,
    pub image: Vec<u8>,
    course: Course,
    stage: &'a S,
}

impl<'a, S: Stage> Notes<'a, S> {
    /// The machine on the tables' image (`IMAGE_LEN` bytes).
    pub fn new(r: Registers, slot: u32, image: Vec<u8>, course: Course, stage: &'a S) -> Self {
        assert_eq!(image.len(), IMAGE_LEN);
        Self {
            r,
            slot,
            image,
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

    pub fn get(&self, at: u32) -> Result<u32, Invalid> {
        let o = self.place(at, 4)?;
        Ok(u32::from_le_bytes(self.image[o..o + 4].try_into().unwrap()))
    }

    pub fn put(&mut self, at: u32, v: u32) -> Result<(), Invalid> {
        let o = self.place(at, 4)?;
        self.image[o..o + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn get8(&self, at: u32) -> Result<u8, Invalid> {
        Ok(self.image[self.place(at, 1)?])
    }

    pub fn put8(&mut self, at: u32, v: u8) -> Result<(), Invalid> {
        let o = self.place(at, 1)?;
        self.image[o] = v;
        Ok(())
    }

    fn by_checkpoint(&self, k: u32) -> Result<u32, Invalid> {
        if k >= MOST_CHECKPOINTS {
            return Err(Invalid("a pace note place past g_checkpointFaces' room"));
        }
        Ok(k)
    }

    /// Checkpoint k's face: (its number, its address) as `mov ebx, g_checkpointFaces[k*4];
    /// mov esi, ebx; shl esi, 6; add esi, g_courseFaces`.
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

    /// `sub_441A37` (`race::pace_note_code`): eax the face esi's code at the slot.
    pub fn code(&mut self) -> Result<(), Invalid> {
        let place = code_place(self.r.esi, self.slot, |slot| self.stage.slot_offset(slot))?;
        self.r.eax = self.face_byte(place)?;
        Ok(())
    }

    /// `sub_441A4B` (`race::next_pace_slot`): the slot on; past 7 back to 0, ebp on.
    pub fn next_slot(&mut self) {
        (self.slot, self.r.ebp) = next_slot(self.slot, self.r.ebp);
    }

    /// The place ebp x 8 + the slot.
    fn here(&self) -> u32 {
        (self.r.ebp << 3).wrapping_add(self.slot)
    }

    /// `sub_441C78` (`race::pace_distance_word`): the distance eax (16.16) / 10, at most
    /// 40, its word; unless 0, at the text's edi, edi on.
    pub fn distance_word(&mut self) -> Result<(), Invalid> {
        let word = self.stage.word(distance_tens(self.r.eax));
        if word != 0 {
            self.put8(TEXT.wrapping_add(self.r.edi), word)?;
            self.r.edi = self.r.edi.wrapping_add(1);
        }
        Ok(())
    }

    /// `sub_441C6D` / `sub_441E01` (the text's NUL / ')' and edi on).
    fn append(&mut self, byte: u8) -> Result<(), Invalid> {
        self.put8(TEXT.wrapping_add(self.r.edi), byte)?;
        self.r.edi = self.r.edi.wrapping_add(1);
        Ok(())
    }

    /// `sub_441CB4` (`race::pace_note_calls`): from place ebx, each code to the next 0 or
    /// 'd' into the text at edi; edx the note before (its +1's end and start are read for
    /// a corner). ecx and edx kept; ebx, esi, eax where it stopped; ebp a corner's flags.
    pub fn note_calls(&mut self) -> Result<(), Invalid> {
        let (ecx, edx) = (self.r.ecx, self.r.edx);
        loop {
            let place = self.r.ebx;
            self.r.esi = self.face(place >> 3)?.1;
            self.slot = place & 7;
            self.code()?;
            if self.r.eax == 0 || self.r.eax == D {
                break;
            }
            let kept = self.r.ebx;
            if self.r.eax == C {
                self.corner()?;
            }
            let al = self.r.eax as u8;
            if al as u32 != B {
                if replaces_join(al)
                    && self.get8(TEXT.wrapping_sub(1).wrapping_add(self.r.edi))? == JOIN
                {
                    self.r.edi = self.r.edi.wrapping_sub(1);
                }
                // (`mov ebx, off_61FE18[eax*4]`: overwritten by the `pop ebx`.)
                self.put8(TEXT.wrapping_add(self.r.edi), al)?;
            }
            self.r.ebx = kept.wrapping_add(1);
            self.r.edi = self.r.edi.wrapping_add(1);
        }
        self.r.ecx = ecx;
        self.r.edx = edx;
        Ok(())
    }

    /// 0x441CFB: a corner's code into eax (ecx, edx, ebx, ebp changed).
    fn corner(&mut self) -> Result<(), Invalid> {
        let n = self.r.edx.wrapping_mul(4);
        let next_end = ENDS.wrapping_add(4).wrapping_add(n);
        let next_start = STARTS.wrapping_add(4).wrapping_add(n);
        let e = self.get(next_end)? >> 3;
        let s = self.get(next_start)? >> 3;
        let length = self.distance(e)?.wrapping_sub(self.distance(s)?);
        let at_end = self.angle(self.get(next_end)? >> 3)?;
        let ebx = corner_start(self.get(next_start)?);
        let c = corner(length, at_end.wrapping_sub(self.angle(ebx)?));
        let r = &mut self.r;
        r.edx = c.remainder;
        r.ecx = c.length;
        r.ebx = ebx;
        r.ebp = c.flags;
        r.eax = c.code;
        Ok(())
    }

    /// `sub_441A66` (`race::init_pace_notes`): the tables made.
    pub fn init(&mut self) -> Result<(), Invalid> {
        let count = self.course.count;
        self.put(GROUPS, 0)?;
        self.put(COUNT, 0)?;
        if count < 2 {
            return Ok(());
        }
        // The first checkpoint whose face is of kind 2 (or the count).
        self.r.ebp = 0;
        loop {
            (self.r.ebx, self.r.esi) = self.face(self.r.ebp)?;
            if self.face_byte(self.r.esi.wrapping_add(0x26))? == 2 {
                break;
            }
            self.r.ebp = self.r.ebp.wrapping_add(1);
            if self.r.ebp == count {
                break;
            }
        }
        self.r.ebp = self.r.ebp.wrapping_add(2) << 3;
        self.put(STARTS, self.r.ebp)?;
        self.r.ebp = self.r.ebp.wrapping_add(1);
        self.put(ENDS, self.r.ebp)?;
        self.r.ebp >>= 3;
        self.put(GROUPS, 0)?;
        self.put(COUNT, 1)?;
        self.slot = 0;
        // Its loops end only on ebp equal to the count (tested after a slot moved on
        // without a note's end or start): past it they would run on.
        let runs_on = || Invalid("the pace notes' search past the checkpoints' count");
        'notes: loop {
            self.r.edx = self.get(COUNT)?;
            loop {
                (self.r.ebx, self.r.esi) = self.face(self.r.ebp)?;
                self.code()?;
                let e = self.r.eax;
                if starts_note(e) {
                    self.r.eax = self.here();
                    self.put(STARTS.wrapping_add(self.r.edx.wrapping_mul(4)), self.r.eax)?;
                    self.next_slot();
                    break;
                }
                self.next_slot();
                if self.r.ebp == count {
                    break 'notes;
                }
                if self.r.ebp > count {
                    return Err(runs_on());
                }
            }
            loop {
                (self.r.ebx, self.r.esi) = self.face(self.r.ebp)?;
                self.code()?;
                let e = self.r.eax;
                if ends_note(e) {
                    let n = self.r.edx.wrapping_mul(4);
                    self.put(KINDS.wrapping_add(n), e)?;
                    self.r.eax = self.here();
                    self.put(ENDS.wrapping_add(n), self.r.eax)?;
                    self.put(COUNT, self.get(COUNT)?.wrapping_add(1))?;
                    self.next_slot();
                    continue 'notes;
                }
                self.next_slot();
                if self.r.ebp == count {
                    break 'notes;
                }
                if self.r.ebp > count {
                    return Err(runs_on());
                }
            }
        }
        if self.get(COUNT)? < 1 {
            return Ok(());
        }
        // Each note's distance to the next start.
        self.r.edx = 0;
        loop {
            let n = self.r.edx.wrapping_mul(4);
            self.get(STARTS.wrapping_add(n))?;
            self.r.ebx = self.get(STARTS.wrapping_add(4).wrapping_add(n))? >> 3;
            self.r.eax = self.distance(self.r.ebx)?;
            self.r.ebx = self.get(ENDS.wrapping_add(n))? >> 3;
            self.r.eax = self.r.eax.wrapping_sub(self.distance(self.r.ebx)?);
            self.put(GAPS.wrapping_add(n), self.r.eax)?;
            self.r.edx = self.r.edx.wrapping_add(1);
            if self.r.edx == self.get(COUNT)? {
                break;
            }
        }
        // The groups.
        self.r.ecx = 0;
        self.r.edx = 0;
        self.r.edi = 0;
        loop {
            let n = self.r.edx.wrapping_mul(4);
            self.r.ebx = self.get(ENDS.wrapping_add(n))? >> 3;
            let g = self.r.ecx.wrapping_mul(4);
            self.put(GROUP_CHECKPOINTS.wrapping_add(g), self.r.ebx)?;
            self.put(GROUP_TEXTS.wrapping_add(g), self.r.edi)?;
            self.r.eax = self.get(GAPS.wrapping_add(n))?;
            self.distance_word()?;
            self.r.ebx = self.get(STARTS.wrapping_add(4).wrapping_add(n))?;
            self.note_calls()?;
            loop {
                self.r.edx = self.r.edx.wrapping_add(1);
                if self.r.edx == self.get(COUNT)? {
                    self.put(GROUPS, self.r.ecx)?;
                    self.put(GROUPS, self.get(GROUPS)?.wrapping_add(1))?;
                    return Ok(());
                }
                let n = self.r.edx.wrapping_mul(4);
                self.r.eax = self.get(GAPS.wrapping_add(n))?;
                if self.r.eax > 0x1E_0000 {
                    break;
                }
                self.r.eax = self.get(KINDS.wrapping_add(n))?;
                if self.r.eax == B {
                    break;
                }
                self.append(JOIN)?;
                self.r.ebx = self.get(STARTS.wrapping_add(4).wrapping_add(n))?;
                self.note_calls()?;
            }
            self.append(0)?;
            self.r.ecx = self.r.ecx.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stage of `n` checkpoints: checkpoint k's face is k, 64 bytes each from 0x1000;
    /// slot s's byte is s; the codes by checkpoint as given.
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
        let mut m = Notes::new(
            Registers::default(),
            5,
            vec![0; IMAGE_LEN],
            course(6),
            &stage,
        );
        m.init().unwrap();
        // Entry 0 the start (checkpoint 0 + 2, slot 0) and its end (+1).
        assert_eq!(m.get(STARTS).unwrap(), 2 << 3);
        assert_eq!(m.get(ENDS).unwrap(), (2 << 3) + 1);
        assert_eq!(m.get(COUNT).unwrap(), 3);
        assert_eq!(m.get(STARTS + 4).unwrap(), (2 << 3) + 1);
        assert_eq!(m.get(ENDS + 4).unwrap(), (2 << 3) + 3);
        assert_eq!(m.get(KINDS + 4).unwrap(), D);
        assert_eq!(m.get(STARTS + 8).unwrap(), 3 << 3);
        assert_eq!(m.get(ENDS + 8).unwrap(), (3 << 3) + 2);
        assert_eq!(m.get(GROUPS).unwrap(), 2);
        // The first group: the gap's word (tens 0), the first note's code, ')', the
        // second's, a NUL; the second (walked from the place after the last note, the
        // table's 0) only its gap's word (wrapped: the last, 40), no NUL.
        assert_eq!(m.get(GROUP_TEXTS).unwrap(), 0);
        assert_eq!(m.get(GROUP_TEXTS + 4).unwrap(), 5);
        assert_eq!(m.get(GROUP_CHECKPOINTS + 4).unwrap(), 3);
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
        let mut m = Notes::new(
            Registers::default(),
            0,
            vec![0; IMAGE_LEN],
            course(3),
            &stage,
        );
        // No face of kind 2: the start is the count + 2, past the window.
        assert!(m.init().is_err());
        assert!(m.get(IMAGE_END - 2).is_err());
        assert!(m.put8(IMAGE_START - 1, 0).is_err());
        let few = Test {
            codes: vec![[0; 8]],
            kinds: vec![0],
        };
        let mut m = Notes::new(
            Registers::default(),
            0,
            vec![0xAA; IMAGE_LEN],
            course(1),
            &few,
        );
        m.init().unwrap();
        assert_eq!((m.get(COUNT), m.get(GROUPS)), (Ok(0), Ok(0)));
    }
}
