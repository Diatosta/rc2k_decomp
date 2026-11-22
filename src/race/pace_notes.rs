//! The pace notes: the co-driver's calls, made at a stage's loading from codes kept in the
//! checkpoints' faces ("init pacenotes", `race::init_pace_notes`) and handed out during the stage,
//! checkpoint by checkpoint, as car events (`physics::pace_notes_step`,
//! `physics::queue_pace_note`).
//!
//! Each checkpoint face holds eight slots of code (bytes at `pace_slot_offsets`); a place on
//! the line is checkpoint x 8 + slot (`pace_slot` the slot). A note runs from a code in
//! 1 .. 0x63 other than 'b' (0x62) to the next 'b' or 'd' (0x64). The tables:
//! - `PaceNotes.note_count` the notes' count plus one (entry 0 is the start: the first checkpoint
//!   with face kind 2, +2), `PaceNotes.group_count` the groups' count;
//! - by note: `PaceNotes.note_starts[0]` start, `PaceNotes.note_ends[0]` end,
//!   `PaceNotes.note_end_codes[0]` the end's code, the distance (`checkpoint_distances`) from its
//!   end to the next start;
//! - by group: `PaceNotes.group_checkpoints[0]` the checkpoint it is called at,
//!   `PaceNotes.group_texts[0]` its text's place in `PaceNotes.text` (0x4000 bytes): a word for the
//!   distance before (by tens of `PaceNotes.distance_words`), then the codes of each note walked
//!   (`write_calls`), notes less than 0x1E0000 apart and not ended by 'b' joined by ')'
//!   (0x29), a group ended by a NUL. A corner ('c', 0x63) becomes 1 .. 5 by how sharp (the
//!   checkpoint angles' `Buffers.checkpoints.angles[2]` change over the note's length times 60), +
//!   8 for a note of 0x780000 or longer, + 0x10 for a turn one way.
//!
//! The tables are made by `rc2k_core::race::pace_notes::Notes` (on an image of the tables, written
//! back only when it is done, so a refusal writes nothing); here the game's stage it asks and the
//! image's load and store.

use crate::errors::{BadInput, Refused};
use crate::physics::pace_store::PaceNotes;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use rc2k_core::race::pace_notes::{IMAGE_LEN, Notes, Stage};

/// Where the course is (`rc2k_core::race::pace_notes::Course`).
pub type Course = rc2k_core::race::pace_notes::Course;

/// The game's course: the faces' table (`race::course_places`: the walk's places are offsets into
/// it, face n at 0x40 n, read by number in `face_byte`) and the checkpoints' count. Without the
/// table (the walk runs only after `race::load_level` has taken it) the window is empty and a face
/// read refused.
pub(super) fn game_course(race: &Race) -> Course {
    let shared = &race.shared;
    let data = &race.course.stage.course;
    let len = if crate::race::course_places::held(data) {
        data.faces.len().map_or(0, |l| l as u32)
    } else {
        0
    };
    Course {
        faces: 0,
        window: (0, len),
        count: shared.vars.get().checkpoint_count.get(),
    }
}

/// The game's stage, as the walk asks it: the faces by offset into their table. It keeps the
/// first refusal of a face read until the walk is done.
pub struct GameStage<'a> {
    race: &'a Race,
    refused: std::sync::Mutex<Option<BadInput>>,
}

impl<'a> GameStage<'a> {
    pub fn new(race: &'a Race) -> Self {
        GameStage {
            race,
            refused: std::sync::Mutex::new(None),
        }
    }
}

impl Stage for GameStage<'_> {
    fn checkpoint_face(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::face_run_on(self.race.checkpoints(), k as usize)
    }
    fn distance(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::distance_run_on(self.race.checkpoints(), k as usize)
    }
    fn angle(&self, k: u32) -> u32 {
        crate::race::checkpoints_store::angle_run_on(self.race.checkpoints(), k as usize)
    }
    fn slot_offset(&self, slot: u32) -> u32 {
        crate::physics::figures::PACE_SLOT_OFFSETS[slot as usize]
    }
    fn word(&self, tens: u32) -> u8 {
        self.race.physics.pace().distance_words[tens as usize]
    }
    fn face_byte(&self, at: u32) -> u8 {
        // The core asks only inside the course's window; a refusal here is kept for `Machine::init`
        // (the core's trait answers a byte).
        let read = crate::race::course_places::face_field::<u8>(
            self.race.course.reads(),
            at / 0x40,
            at % 0x40,
        );
        match read {
            Ok(b) => b,
            Err(e) => {
                if let Ok(mut held) = self.refused.lock() {
                    held.get_or_insert(e);
                }
                0
            }
        }
    }
}

/// The core's machine on the game's slot and tables (`rc2k_core::race::pace_notes::Notes`).
pub struct Machine<'a> {
    notes: Notes<'a, GameStage<'a>>,
    stage: &'a GameStage<'a>,
}

impl<'a> std::ops::Deref for Machine<'a> {
    type Target = Notes<'a, GameStage<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.notes
    }
}

impl std::ops::DerefMut for Machine<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.notes
    }
}

const _: () = assert!(IMAGE_LEN == 8 + 6 * 4096 + 0x4000);

/// The tables the machine works on, as one image: from the notes' count to the end of the text, the
/// pace store's fields in their order, no gap between them.
fn image(physics: &Physics) -> Vec<u8> {
    let pace = physics.pace();
    let mut image = Vec::with_capacity(IMAGE_LEN);
    image.extend(pace.note_count.to_le_bytes());
    image.extend(pace.group_count.to_le_bytes());
    for table in tables(&pace) {
        for dword in table {
            image.extend(dword.to_le_bytes());
        }
    }
    image.extend(pace.text);
    image
}

/// The image put back in the fields it was read from.
fn put_image(physics: &Physics, image: &[u8]) {
    let mut pace = physics.pace_mut();
    let (dwords, text) = image.split_at(IMAGE_LEN - pace.text.len());
    let mut words = dwords
        .as_chunks::<4>()
        .0
        .iter()
        .map(|w| u32::from_le_bytes(*w));
    let mut next = || words.next().expect("the image holds every table");
    pace.note_count = next();
    pace.group_count = next();
    let PaceNotes {
        note_starts,
        note_ends,
        note_end_codes,
        note_gaps,
        group_checkpoints,
        group_texts,
        text: pace_text,
        ..
    } = &mut *pace;
    for table in [
        note_starts,
        note_ends,
        note_end_codes,
        note_gaps,
        group_checkpoints,
        group_texts,
    ] {
        for dword in table {
            *dword = next();
        }
    }
    pace_text.copy_from_slice(text);
}

/// The six dword tables in the image's order.
fn tables(pace: &PaceNotes) -> [&[u32; 1024]; 6] {
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
        if let Ok(mut held) = stage.refused.lock() {
            *held = None;
        }
        Ok(Self {
            notes: Notes::new(
                race.shared.physics_tables().pace_slot,
                image(physics),
                course,
                stage,
            ),
            stage,
        })
    }

    /// The slot and the tables written back.
    pub fn commit(&self, race: &Race) -> Result<(), BadInput> {
        let physics = &race.physics;
        race.shared.physics_tables_mut().pace_slot = self.slot;
        put_image(physics, &self.image);
        Ok(())
    }

    /// Makes the tables (see `race::init_pace_notes`).
    pub fn init(&mut self) -> Result<(), Refused> {
        let done = self.notes.init();
        if let Ok(mut held) = self.stage.refused.lock()
            && let Some(e) = held.take()
        {
            return Err(e.into());
        }
        Ok(done?)
    }
}
