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
//! The tables are made by `rc2k_core::race::pace_notes::Notes` (on an image of the tables,
//! written back only when it is done, so a refusal writes nothing);
//! here the game's stage it asks and the image's load and store.

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::Shared;
pub use rc2k_core::race::pace_notes::IMAGE_START;
use rc2k_core::race::pace_notes::{IMAGE_LEN, Notes, Stage};

/// Where the course is (`rc2k_core::race::pace_notes::Course`).
pub type Course = rc2k_core::race::pace_notes::Course;

/// The game's course: the faces' table (`race::course_places`: the walk's places are
/// offsets into it, face n at 0x40 n, read by number in `face_byte`) and the checkpoints'
/// count. Without the table (the walk runs only after `race::load_level` has taken it) the
/// window is empty and a face read refused.
pub fn game_course(shared: &Shared) -> Course {
    let len = if crate::race::course_places::held() {
        crate::race::course_places::FACES
            .with(|f| f.len() as u32)
            .unwrap_or(0)
    } else {
        0
    };
    Course {
        faces: 0,
        window: (0, len),
        count: shared.vars.checkpoint_count.bits(),
    }
}

/// The first refusal of a face read, kept until the walk is done.
static REFUSED: std::sync::Mutex<Option<BadInput>> = std::sync::Mutex::new(None);

/// The game's stage, as the walk asks it: the faces by offset into their table.
pub struct GameStage;

static STAGE: GameStage = GameStage;

impl Stage for GameStage {
    fn checkpoint_face(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::face(k as usize).get()
    }
    fn distance(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::distance(k as usize).get()
    }
    fn angle(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::angle(k as usize).get()
    }
    fn slot_offset(&self, slot: u32) -> u32 {
        race().shared.physics_tables.pace_slot_offsets[slot as usize].bits() // dword_614D14: each slot's byte in the face
    }
    fn word(&self, tens: u32) -> u8 {
        race().physics.pace.distance_words.get(tens as usize)
    }
    fn face_byte(&self, at: u32) -> u8 {
        // The core asks only inside the course's window; a refusal here is kept for
        // `Machine::init` (the core's trait answers a byte).
        let read = crate::race::course_places::face_field::<u8>(
            at / 0x40,
            at % 0x40,
            "a checkpoint face's byte",
        );
        match read {
            Ok(b) => b,
            Err(e) => {
                if let Ok(mut held) = REFUSED.lock() {
                    held.get_or_insert(e);
                }
                0
            }
        }
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

/// The tables the machine works on: the pace store from `IMAGE_START` to its end.
fn tables(physics: &Physics) -> GamePtr {
    use crate::physics::pace_store::{END, START};
    const _: () = assert!(IMAGE_START as usize >= START && IMAGE_START as usize + IMAGE_LEN == END);
    // The tables are the image's bytes, read by the notes' own offsets.
    GamePtr::to(&physics.pace).offset(IMAGE_START - START as u32)
}

impl Machine {
    pub fn load(race: &Race, course: Course) -> Result<Self, BadInput> {
        let physics = &race.physics;
        if let Ok(mut held) = REFUSED.lock() {
            *held = None;
        }
        Ok(Self(Notes::new(
            race.shared.physics_tables.pace_slot.bits(),
            tables(physics).read_bytes(IMAGE_LEN, "the pace notes' tables")?,
            course,
            &STAGE,
        )))
    }

    /// The slot and the tables written back.
    pub fn commit(&self, race: &Race) -> Result<(), BadInput> {
        let physics = &race.physics;
        race.shared.physics_tables.pace_slot.set_bits(self.slot);
        tables(physics).write_bytes(&self.image, "the pace notes' tables")
    }

    /// 0x441A66 (`race::init_pace_notes`).
    pub fn init(&mut self) -> Result<(), Refused> {
        let done = self.0.init();
        if let Ok(mut held) = REFUSED.lock()
            && let Some(e) = held.take()
        {
            return Err(e.into());
        }
        Ok(done?)
    }
}
