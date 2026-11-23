//! The checkpoint tables: for each checkpoint of the stage (up to 5146) its face, its face count,
//! its angle, its distance, its map place and its nearest replay cameras, then the two replay camera
//! lists. A plain record (`session::DrawArc`): written on the game's thread when the stage's best
//! line is stored and read there and, shared copy-on-write when a drawing scope opens, by the
//! drawing parts (`CheckpointsRef`).
//!
//! The tables lie one after the other in this order (`Checkpoints::record_value`), and a car's own
//! checkpoint number (which a replay frame puts in from the file, plus the look-aheads added to it)
//! is not held to a table's length: the `*_run_on` readers read a number past a table's end on into the
//! tables after it, and past the last one end the program.

use crate::memory::session::{ArcRead, DrawArc};

/// How many checkpoints a table has room for.
const ROOM: usize = 5146;
/// The words the angle table and the distance table are apart: nothing reads or writes them, but a
/// number read on from the angles or the counts reaches them (they read 0).
const GAP: usize = 5148;

#[derive(Clone)]
pub struct Checkpoints {
    /// Each checkpoint's face number (`race::best_line`, `race::level_init`,
    /// `physics::passed_split_point`, `drive::drive_computer_car`).
    pub checkpoint_faces: [u32; ROOM],
    /// How many faces each checkpoint's kind has (`race::best_line`,
    /// `physics::track_checkpoints`).
    pub check_counts: [u32; ROOM - 2],
    /// Each link's angle, from +4 (the third): `race::best_line`, `race::pace_notes`,
    /// `physics::passed_split_point`.
    pub angles: [u32; ROOM],
    /// Each checkpoint's distance along the line (`race::best_line`, `race::pace_notes`,
    /// `race::frame_draw`, `race::init_overlay`).
    pub distances: [u32; ROOM],
    /// The map's x of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_x: [u32; ROOM],
    /// The map's y of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_y: [u32; ROOM],
    /// Each checkpoint's nearest replay camera of the first set
    /// (`physics::update_camera`, `race::best_line`).
    pub nearest_camera_1: [u32; ROOM],
    /// Each checkpoint's nearest camera of the second set (stage file section
    /// "replaycam2", entry kind 0x38; used by camera modes 0x2A ..= 0x2D, 9, 10:
    /// `rc2k_core::physics::camera_view::fixed_camera` true). The first set is the section
    /// "replaycam1" (kind 0x3C; modes 0x2E ..= 0x31, 7, 8).
    pub nearest_camera_2: [u32; ROOM],
    /// The line's length (`race::best_line`).
    pub line_length: u32,
    /// The first set's faces, 0x400 (`physics::update_camera`, `race::best_line`).
    pub replay_cameras_1: [u32; 1024],
    /// The second set's faces, 0x400 (section "replaycam2"; see `nearest_camera_2`).
    pub replay_cameras_2: [u32; 1024],
}

impl Checkpoints {
    pub const fn new() -> Self {
        Self {
            checkpoint_faces: [0; ROOM],
            check_counts: [0; ROOM - 2],
            angles: [0; ROOM],
            distances: [0; ROOM],
            map_x: [0; ROOM],
            map_y: [0; ROOM],
            nearest_camera_1: [0; ROOM],
            nearest_camera_2: [0; ROOM],
            line_length: 0,
            replay_cameras_1: [0; 1024],
            replay_cameras_2: [0; 1024],
        }
    }

    /// The tables' words in the order they lie, the gap between the angles and the distances a
    /// run of zeros.
    fn runs(&self) -> [&[u32]; 12] {
        [
            &self.checkpoint_faces,
            &self.check_counts,
            &self.angles,
            &[0; GAP],
            &self.distances,
            &self.map_x,
            &self.map_y,
            &self.nearest_camera_1,
            &self.nearest_camera_2,
            core::slice::from_ref(&self.line_length),
            &self.replay_cameras_1,
            &self.replay_cameras_2,
        ]
    }

    /// The word `index` places from the start of the record, as the tables lie one after the
    /// other: a word past a table's end is the next table's first; none past the last table.
    fn record_value(&self, index: usize) -> Option<u32> {
        let mut left = index;
        for run in self.runs() {
            if let Some(&value) = run.get(left) {
                return Some(value);
            }
            left -= run.len();
        }
        None
    }

    /// Word `n` of a table whose word `first` of the record is its first entry read, run on
    /// through the tables after it. The byte offset is worked out in 32 bits, wrapping. A word past
    /// the record ends the program.
    fn run_on(&self, first: usize, n: usize) -> u32 {
        let offset = (first * 4).wrapping_add((n as u32).wrapping_mul(4) as usize);
        self.record_value(offset / 4).unwrap_or_else(|| {
            crate::memory::host_hooks::end(
                "a word past a table",
                crate::errors::BadInput("a word past the tables of its record"),
            )
        })
    }

    /// The hash lines of the tables: each table one run of dwords under its name, the gap a run of
    /// zeros (`record_hash`'s way).
    #[cfg(feature = "remote")]
    pub fn hash(&self) -> u64 {
        use crate::drive::sim_hash::dword_run_hash;
        [
            (".checkpoint_faces[0]", &self.checkpoint_faces[..]),
            (".check_counts[0]", &self.check_counts),
            (".angles[0]", &self.angles),
            (".reserved_f130[0]", &[0; GAP]),
            (".distances[0]", &self.distances),
            (".map_x[0]", &self.map_x),
            (".map_y[0]", &self.map_y),
            (".nearest_camera_1[0]", &self.nearest_camera_1),
            (".nearest_camera_2[0]", &self.nearest_camera_2),
            (".line_length", core::slice::from_ref(&self.line_length)),
            (".replay_cameras_1[0]", &self.replay_cameras_1),
            (".replay_cameras_2[0]", &self.replay_cameras_2),
        ]
        .iter()
        .fold(0, |sum, (path, run)| {
            sum.wrapping_add(dword_run_hash(path, run))
        })
    }
}

/// The tables as one reader finds them: the game's thread's own (read through the store when
/// used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum CheckpointsRef<'a> {
    Store(&'a DrawArc<Checkpoints>),
    Copy(&'a Checkpoints),
}

impl CheckpointsRef<'_> {
    /// `f` on the tables as they stand (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&Checkpoints) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(tables) => f(tables),
        }
    }
}

impl<'a> From<&'a ArcRead<Checkpoints>> for CheckpointsRef<'a> {
    fn from(copy: &'a ArcRead<Checkpoints>) -> Self {
        Self::Copy(copy)
    }
}

/// Entry `n` of `table`, which the callers hold below `MOST_CHECKS` (0x1418): inside every one of
/// the tables. A number past the table ends the program.
fn in_table(table: &[u32], n: usize) -> u32 {
    table.get(n).copied().unwrap_or_else(|| {
        crate::memory::host_hooks::end(
            "a checkpoint number past its table",
            crate::errors::BadInput("a checkpoint number past its table"),
        )
    })
}

/// Where each readable table begins in the record, in words (the tables' lengths, in order).
const FACES_AT: usize = 0;
const COUNTS_AT: usize = ROOM;
const ANGLES_AT: usize = COUNTS_AT + ROOM - 2;
const DISTANCES_AT: usize = ANGLES_AT + ROOM + GAP;
const MAP_X_AT: usize = DISTANCES_AT + ROOM;
const MAP_Y_AT: usize = MAP_X_AT + ROOM;

// Each table has two readers. The plain one is for a checkpoint number the caller has held below
// `MOST_CHECKS` (the line's building, the nearest-checkpoint search, the start places): it stays in
// the table. The `*_run_on` one is for a car's own checkpoint (`Car.checkpoint`, which a replay
// frame puts in from the file, plus the look-aheads added to it): it reads on through the tables
// after the table.

/// Checkpoint `n`'s face (`checkpoint_faces[n]`), `n` below `MOST_CHECKS`.
pub fn face(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| in_table(&t.checkpoint_faces, n))
}

/// Checkpoint `n`'s face, run on past the table.
pub fn face_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(FACES_AT, n))
}

/// Checkpoint `n`'s angle, as the readers index it: the angles' third entry is checkpoint 0's;
/// `n` below `MOST_CHECKS`.
pub fn angle(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| in_table(&t.angles[2..], n))
}

/// Checkpoint `n`'s angle, run on past the table.
pub fn angle_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(ANGLES_AT + 2, n))
}

/// Checkpoint `n`'s distance, run on past the table.
pub fn distance_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(DISTANCES_AT, n))
}

/// Checkpoint `n`'s map x (`map_x[n]`), run on past the table.
pub fn map_x_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(MAP_X_AT, n))
}

/// Checkpoint `n`'s map y (`map_y[n]`), run on past the table.
pub fn map_y_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(MAP_Y_AT, n))
}

/// How many faces checkpoint `n`'s kind has (`check_counts[n]`), `n` below `MOST_CHECKS`.
pub fn check_count(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| in_table(&t.check_counts, n))
}

/// How many faces checkpoint `n`'s kind has, run on past the table.
pub fn check_count_run_on(table: CheckpointsRef, n: usize) -> u32 {
    table.read(|t| t.run_on(COUNTS_AT, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tables in one record of plain words, as they once were: found by byte offset.
    #[repr(C)]
    struct Layout {
        checkpoint_faces: [u32; ROOM],
        check_counts: [u32; ROOM - 2],
        angles: [u32; ROOM],
        reserved_f130: [u32; GAP],
        distances: [u32; ROOM],
        map_x: [u32; ROOM],
        map_y: [u32; ROOM],
        nearest_camera_1: [u32; ROOM],
        nearest_camera_2: [u32; ROOM],
        line_length: u32,
        replay_cameras_1: [u32; 1024],
        replay_cameras_2: [u32; 1024],
    }

    impl Layout {
        /// Every word in field order (no padding: all are words).
        fn words(&self) -> Vec<u32> {
            [
                &self.checkpoint_faces[..],
                &self.check_counts,
                &self.angles,
                &self.reserved_f130,
                &self.distances,
                &self.map_x,
                &self.map_y,
                &self.nearest_camera_1,
                &self.nearest_camera_2,
                core::slice::from_ref(&self.line_length),
                &self.replay_cameras_1,
                &self.replay_cameras_2,
            ]
            .concat()
        }
    }

    /// Both records with a different nonzero value in every word but the gap's zeros.
    fn filled() -> (Box<Layout>, Box<Checkpoints>) {
        let mut layout = Box::new(Layout {
            checkpoint_faces: [0; ROOM],
            check_counts: [0; ROOM - 2],
            angles: [0; ROOM],
            reserved_f130: [0; GAP],
            distances: [0; ROOM],
            map_x: [0; ROOM],
            map_y: [0; ROOM],
            nearest_camera_1: [0; ROOM],
            nearest_camera_2: [0; ROOM],
            line_length: 0,
            replay_cameras_1: [0; 1024],
            replay_cameras_2: [0; 1024],
        });
        let mut plain = Box::new(Checkpoints::new());
        let mut next = 0u32;
        macro_rules! fill {
            ($($field:ident),*) => {$(
                for (cell, word) in layout.$field.iter_mut().zip(plain.$field.iter_mut()) {
                    next += 1;
                    *cell = next;
                    *word = next;
                }
            )*};
        }
        fill!(checkpoint_faces, check_counts, angles);
        fill!(distances, map_x, map_y, nearest_camera_1, nearest_camera_2);
        next += 1;
        layout.line_length = next;
        plain.line_length = next;
        fill!(replay_cameras_1, replay_cameras_2);
        (layout, plain)
    }

    /// The earlier way: the word found by byte offset in the record.
    fn oracle(words: &[u32], field: usize, n: usize) -> Option<u32> {
        let offset = field.wrapping_add(((n as u32).wrapping_mul(4)) as usize);
        offset
            .is_multiple_of(4)
            .then(|| words.get(offset / 4).copied())
            .flatten()
    }

    /// The new reader's word, or none past the record (where it ends the program).
    fn reach(plain: &Checkpoints, first: usize, n: usize) -> Option<u32> {
        let offset = (first * 4).wrapping_add((n as u32).wrapping_mul(4) as usize);
        plain.record_value(offset / 4)
    }

    #[test]
    fn the_tables_lie_as_the_cell_record_did() {
        let (layout, plain) = filled();
        assert_eq!(
            plain.runs().iter().map(|run| run.len()).sum::<usize>() * 4,
            core::mem::size_of::<Layout>()
        );
        for (name, at, offset) in [
            (
                "faces",
                FACES_AT,
                core::mem::offset_of!(Layout, checkpoint_faces),
            ),
            (
                "counts",
                COUNTS_AT,
                core::mem::offset_of!(Layout, check_counts),
            ),
            ("angles", ANGLES_AT, core::mem::offset_of!(Layout, angles)),
            (
                "distances",
                DISTANCES_AT,
                core::mem::offset_of!(Layout, distances),
            ),
            ("map_x", MAP_X_AT, core::mem::offset_of!(Layout, map_x)),
            ("map_y", MAP_Y_AT, core::mem::offset_of!(Layout, map_y)),
        ] {
            assert_eq!(at * 4, offset, "{name}");
        }
        let _ = layout;
    }

    #[test]
    fn the_run_on_readers_reach_what_the_byte_offset_found() {
        let (layout, plain) = filled();
        let words = layout.words();
        assert_eq!(words.len() * 4, core::mem::size_of::<Layout>());
        let end = words.len() + 3;
        macro_rules! check {
            ($field:ident, $skip:expr, $at:expr, $reader:ident) => {
                for n in 0..end {
                    let field = core::mem::offset_of!(Layout, $field) + 4 * $skip;
                    let new = reach(&plain, field / 4, n);
                    assert_eq!(new, oracle(&words, field, n), "{} {n}", stringify!($field));
                    assert_eq!(field / 4, $at + $skip);
                    if let Some(word) = new {
                        assert_eq!($reader(CheckpointsRef::Copy(&plain), n), word);
                    }
                }
            };
        }
        check!(checkpoint_faces, 0, FACES_AT, face_run_on);
        check!(angles, 2, ANGLES_AT, angle_run_on);
        check!(distances, 0, DISTANCES_AT, distance_run_on);
        check!(map_x, 0, MAP_X_AT, map_x_run_on);
        check!(map_y, 0, MAP_Y_AT, map_y_run_on);
        check!(check_counts, 0, COUNTS_AT, check_count_run_on);
        // Counts that wrap in 32 bits.
        let wide = usize::try_from(0x1_0000_0000u64).ok();
        for n in [0x4000_0000usize, 0x4000_0001, 0xFFFF_FFFF]
            .into_iter()
            .chain(wide)
        {
            let field = core::mem::offset_of!(Layout, map_x);
            assert_eq!(reach(&plain, field / 4, n), oracle(&words, field, n));
        }
    }

    #[test]
    fn a_word_past_a_table_is_the_next_tables_first() {
        let (_, plain) = filled();
        let at = CheckpointsRef::Copy(&plain);
        assert_eq!(distance_run_on(at, ROOM), plain.map_x[0]);
        assert_eq!(map_x_run_on(at, ROOM + 1), plain.map_y[1]);
        assert_eq!(check_count_run_on(at, ROOM - 2), plain.angles[0]);
        assert_eq!(distance_run_on(at, 3), plain.distances[3]);
        // The gap reads zero.
        assert_eq!(angle_run_on(at, ROOM - 2), 0);
    }
}
