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
use crate::framework::{game_addr, game_read, rva};
use crate::physics::tables_store::PHYSICS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
pub use rc2k_core::race::pace_notes::IMAGE_START;
use rc2k_core::race::pace_notes::{IMAGE_LEN, Notes, Registers, Stage};

const WORDS_RVA: usize = rva(0x614D38); // byte_614D38: the distance's word by tens (41)
pub const CHECKPOINT_FACES_RVA: usize = rva(0x9E3738); // g_checkpointFaces
pub const DISTANCES_RVA: usize = rva(0x9F78D8); // g_checkpointDistances
pub const ANGLES_RVA: usize = rva(0x9ED808); // dword_9ED808: the checkpoints' angles
const COURSE_MEMORY_LEN: u32 = 0x130_0000;

/// Where the course is (`rc2k_core::race::pace_notes::Course`).
pub type Course = rc2k_core::race::pace_notes::Course;

/// A test stage's faces while one is up (start, end): the original's code
/// reads them there.
static TEST_WINDOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The game's course: the faces (`g_courseFaces`), where they may be read (the course
/// memory, or a test stage's faces), the checkpoints' count.
pub fn game_course() -> Course {
    {
        let memory: u32 = COURSE_LISTS.course_memory.bits();
        let faces: u32 = COURSE.course_faces.bits();
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
            count: RACE_VARS.checkpoint_count.bits(),
        }
    }
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
        PHYSICS.pace_slot_offsets[slot as usize].bits() // dword_614D14: each slot's byte in the face
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
    pub fn load(course: Course) -> Self {
        let at = game_addr(rva(IMAGE_START as usize)) as *const u8;
        Self(Notes::new(
            Registers::default(),
            PHYSICS.pace_slot.bits(),
            unsafe { std::slice::from_raw_parts(at, IMAGE_LEN) }.to_vec(),
            course,
            &GameStage,
        ))
    }

    /// The slot and the tables written back.
    pub fn commit(&self) {
        unsafe {
            PHYSICS.pace_slot.set_bits(self.slot);
            let at = game_addr(rva(IMAGE_START as usize)) as *mut u8;
            std::slice::from_raw_parts_mut(at, self.image.len()).copy_from_slice(&self.image);
        }
    }

    /// `sub_441A66` (`race::init_pace_notes`).
    pub fn init(&mut self) -> Result<(), Refused> {
        Ok(self.0.init()?)
    }
}
