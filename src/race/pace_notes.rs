//! The pace notes (IDA 2026-10-19): the co-driver's calls, made at a stage's loading from
//! codes kept in the checkpoints' faces ("init pacenotes", `race::init_pace_notes`
//! 0x441A66) and handed out during the stage, checkpoint by checkpoint, as car events
//! (`physics::pace_notes_step` 0x4417EC, `physics::queue_pace_note` 0x4418B3).
//!
//! Each checkpoint face holds eight slots of code (bytes +0x28, +0x29, +0x2A, +0x2B, +0x19,
//! +0x1A, +0x1B, +0x20: `dword_614D14`); a place on the line is checkpoint x 8 + slot
//! (`dword_614D34` the slot). A note runs from a code in 1 .. 0x63 other than 'b' (0x62) to
//! the next 'b' or 'd' (0x64). The tables (0x615E10 ..):
//! - `dword_615E10` the notes' count plus one (entry 0 is the start: the first checkpoint
//!   with face kind 2, +2), `dword_615E14` the groups' count;
//! - by note: `dword_615E18` start, `dword_616E18` end, `dword_617E18` the end's code,
//!   `dword_618E18` the distance (`g_checkpointDistances`) from its end to the next start;
//! - by group: `dword_619E18` the checkpoint it is called at, `dword_61AE18` its text's
//!   place in `byte_61BE18` (0x4000 bytes): a word for the distance before (by tens of
//!   `byte_614D38`), then the codes of each note walked (`race::pace_note_calls`), notes less
//!   than 0x1E0000 apart and not ended by 'b' joined by ')' (0x29), a group ended by a NUL.
//!   A corner ('c', 0x63) becomes 1 .. 5 by how sharp (the checkpoint angles'
//!   `dword_9ED808` change over the note's length times 60), + 8 for a note of 0x780000 or
//!   longer, + 0x10 turning one way.
//!
//! The functions are modelled here on a machine of the registers and an image of the
//! tables: the Rust versions read and write the image, which is written back only when the
//! function is done (so a refusal writes nothing). Every place read is checked: a table
//! place outside 0x615E10 .. 0x61FE18, a checkpoint 0x1418 or past (`g_checkpointFaces`'
//! room), a face outside the course memory: refused (the original reads elsewhere, or past
//! its memory). The original's own slips are kept (section 16): the last group is walked
//! from the place after the last note (a dangling start or a left-over of an earlier
//! stage), the last group's text has no NUL, a corner after a corner in one walk reads the
//! tables at the remainder of the first one's division (edx), 'b' moves the text on
//! without writing.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI, Regs, game_addr, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::Region;

pub const SLOT_RVA: usize = rva(0x614D34); // dword_614D34: the slot 0 .. 7
const SLOT_OFFSETS_RVA: usize = rva(0x614D14); // dword_614D14: each slot's byte in the face
const WORDS_RVA: usize = rva(0x614D38); // byte_614D38: the distance's word by tens (41)
pub const CHECKPOINT_COUNT_RVA: usize = rva(0x623640); // g_checkpointCount
pub const CHECKPOINT_FACES_RVA: usize = rva(0x9E3738); // g_checkpointFaces
pub const DISTANCES_RVA: usize = rva(0x9F78D8); // g_checkpointDistances
pub const ANGLES_RVA: usize = rva(0x9ED808); // dword_9ED808: the checkpoints' angles
pub const COURSE_FACES_RVA: usize = rva(0x5CCCBC); // g_courseFaces
const COURSE_MEMORY_RVA: usize = rva(0x5C6F94); // g_courseMemory
const COURSE_MEMORY_LEN: u32 = 0x130_0000;
/// `g_checkpointFaces`' room.
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

use rc2k_core::race::pace::{B, C, D, JOIN};

/// Where the course is: the faces (`g_courseFaces`), where they may be read, the count.
#[derive(Clone, Copy)]
pub struct Course {
    pub faces: u32,
    pub window: (u32, u32),
    pub count: u32,
}

/// A test stage's faces while one is up (start, end): the originals' calls of the hooks
/// read them there.
static TEST_WINDOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Course {
    pub fn game() -> Self {
        unsafe {
            let memory: u32 = game_read(COURSE_MEMORY_RVA);
            let faces: u32 = game_read(COURSE_FACES_RVA);
            let test = TEST_WINDOW.load(std::sync::atomic::Ordering::SeqCst);
            let (lo, hi) = (test as u32, (test >> 32) as u32);
            Self {
                faces,
                window: if test != 0 && faces >= lo && faces < hi {
                    (lo, hi)
                } else if memory == 0 {
                    (0, 0)
                } else {
                    (memory, memory.wrapping_add(COURSE_MEMORY_LEN))
                },
                count: game_read(CHECKPOINT_COUNT_RVA),
            }
        }
    }
}

/// What the pace notes' set-up writes: the slot and the tables.
pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(SLOT_RVA) as u32, 4),
        (
            game_addr(rva(IMAGE_START as usize)) as u32,
            (IMAGE_END - IMAGE_START) as usize,
        ),
    ]
}

fn refuse(why: &'static str) -> Refused {
    BadInput(why).into()
}

/// A checkpoint's entry of a table by checkpoint (0x1418 or past refused).
fn by_checkpoint(table: usize, k: u32) -> Result<u32, Refused> {
    if k >= MOST_CHECKPOINTS {
        return Err(refuse("a pace note place past g_checkpointFaces' room"));
    }
    Ok(unsafe { game_read(table + 4 * k as usize) })
}

/// The registers, the slot and the tables' image.
pub struct Machine {
    pub r: Regs,
    pub slot: u32,
    image: Vec<u8>,
    course: Course,
}

impl Machine {
    pub fn load(r: &Regs, course: Course) -> Self {
        let len = (IMAGE_END - IMAGE_START) as usize;
        let at = game_addr(rva(IMAGE_START as usize)) as *const u8;
        Self {
            r: *r,
            slot: unsafe { game_read(SLOT_RVA) },
            image: unsafe { std::slice::from_raw_parts(at, len) }.to_vec(),
            course,
        }
    }

    /// The slot and the tables written back.
    pub fn commit(&self) {
        unsafe {
            crate::framework::game_write(SLOT_RVA, self.slot);
            let at = game_addr(rva(IMAGE_START as usize)) as *mut u8;
            std::slice::from_raw_parts_mut(at, self.image.len()).copy_from_slice(&self.image);
        }
    }

    /// Every register as it stands (the caller's `Left`).
    pub fn left(&self) -> Left {
        let r = &self.r;
        Left::nothing()
            .with(EAX, r.eax)
            .with(EBX, r.ebx)
            .with(ECX, r.ecx)
            .with(EDX, r.edx)
            .with(ESI, r.esi)
            .with(EDI, r.edi)
            .with(EBP, r.ebp)
    }

    fn place(&self, at: u32, len: u32) -> Result<usize, Refused> {
        let off = at.wrapping_sub(IMAGE_START);
        if off > IMAGE_END - IMAGE_START - len {
            return Err(refuse(
                "a pace notes table place outside 0x615E10 .. 0x61FE18",
            ));
        }
        Ok(off as usize)
    }

    pub fn get(&self, at: u32) -> Result<u32, Refused> {
        let o = self.place(at, 4)?;
        Ok(u32::from_le_bytes(self.image[o..o + 4].try_into().unwrap()))
    }

    pub fn put(&mut self, at: u32, v: u32) -> Result<(), Refused> {
        let o = self.place(at, 4)?;
        self.image[o..o + 4].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn get8(&self, at: u32) -> Result<u8, Refused> {
        Ok(self.image[self.place(at, 1)?])
    }

    pub fn put8(&mut self, at: u32, v: u8) -> Result<(), Refused> {
        let o = self.place(at, 1)?;
        self.image[o] = v;
        Ok(())
    }

    /// Checkpoint k's face: (its number, its address) as `mov ebx, g_checkpointFaces[k*4];
    /// mov esi, ebx; shl esi, 6; add esi, g_courseFaces`.
    fn face(&self, k: u32) -> Result<(u32, u32), Refused> {
        let n = by_checkpoint(CHECKPOINT_FACES_RVA, k)?;
        Ok((n, (n << 6).wrapping_add(self.course.faces)))
    }

    fn face_byte(&self, at: u32) -> Result<u32, Refused> {
        let (lo, hi) = self.course.window;
        if at < lo || at >= hi {
            return Err(refuse("a pace note face outside the course memory"));
        }
        Ok(unsafe { (at as *const u8).read() } as u32)
    }

    fn distance(&self, k: u32) -> Result<u32, Refused> {
        by_checkpoint(DISTANCES_RVA, k)
    }

    fn angle(&self, k: u32) -> Result<u32, Refused> {
        by_checkpoint(ANGLES_RVA, k)
    }

    /// `sub_441A37` (`race::pace_note_code`): eax the face esi's code at the slot.
    pub fn code(&mut self) -> Result<(), Refused> {
        // Converted: `rc2k_core::race::pace::code_place`.
        let place = rc2k_core::race::pace::code_place(self.r.esi, self.slot, |slot| unsafe {
            game_read(SLOT_OFFSETS_RVA + 4 * slot as usize)
        })?;
        self.r.eax = self.face_byte(place)?;
        Ok(())
    }

    /// `sub_441A4B` (`race::next_pace_slot`): the slot on; past 7 back to 0, ebp on.
    pub fn next_slot(&mut self) {
        // Converted: `rc2k_core::race::pace::next_slot`.
        (self.slot, self.r.ebp) = rc2k_core::race::pace::next_slot(self.slot, self.r.ebp);
    }

    /// The place ebp x 8 + the slot.
    fn here(&self) -> u32 {
        (self.r.ebp << 3).wrapping_add(self.slot)
    }

    /// `sub_441C78` (`race::pace_distance_word`): the distance eax (16.16) / 10, at most
    /// 40, its word from `byte_614D38`; unless 0, at the text's edi, edi on.
    pub fn distance_word(&mut self) -> Result<(), Refused> {
        // Converted: `rc2k_core::race::pace::distance_tens` (shr 16; div 10; mul 10;
        // div 10: the tens).
        let tens = rc2k_core::race::pace::distance_tens(self.r.eax);
        let word: u8 = unsafe { game_read(WORDS_RVA + tens as usize) };
        if word != 0 {
            self.put8(TEXT.wrapping_add(self.r.edi), word)?;
            self.r.edi = self.r.edi.wrapping_add(1);
        }
        Ok(())
    }

    /// `sub_441C6D` / `sub_441E01` (the text's NUL / ')' and edi on; not hooked: one-line
    /// cursor helpers).
    fn append(&mut self, byte: u8) -> Result<(), Refused> {
        self.put8(TEXT.wrapping_add(self.r.edi), byte)?;
        self.r.edi = self.r.edi.wrapping_add(1);
        Ok(())
    }

    /// `sub_441CB4` (`race::pace_note_calls`): from place ebx, each code to the next 0 or
    /// 'd' into the text at edi (see the header); edx the note before (its +1's end and
    /// start are read for a corner). ecx and edx kept; ebx, esi, eax where it stopped;
    /// ebp a corner's flags.
    pub fn note_calls(&mut self) -> Result<(), Refused> {
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
                if rc2k_core::race::pace::replaces_join(al)
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
    fn corner(&mut self) -> Result<(), Refused> {
        // Converted: `rc2k_core::race::pace::{corner_start, corner}`.
        use rc2k_core::race::pace::{corner, corner_start};
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

    /// `sub_441A66` (`race::init_pace_notes`): the tables made (see the header).
    pub fn init(&mut self) -> Result<(), Refused> {
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
        let runs_on = || refuse("the pace notes' search past the checkpoints' count");
        'notes: loop {
            self.r.edx = self.get(COUNT)?;
            loop {
                (self.r.ebx, self.r.esi) = self.face(self.r.ebp)?;
                self.code()?;
                let e = self.r.eax;
                if rc2k_core::race::pace::starts_note(e) {
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
                if rc2k_core::race::pace::ends_note(e) {
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

// ---- for the self-tests ----

/// A small deterministic random source for the tests.
pub struct Lot(pub u32);

impl Lot {
    pub fn draw(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
        self.0 >> 8
    }
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 { 0 } else { self.draw() % n }
    }
}

/// The codes a test stage's slots are drawn from (besides 0).
const TEST_CODES: [u8; 20] = [
    1, 2, 5, 9, 0x10, 0x1F, 0x20, 0x29, 0x2F, 0x34, 0x36, 0x38, 0x3C, 0x50, 0x53, 0x62, 0x62, 0x63,
    0x64, 0x70,
];

/// A stage of the test's in the game's variables (the checkpoint count, `g_courseFaces`,
/// the checkpoints' faces, distances and angles, the slot and the tables), all put back
/// when dropped.
pub struct TestStage {
    /// The faces (64 bytes each), written only through `at`.
    faces: Vec<u8>,
    at: *mut u8,
    saved: Vec<u8>,
    kept: Vec<Region>,
    /// Checkpoints in the tables the test fills (count + 8).
    pub room: u32,
}

impl Default for TestStage {
    fn default() -> Self {
        Self::new()
    }
}

impl TestStage {
    pub const MOST: u32 = 48;

    pub fn new() -> Self {
        let room = Self::MOST + 8;
        let a = |x: usize| game_addr(x) as u32;
        let mut kept = regions();
        kept.extend([
            (a(CHECKPOINT_COUNT_RVA), 4),
            (a(COURSE_FACES_RVA), 4),
            (a(CHECKPOINT_FACES_RVA), 4 * room as usize),
            (a(DISTANCES_RVA), 4 * room as usize),
            (a(ANGLES_RVA), 4 * room as usize),
        ]);
        let saved = crate::service::harness::capture(&kept);
        let mut faces = vec![0u8; 64 * room as usize];
        let at = faces.as_mut_ptr();
        let (lo, hi) = (at as u64, at as u64 + faces.len() as u64);
        TEST_WINDOW.store(lo | hi << 32, std::sync::atomic::Ordering::SeqCst);
        Self {
            faces,
            at,
            saved,
            kept,
            room,
        }
    }

    /// The course as the Rust version sees it (the faces' window the test's).
    pub fn course(&self, count: u32) -> Course {
        let at = self.at as u32;
        Course {
            faces: at,
            window: (at, at + self.faces.len() as u32),
            count,
        }
    }

    /// Fills everything for a case: `count` checkpoints, the first of face kind 2 at
    /// `kind2` (none if past the count), codes and distances by lot `seed`. The tables are
    /// filled with places inside the count (as a stage's left-overs) and a text pattern.
    pub fn fill(&self, count: u32, kind2: u32, seed: u32) {
        let mut lot = Lot(seed);
        let room = self.room;
        let faces_at = self.at as u32;
        let put = |i: usize, v: u8| unsafe { self.at.add(i).write(v) };
        let offsets: [u32; 8] = unsafe { game_read(SLOT_OFFSETS_RVA) };
        // Face numbers: checkpoint k's face is room - 1 - k.
        for k in 0..room {
            let f = (room - 1 - k) as usize * 64;
            for b in 0..64 {
                put(f + b, lot.draw() as u8);
            }
            let kind = if k == kind2 || (k > kind2 && k % 7 == 3) {
                2
            } else {
                (k % 2) as u8
            };
            put(f + 0x26, kind);
            for &o in &offsets {
                let code = if k + 1 >= count || lot.below(10) < 6 {
                    0
                } else {
                    TEST_CODES[lot.below(TEST_CODES.len() as u32) as usize]
                };
                put(f + o as usize, code);
            }
        }
        let places = 8 * count.max(1);
        unsafe {
            use crate::framework::game_write;
            game_write(CHECKPOINT_COUNT_RVA, count);
            game_write(COURSE_FACES_RVA, faces_at);
            let mut d = 0u32;
            for k in 0..room as usize {
                game_write(CHECKPOINT_FACES_RVA + 4 * k, room - 1 - k as u32);
                game_write(DISTANCES_RVA + 4 * k, d);
                d = d.wrapping_add(0x4_0000 + lot.below(0x40_0000));
                game_write(ANGLES_RVA + 4 * k, lot.draw().wrapping_mul(0x101));
            }
            game_write(SLOT_RVA, lot.below(8));
            for at in (STARTS..TEXT).step_by(4) {
                game_write(rva(at as usize), lot.below(places));
            }
            for at in TEXT..IMAGE_END {
                game_write(rva(at as usize), (at as u8) | 0x80);
            }
            game_write(rva(COUNT as usize), 1 + lot.below(4));
            game_write(rva(GROUPS as usize), lot.below(4));
        }
    }
}

impl Drop for TestStage {
    fn drop(&mut self) {
        crate::service::harness::restore(&self.kept, &self.saved);
        TEST_WINDOW.store(0, std::sync::atomic::Ordering::SeqCst);
    }
}

/// One case of a pace notes test: the Rust version probed first (after `fill`); a refusal
/// (where the original would read outside its tables or run on) is counted, not compared;
/// else `check_state` on the original and the Rust version. Returns whether it was
/// compared.
pub fn case(
    t: &mut crate::framework::SelfTest,
    what: &str,
    call_original: crate::screens::harness::CallOriginal,
    before: Regs,
    fill: &dyn Fn(),
    rust: crate::service::harness::Rust,
) -> bool {
    fill();
    if rust(&before).is_err() {
        return false;
    }
    crate::service::harness::check_state(t, what, call_original, before, fill, &regions(), rust);
    true
}
