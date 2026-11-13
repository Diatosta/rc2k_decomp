//! The checkpoint tables, typed (phase 7): IDA 0x9E3738 .. 0xA12AE4, in the image's zeroed
//! tail (`race::buffers_store`): for each checkpoint of the stage (up to 5146) its face, its
//! face count, its angle, its distance, its map place and its nearest replay cameras, then
//! the two replay camera lists. Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Checkpoints {
    /// +0x0 (0x9E3738): `g_checkpointFaces`: each checkpoint's face number (`race::best_line`, `race::level_init`, `physics::passed_split_point`, `drive::drive_computer_car`).
    pub checkpoint_faces: [PlainDword; 5146],
    /// +0x5068 (0x9E87A0): `g_checkCounts`: how many faces each checkpoint's kind has (`race::best_line`, `physics::track_checkpoints`).
    pub check_counts: [PlainDword; 5144],
    /// +0xA0C8 (0x9ED800): `g_checkpointAngles`: each link's angle, from +4 (`dword_9ED808` is the third): `race::best_line`, `race::pace_notes`, `physics::passed_split_point`.
    pub angles: [PlainDword; 5146],
    /// +0xF130 (0x9F2868): nothing reads or writes it (no Rust use, no IDA xref to 0x9F2868 ..; the gap between the angles and the distances).
    pub unused_f130: [PlainDword; 5148],
    /// +0x14178 (0x9F78D8): `g_checkpointDistances` (`race::best_line`, `race::pace_notes`, `race::frame_draw`, `race::init_overlay`).
    pub distances: [PlainDword; 5146],
    /// +0x191e8 (0x9FC940): `g_mapX`: the map's x of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_x: [PlainDword; 5146],
    /// +0x1e250 (0xA019A8): `g_mapY` (`race::best_line`, `race::map_scroll`).
    pub map_y: [PlainDword; 5146],
    /// +0x232b8 (0xA06A10): `g_nearestCamera1`: each checkpoint's nearest replay camera of the first set (`physics::update_camera`, `race::best_line`).
    pub nearest_camera_1: [PlainDword; 5146],
    /// +0x28320 (0xA0BA78): `g_nearestCamera2`: each checkpoint's nearest camera of the second set (stage file section "replaycam2", entry kind 0x38; used by camera modes 0x2A ..= 0x2D, 9, 10: `rc2k_core::physics::camera_view::fixed_camera` true). The first set is the section "replaycam1" (kind 0x3C; modes 0x2E ..= 0x31, 7, 8).
    pub nearest_camera_2: [PlainDword; 5146],
    /// +0x2d388 (0xA10AE0): `g_lineLength` (`race::best_line`).
    pub line_length: PlainDword,
    /// +0x2d38c (0xA10AE4): `g_replayCameras1`: the first set's faces, 0x400 (`physics::update_camera`, `race::best_line`).
    pub replay_cameras_1: [PlainDword; 1024],
    /// +0x2e38c (0xA11AE4): `g_replayCameras2`: the second set's faces, 0x400 (section "replaycam2"; see `nearest_camera_2`).
    pub replay_cameras_2: [PlainDword; 1024],
}

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
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

/// Cell `n` of the table `table` that is the field at byte `field` of `record`. The
/// original indexes these tables by a number it holds to a count, not to the table's room,
/// and a cell past a field's end is the next field's first (as in the image): the cell is
/// found in `record` by its offset. A cell past the record ends the program.
#[inline]
pub fn cell_in<'a, T: crate::cells::Cells>(
    table: &'a [PlainDword],
    record: &'a T,
    field: usize,
    n: usize,
) -> &'a PlainDword {
    table.get(n).unwrap_or_else(|| run_on(record, field, n))
}

/// The dword `n` places on from byte `field` of `record`, across the fields after it.
pub fn run_on<T: crate::cells::Cells>(record: &T, field: usize, n: usize) -> &PlainDword {
    use crate::cells::CellRun;
    let offset = field.wrapping_add(((n as u32).wrapping_mul(4)) as usize);
    let cell = (offset < core::mem::size_of::<T>())
        .then(|| record.cell_run(offset))
        .and_then(|(start, run)| match run {
            CellRun::PlainDword(cells) if (offset - start).is_multiple_of(4) => {
                cells.get((offset - start) / 4)
            }
            _ => None,
        });
    cell.unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a cell past a table",
            crate::errors::BadInput("a cell past the tables of its record"),
        )
    })
}

/// Cell `n` of the field `$field` of the checkpoint tables, run on through the fields
/// after it (`cell_in`).
macro_rules! checkpoint_cell {
    ($table:expr, $field:ident, $skip:expr, $n:expr) => {{
        use core::mem::offset_of;
        cell_in(
            &$table.$field[$skip..],
            $table,
            offset_of!(Checkpoints, $field) + 4 * $skip,
            $n,
        )
    }};
}

/// Cell `n` of the table `table`, which the callers hold below `MOST_CHECKS` (0x1418): inside
/// every one of the tables. A number past the table ends the program.
fn in_table(table: &[PlainDword], n: usize) -> &PlainDword {
    table.get(n).unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a checkpoint number past its table",
            crate::errors::BadInput("a checkpoint number past its table"),
        )
    })
}

// Each table has two readers. The plain one is for a checkpoint number the caller has held
// below `MOST_CHECKS` (the line's building, the nearest-checkpoint search, the start places):
// it stays in the table. The `_run_on` one is for a car's own checkpoint (`Car.checkpoint`,
// which a replay frame puts in from the file, plus the look-aheads added to it): the original
// reads on through the tables after the field.

/// Checkpoint `n`'s face cell (`g_checkpointFaces[n]`), `n` below `MOST_CHECKS`.
pub fn face(table: &Checkpoints, n: usize) -> &PlainDword {
    in_table(&table.checkpoint_faces, n)
}

/// Checkpoint `n`'s face cell, run on past the table.
pub fn face_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, checkpoint_faces, 0, n)
}

/// Checkpoint `n`'s angle cell, as the readers index it: `Buffers.checkpoints.angles[2][n]`, 8
/// bytes into `g_checkpointAngles`; `n` below `MOST_CHECKS`.
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

/// Checkpoint `n`'s map x (`g_mapX[n]`), run on past the table.
pub fn map_x_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, map_x, 0, n)
}

/// Checkpoint `n`'s map y (`g_mapY[n]`), run on past the table.
pub fn map_y_run_on(table: &Checkpoints, n: usize) -> &PlainDword {
    checkpoint_cell!(table, map_y, 0, n)
}

/// How many faces checkpoint `n`'s kind has (`g_checkCounts[n]`), `n` below `MOST_CHECKS`.
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
