//! The checkpoint tables, typed, in the image's zeroed tail (`race::buffers_store`): for each
//! checkpoint of the stage (up to 5146) its face, its face count, its angle, its distance, its map
//! place and its nearest replay cameras, then the two replay camera lists.

use crate::memory::session::PlainDword;
#[cfg(test)]
use crate::test_support::own::race;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Checkpoints {
    /// +0x0: each checkpoint's face number (`race::best_line`, `race::level_init`,
    /// `physics::passed_split_point`, `drive::drive_computer_car`).
    pub checkpoint_faces: [PlainDword; 5146],
    /// +0x5068: how many faces each checkpoint's kind has (`race::best_line`,
    /// `physics::track_checkpoints`).
    pub check_counts: [PlainDword; 5144],
    /// +0xA0C8: each link's angle, from +4 (the third): `race::best_line`, `race::pace_notes`,
    /// `physics::passed_split_point`.
    pub angles: [PlainDword; 5146],
    /// +0xF130: nothing reads or writes it (the gap between the angles and the distances).
    pub unused_f130: [PlainDword; 5148],
    /// +0x14178: each checkpoint's distance along the line (`race::best_line`, `race::pace_notes`,
    /// `race::frame_draw`, `race::init_overlay`).
    pub distances: [PlainDword; 5146],
    /// +0x191e8: the map's x of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_x: [PlainDword; 5146],
    /// +0x1e250: the map's y of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_y: [PlainDword; 5146],
    /// +0x232b8: each checkpoint's nearest replay camera of the first set
    /// (`physics::update_camera`, `race::best_line`).
    pub nearest_camera_1: [PlainDword; 5146],
    /// +0x28320: each checkpoint's nearest camera of the second set (stage file section
    /// "replaycam2", entry kind 0x38; used by camera modes 0x2A ..= 0x2D, 9, 10:
    /// `rc2k_core::physics::camera_view::fixed_camera` true). The first set is the section
    /// "replaycam1" (kind 0x3C; modes 0x2E ..= 0x31, 7, 8).
    pub nearest_camera_2: [PlainDword; 5146],
    /// +0x2d388: the line's length (`race::best_line`).
    pub line_length: PlainDword,
    /// +0x2d38c: the first set's faces, 0x400 (`physics::update_camera`, `race::best_line`).
    pub replay_cameras_1: [PlainDword; 1024],
    /// +0x2e38c: the second set's faces, 0x400 (section "replaycam2"; see `nearest_camera_2`).
    pub replay_cameras_2: [PlainDword; 1024],
}

/// The piece, a field of `crate::test_support::own::race().shared.buffers`.
#[cfg(test)]
pub struct InBuffers;

#[cfg(test)]
impl core::ops::Deref for InBuffers {
    type Target = Checkpoints;
    fn deref(&self) -> &Checkpoints {
        &race().shared.buffers.reads.get().checkpoints
    }
}

#[cfg(test)]
pub static CHECKPOINTS: InBuffers = InBuffers;

/// The dword `index` places from the start of the record, as the neighbour fields name it: each
/// field's cells in the order they are declared, the one after the last field's end being none.
/// These tables are indexed by a number held to a count, not to the table's room, and a cell past a
/// field's end is the next field's first.
fn record_cell(table: &Checkpoints, index: usize) -> Option<&PlainDword> {
    let fields: [&[PlainDword]; 12] = [
        &table.checkpoint_faces,
        &table.check_counts,
        &table.angles,
        &table.unused_f130,
        &table.distances,
        &table.map_x,
        &table.map_y,
        &table.nearest_camera_1,
        &table.nearest_camera_2,
        core::slice::from_ref(&table.line_length),
        &table.replay_cameras_1,
        &table.replay_cameras_2,
    ];
    let mut left = index;
    for cells in fields {
        if let Some(cell) = cells.get(left) {
            return Some(cell);
        }
        left -= cells.len();
    }
    None
}

/// Cell `n` of the table `field` (its cells from the first one the readers index, `skip`), run on
/// through the fields after it. `first` is the field's first cell's place in the record in dwords.
/// The byte offset is worked out in 32 bits, wrapping. A cell past the record ends the program.
fn run_on_from<'a>(
    table: &'a Checkpoints,
    cells: &'a [PlainDword],
    first: usize,
    n: usize,
) -> &'a PlainDword {
    cells.get(n).unwrap_or_else(|| {
        let offset = (first * 4).wrapping_add((n as u32).wrapping_mul(4) as usize);
        let index = offset / 4;
        record_cell(table, index).unwrap_or_else(|| {
            crate::fatal::raise::end(
                "a cell past a table",
                crate::errors::BadInput("a cell past the tables of its record"),
            )
        })
    })
}

/// Cell `n` of the field `$field` of the checkpoint tables, from its cell `$skip`, run on through
/// the fields after it (`run_on_from`).
macro_rules! checkpoint_cell {
    ($table:expr, $field:ident, $skip:expr, $n:expr) => {{
        use core::mem::offset_of;
        run_on_from(
            $table,
            &$table.$field[$skip..],
            offset_of!(Checkpoints, $field) / 4 + $skip,
            $n,
        )
    }};
}

/// Cell `n` of the table `table`, which the callers hold below `MOST_CHECKS` (0x1418): inside every
/// one of the tables. A number past the table ends the program.
fn in_table(table: &[PlainDword], n: usize) -> &PlainDword {
    table.get(n).unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a checkpoint number past its table",
            crate::errors::BadInput("a checkpoint number past its table"),
        )
    })
}

// Each table has two readers. The plain one is for a checkpoint number the caller has held below
// `MOST_CHECKS` (the line's building, the nearest-checkpoint search, the start places): it stays in
// the table. The `*_run_on` one is for a car's own checkpoint (`Car.checkpoint`, which a replay
// frame puts in from the file, plus the look-aheads added to it): it reads on through the tables
// after the field, kept as is.

/// Checkpoint `n`'s face cell (`checkpoint_faces[n]`), `n` below `MOST_CHECKS`.
pub fn face(table: &Checkpoints, n: usize) -> &PlainDword {
    in_table(&table.checkpoint_faces, n)
}

/// Checkpoint `n`'s face cell, run on past the table.
pub fn face_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, checkpoint_faces, 0, n)
}

/// Checkpoint `n`'s angle cell, as the readers index it: `Buffers.checkpoints.angles[2][n]`, 8
/// bytes into `checkpoint_angles`; `n` below `MOST_CHECKS`.
pub fn angle(table: &Checkpoints, n: usize) -> &PlainDword {
    in_table(&table.angles[2..], n)
}

/// Checkpoint `n`'s angle cell, run on past the table.
pub fn angle_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, angles, 2, n)
}

/// Checkpoint `n`'s distance cell, run on past the table.
pub fn distance_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, distances, 0, n)
}

/// Checkpoint `n`'s map x (`map_x[n]`), run on past the table.
pub fn map_x_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, map_x, 0, n)
}

/// Checkpoint `n`'s map y (`map_y[n]`), run on past the table.
pub fn map_y_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, map_y, 0, n)
}

/// How many faces checkpoint `n`'s kind has (`check_counts[n]`), `n` below `MOST_CHECKS`.
pub fn check_count(table: &Checkpoints, n: usize) -> &PlainDword {
    in_table(&table.check_counts, n)
}

/// How many faces checkpoint `n`'s kind has, run on past the table.
pub fn check_count_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, check_counts, 0, n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cells::CellRun;

    /// The earlier way: the cell found by byte offset in the record's cell runs.
    fn oracle(table: &Checkpoints, field: usize, n: usize) -> Option<*const PlainDword> {
        use crate::memory::cells::ByteView;
        let offset = field.wrapping_add(((n as u32).wrapping_mul(4)) as usize);
        (offset < core::mem::size_of::<Checkpoints>())
            .then(|| table.cell_run(offset))
            .and_then(|(start, run)| match run {
                CellRun::PlainDword(cells) if (offset - start).is_multiple_of(4) => {
                    cells.get((offset - start) / 4).map(std::ptr::from_ref)
                }
                _ => None,
            })
    }

    /// The new reader's cell, or none past the record (where it ends the program).
    fn reach(
        table: &Checkpoints,
        cells: &[PlainDword],
        first: usize,
        n: usize,
    ) -> Option<*const PlainDword> {
        match cells.get(n) {
            Some(c) => Some(c),
            None => {
                let offset = (first * 4).wrapping_add((n as u32).wrapping_mul(4) as usize);
                record_cell(table, offset / 4)
            }
        }
        .map(std::ptr::from_ref)
    }

    #[test]
    fn the_run_on_readers_reach_what_the_byte_offset_found() {
        let t: &Checkpoints = &CHECKPOINTS;
        let end = core::mem::size_of::<Checkpoints>() / 4 + 3;
        macro_rules! check {
            ($field:ident, $skip:expr, $reader:ident) => {
                for n in 0..end {
                    let field = core::mem::offset_of!(Checkpoints, $field) + 4 * $skip;
                    let cells = &t.$field[$skip..];
                    let new = reach(t, cells, field / 4, n);
                    assert_eq!(new, oracle(t, field, n), "{} {n}", stringify!($field));
                    if let Some(p) = new {
                        assert!(core::ptr::eq($reader(t, n), p));
                    }
                }
            };
        }
        check!(checkpoint_faces, 0, face_run_on);
        check!(angles, 2, angle_run_on);
        check!(distances, 0, distance_run_on);
        check!(map_x, 0, map_x_run_on);
        check!(map_y, 0, map_y_run_on);
        check!(check_counts, 0, check_count_run_on);
        // Counts that wrap in 32 bits.
        let wide = usize::try_from(0x1_0000_0000u64).ok();
        for n in [0x4000_0000usize, 0x4000_0001, 0xFFFF_FFFF]
            .into_iter()
            .chain(wide)
        {
            let field = core::mem::offset_of!(Checkpoints, map_x);
            assert_eq!(reach(t, &t.map_x, field / 4, n), oracle(t, field, n));
        }
    }

    #[test]
    fn a_cell_past_a_table_is_the_next_fields_first() {
        let table = &CHECKPOINTS.distances;
        assert!(std::ptr::eq(
            distance_run_on(&CHECKPOINTS, table.len()),
            &raw const CHECKPOINTS.map_x[0]
        ));
        assert!(std::ptr::eq(
            map_x_run_on(&CHECKPOINTS, table.len() + 1),
            &raw const CHECKPOINTS.map_y[1]
        ));
        assert!(std::ptr::eq(
            check_count_run_on(&CHECKPOINTS, 5144),
            &raw const CHECKPOINTS.angles[0]
        ));
        assert!(std::ptr::eq(
            distance_run_on(&CHECKPOINTS, 3),
            &raw const table[3]
        ));
    }
}
