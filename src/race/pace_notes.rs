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
//! The functions run on `rc2k_core::race::pace_notes::Notes` (the registers and an image of
//! the tables, written back only when the function is done, so a refusal writes nothing);
//! here the game's stage it asks and the image's load and store.

use crate::errors::Refused;
use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI, Regs, game_addr, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::Region;
pub use rc2k_core::race::pace_notes::{
    COUNT, ENDS, GAPS, GROUP_CHECKPOINTS, GROUP_TEXTS, GROUPS, IMAGE_END, IMAGE_START, KINDS,
    MOST_CHECKPOINTS, STARTS, TEXT, TEXT_LEN,
};
use rc2k_core::race::pace_notes::{IMAGE_LEN, Notes, Registers, Stage};

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

/// Where the course is (`rc2k_core::race::pace_notes::Course`).
pub type Course = rc2k_core::race::pace_notes::Course;

/// A test stage's faces while one is up (start, end): the originals' calls of the hooks
/// read them there.
static TEST_WINDOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The game's course: the faces (`g_courseFaces`), where they may be read (the course
/// memory, or a test stage's faces), the checkpoints' count.
pub fn game_course() -> Course {
    unsafe {
        let memory: u32 = game_read(COURSE_MEMORY_RVA);
        let faces: u32 = game_read(COURSE_FACES_RVA);
        let test = TEST_WINDOW.load(std::sync::atomic::Ordering::SeqCst);
        let (lo, hi) = (test as u32, (test >> 32) as u32);
        Course {
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

/// The game's stage, as the walk asks it.
pub struct GameStage;

impl Stage for GameStage {
    fn checkpoint_face(&self, k: u32) -> u32 {
        unsafe { game_read(CHECKPOINT_FACES_RVA + 4 * k as usize) }
    }
    fn distance(&self, k: u32) -> u32 {
        unsafe { game_read(DISTANCES_RVA + 4 * k as usize) }
    }
    fn angle(&self, k: u32) -> u32 {
        unsafe { game_read(ANGLES_RVA + 4 * k as usize) }
    }
    fn slot_offset(&self, slot: u32) -> u32 {
        unsafe { game_read(SLOT_OFFSETS_RVA + 4 * slot as usize) }
    }
    fn word(&self, tens: u32) -> u8 {
        unsafe { game_read(WORDS_RVA + tens as usize) }
    }
    fn face_byte(&self, at: u32) -> u8 {
        unsafe { (at as *const u8).read() }
    }
}

/// The core's machine on the game's slot and tables. Converted:
/// `rc2k_core::race::pace_notes::Notes`.
pub struct Machine(Notes<'static, GameStage>);

impl std::ops::Deref for Machine {
    type Target = Notes<'static, GameStage>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Machine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Machine {
    pub fn load(r: &Regs, course: Course) -> Self {
        let at = game_addr(rva(IMAGE_START as usize)) as *const u8;
        let regs = Registers {
            eax: r.eax,
            ebx: r.ebx,
            ecx: r.ecx,
            edx: r.edx,
            esi: r.esi,
            edi: r.edi,
            ebp: r.ebp,
        };
        Self(Notes::new(
            regs,
            unsafe { game_read(SLOT_RVA) },
            unsafe { std::slice::from_raw_parts(at, IMAGE_LEN) }.to_vec(),
            course,
            &GameStage,
        ))
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

    /// `sub_441A37` (`race::pace_note_code`).
    pub fn code(&mut self) -> Result<(), Refused> {
        Ok(self.0.code()?)
    }

    /// `sub_441C78` (`race::pace_distance_word`).
    pub fn distance_word(&mut self) -> Result<(), Refused> {
        Ok(self.0.distance_word()?)
    }

    /// `sub_441CB4` (`race::pace_note_calls`).
    pub fn note_calls(&mut self) -> Result<(), Refused> {
        Ok(self.0.note_calls()?)
    }

    /// `sub_441A66` (`race::init_pace_notes`).
    pub fn init(&mut self) -> Result<(), Refused> {
        Ok(self.0.init()?)
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
