//! The pace notes: the co-driver's calls, made at a stage's loading from
//! codes kept in the checkpoints' faces ("init pacenotes", `race::init_pace_notes`
//! 0x441A66) and handed out during the stage, checkpoint by checkpoint, as car events
//! (`physics::pace_notes_step` 0x4417EC, `physics::queue_pace_note` 0x4418B3).
//!
//! Each checkpoint face holds eight slots of code (bytes +0x28, +0x29, +0x2A, +0x2B, +0x19,
//! +0x1A, +0x1B, +0x20: `pace_slot_offsets`); a place on the line is checkpoint x 8 + slot
//! (`pace_slot` the slot). A note runs from a code in 1 .. 0x63 other than 'b' (0x62) to
//! the next 'b' or 'd' (0x64). The tables (0x615E10 ..):
//! - `PaceNotes.note_count` the notes' count plus one (entry 0 is the start: the first checkpoint
//!   with face kind 2, +2), `PaceNotes.group_count` the groups' count;
//! - by note: `PaceNotes.note_starts[0]` start, `PaceNotes.note_ends[0]` end, `PaceNotes.note_end_codes[0]` the end's code,
//!   `dword_618E18` the distance (`g_checkpointDistances`) from its end to the next start;
//! - by group: `PaceNotes.group_checkpoints[0]` the checkpoint it is called at, `PaceNotes.group_texts[0]` its text's
//!   place in `PaceNotes.text` (0x4000 bytes): a word for the distance before (by tens of
//!   `PaceNotes.distance_words`), then the codes of each note walked (`race::pace_note_calls`), notes less
//!   than 0x1E0000 apart and not ended by 'b' joined by ')' (0x29), a group ended by a NUL.
//!   A corner ('c', 0x63) becomes 1 .. 5 by how sharp (the checkpoint angles'
//!   `Buffers.checkpoints.angles[2]` change over the note's length times 60), + 8 for a note of 0x780000 or
//!   longer, + 0x10 turning one way.
//!
//! The tables are made by `rc2k_core::race::pace_notes::Notes` (on an image of the tables,
//! written back only when it is done, so a refusal writes nothing);
//! here the game's stage it asks and the image's load and store.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use rc2k_core::race::pace_notes::{IMAGE_LEN, Notes, Stage};

/// Where the course is (`rc2k_core::race::pace_notes::Course`).
pub type Course = rc2k_core::race::pace_notes::Course;

/// The game's course: the faces' table (`race::course_places`: the walk's places are
/// offsets into it, face n at 0x40 n, read by number in `face_byte`) and the checkpoints'
/// count. Without the table (the walk runs only after `race::load_level` has taken it) the
/// window is empty and a face read refused.
pub fn game_course(race: &Race) -> Course {
    let shared = &race.shared;
    let len = if crate::race::course_places::held() {
        crate::race::course_places::FACES
            .len()
            .map_or(0, |l| l as u32)
    } else {
        0
    };
    Course {
        faces: 0,
        window: (0, len),
        count: shared.vars.checkpoint_count.get(),
    }
}

/// The first refusal of a face read, kept until the walk is done.
static REFUSED: std::sync::Mutex<Option<BadInput>> = std::sync::Mutex::new(None);

/// The game's stage, as the walk asks it: the faces by offset into their table.
pub struct GameStage<'a>(pub &'a Race);

impl Stage for GameStage<'_> {
    fn checkpoint_face(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::face(self.0.checkpoints(), k as usize).get()
    }
    fn distance(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::distance(self.0.checkpoints(), k as usize).get()
    }
    fn angle(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::angle(self.0.checkpoints(), k as usize).get()
    }
    fn slot_offset(&self, slot: u32) -> u32 {
        self.0.shared.physics_tables.pace_slot_offsets[slot as usize].get() // PhysicsTables.pace_slot_offsets[0]: each slot's byte in the face
    }
    fn word(&self, tens: u32) -> u8 {
        self.0.physics.pace.distance_words.get(tens as usize)
    }
    fn face_byte(&self, at: u32) -> u8 {
        // The core asks only inside the course's window; a refusal here is kept for
        // `Machine::init` (the core's trait answers a byte).
        let read =
            crate::race::course_places::face_field::<u8>(&self.0.course, at / 0x40, at % 0x40);
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
pub struct Machine<'a>(Notes<'a, GameStage<'a>>);

impl<'a> std::ops::Deref for Machine<'a> {
    type Target = Notes<'a, GameStage<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Machine<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const _: () = assert!(IMAGE_LEN == 8 + 6 * 4096 + 0x4000);

/// The tables the machine works on, as one image: from the notes' count to the end of the
/// text, the pace store's fields in their order, no gap between them.
fn image(physics: &Physics) -> Vec<u8> {
    let pace = &physics.pace;
    let mut image = Vec::with_capacity(IMAGE_LEN);
    image.extend(pace.note_count.get().to_le_bytes());
    image.extend(pace.group_count.get().to_le_bytes());
    for table in tables(pace) {
        for dword in table {
            image.extend(dword.get().to_le_bytes());
        }
    }
    image.extend(pace.text.read());
    image
}

/// The image put back in the fields it was read from.
fn put_image(physics: &Physics, image: &[u8]) {
    let pace = &physics.pace;
    let (dwords, text) = image.split_at(IMAGE_LEN - pace.text.atomics().len());
    let mut words = dwords
        .as_chunks::<4>()
        .0
        .iter()
        .map(|w| u32::from_le_bytes(*w));
    let mut next = || words.next().expect("the image holds every table");
    pace.note_count.set(next());
    pace.group_count.set(next());
    for table in tables(pace) {
        for dword in table {
            dword.set(next());
        }
    }
    for (at, &byte) in text.iter().enumerate() {
        pace.text.set(at, byte);
    }
}

/// The six dword tables in the image's order.
fn tables(pace: &crate::physics::pace_store::PaceNotes) -> [&[crate::session::Dword; 1024]; 6] {
    [
        &pace.note_starts,
        &pace.note_ends,
        &pace.note_end_codes,
        &pace.note_gaps,
        &pace.group_checkpoints,
        &pace.group_texts,
    ]
}

impl<'a> Machine<'a> {
    pub fn load(race: &Race, course: Course, stage: &'a GameStage<'a>) -> Result<Self, BadInput> {
        let physics = &race.physics;
        if let Ok(mut held) = REFUSED.lock() {
            *held = None;
        }
        Ok(Self(Notes::new(
            race.shared.physics_tables.pace_slot.get(),
            image(physics),
            course,
            stage,
        )))
    }

    /// The slot and the tables written back.
    pub fn commit(&self, race: &Race) -> Result<(), BadInput> {
        let physics = &race.physics;
        race.shared.physics_tables.pace_slot.set(self.slot);
        put_image(physics, &self.image);
        Ok(())
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
