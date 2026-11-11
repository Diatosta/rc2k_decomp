//! The checkpoint tables, typed (phase 7): IDA 0x9E3738 .. 0xA12AE4, in the image's zeroed
//! tail (`race::buffers_store`): for each checkpoint of the stage (up to 5146) its face, its
//! face count, its angle, its distance, its map place and its nearest replay cameras, then
//! the two replay camera lists. Names are readings unless IDA's.

use crate::race::owner::race;
use crate::session::Dword;

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Checkpoints {
    /// +0x0 (0x9E3738): `g_checkpointFaces`: each checkpoint's face number (`race::best_line`, `race::level_init`, `physics::passed_split_point`, `drive::drive_computer_car`).
    pub checkpoint_faces: [Dword; 5146],
    /// +0x5068 (0x9E87A0): `g_checkCounts`: how many faces each checkpoint's kind has (`race::best_line`, `physics::track_checkpoints`).
    pub check_counts: [Dword; 5144],
    /// +0xA0C8 (0x9ED800): `g_checkpointAngles`: each link's angle, from +4 (`dword_9ED808` is the third): `race::best_line`, `race::pace_notes`, `physics::passed_split_point`.
    pub angles: [Dword; 5146],
    /// +0xF130 (0x9F2868): nothing reads or writes it (no Rust use, no IDA xref to 0x9F2868 ..; the gap between the angles and the distances).
    pub unused_f130: [Dword; 5148],
    /// +0x14178 (0x9F78D8): `g_checkpointDistances` (`race::best_line`, `race::pace_notes`, `race::frame_draw`, `race::init_overlay`).
    pub distances: [Dword; 5146],
    /// +0x191e8 (0x9FC940): `g_mapX`: the map's x of each checkpoint (`race::best_line`, `race::map_scroll`).
    pub map_x: [Dword; 5146],
    /// +0x1e250 (0xA019A8): `g_mapY` (`race::best_line`, `race::map_scroll`).
    pub map_y: [Dword; 5146],
    /// +0x232b8 (0xA06A10): `g_nearestCamera1`: each checkpoint's nearest replay camera of the first set (`physics::update_camera`, `race::best_line`).
    pub nearest_camera_1: [Dword; 5146],
    /// +0x28320 (0xA0BA78): `g_nearestCamera2`: each checkpoint's nearest camera of the second set (stage file section "replaycam2", entry kind 0x38; used by camera modes 0x2A ..= 0x2D, 9, 10: `rc2k_core::physics::camera_view::fixed_camera` true). The first set is the section "replaycam1" (kind 0x3C; modes 0x2E ..= 0x31, 7, 8).
    pub nearest_camera_2: [Dword; 5146],
    /// +0x2d388 (0xA10AE0): `g_lineLength` (`race::best_line`).
    pub line_length: Dword,
    /// +0x2d38c (0xA10AE4): `g_replayCameras1`: the first set's faces, 0x400 (`physics::update_camera`, `race::best_line`).
    pub replay_cameras_1: [Dword; 1024],
    /// +0x2e38c (0xA11AE4): `g_replayCameras2`: the second set's faces, 0x400 (section "replaycam2"; see `nearest_camera_2`).
    pub replay_cameras_2: [Dword; 1024],
}

/// The piece, a field of `crate::race::owner::race().shared.buffers`.
pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = Checkpoints;
    fn deref(&self) -> &Checkpoints {
        &race().shared.buffers.checkpoints
    }
}

pub static CHECKPOINTS: InBuffers = InBuffers;

/// Cell `n` of the table `table` that is the field at byte `field` of `record`. The
/// original indexes these tables by a number it holds to a count, not to the table's room,
/// and a cell past a field's end is the next field's first (as in the image): the cell is
/// found in `record` by its offset. A cell past the record ends the program.
#[inline]
pub fn cell_in<'a, T: crate::pointers::Cells>(
    table: &'a [Dword],
    record: &'a T,
    field: usize,
    n: usize,
) -> &'a Dword {
    table.get(n).unwrap_or_else(|| run_on(record, field, n))
}

/// The dword `n` places on from byte `field` of `record`, across the fields after it.
pub fn run_on<T: crate::pointers::Cells>(record: &T, field: usize, n: usize) -> &Dword {
    use crate::pointers::CellRun;
    let offset = field.wrapping_add(((n as u32).wrapping_mul(4)) as usize);
    let cell = (offset < core::mem::size_of::<T>())
        .then(|| record.cell_run(offset))
        .and_then(|(start, run)| match run {
            CellRun::Dword(cells) if (offset - start).is_multiple_of(4) => {
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

/// Cell `n` of the field `$field` of the checkpoint tables (`cell_in`).
macro_rules! checkpoint_cell {
    ($field:ident, $skip:expr, $n:expr) => {{
        use core::mem::offset_of;
        cell_in(
            &CHECKPOINTS.$field[$skip..],
            &*CHECKPOINTS,
            offset_of!(Checkpoints, $field) + 4 * $skip,
            $n,
        )
    }};
}

/// Checkpoint `n`'s face cell (`g_checkpointFaces[n]`).
pub fn face(n: usize) -> &'static Dword {
    checkpoint_cell!(checkpoint_faces, 0, n)
}

/// Checkpoint `n`'s angle cell, as the readers index it: `Buffers.checkpoints.angles[2][n]`, 8 bytes into
/// `g_checkpointAngles`.
pub fn angle(n: usize) -> &'static Dword {
    checkpoint_cell!(angles, 2, n)
}

/// Checkpoint `n`'s distance cell (`g_checkpointDistances[n]`).
pub fn distance(n: usize) -> &'static Dword {
    checkpoint_cell!(distances, 0, n)
}

/// Checkpoint `n`'s map x (`g_mapX[n]`).
pub fn map_x(n: usize) -> &'static Dword {
    checkpoint_cell!(map_x, 0, n)
}

/// Checkpoint `n`'s map y (`g_mapY[n]`).
pub fn map_y(n: usize) -> &'static Dword {
    checkpoint_cell!(map_y, 0, n)
}

/// How many faces checkpoint `n`'s kind has (`g_checkCounts[n]`).
pub fn check_count(n: usize) -> &'static Dword {
    checkpoint_cell!(check_counts, 0, n)
}

/// IDA 0x9E3738, the first byte of the piece.
pub const START: usize = 0x9E3738;
/// IDA 0xA12AE4, the end of the piece.
pub const END: usize = 0xA12AE4;

const _: () = assert!(END - START == core::mem::size_of::<Checkpoints>());

/// Where IDA 0x9E3738 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::moved::place(START) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn a_cell_past_a_table_is_the_next_fields_first() {
        let table = &CHECKPOINTS.distances;
        assert!(std::ptr::eq(
            distance(table.len()),
            &raw const CHECKPOINTS.map_x[0]
        ));
        assert!(std::ptr::eq(
            map_x(table.len() + 1),
            &raw const CHECKPOINTS.map_y[1]
        ));
        assert!(std::ptr::eq(
            check_count(5144),
            &raw const CHECKPOINTS.angles[0]
        ));
        assert!(std::ptr::eq(distance(3), &raw const table[3]));
    }

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(Checkpoints, check_counts), 0x9E87A0 - START);
        assert_eq!(offset_of!(Checkpoints, angles), 0x9ED800 - START);
        assert_eq!(offset_of!(Checkpoints, distances), 0x9F78D8 - START);
        assert_eq!(offset_of!(Checkpoints, map_x), 0x9FC940 - START);
        assert_eq!(offset_of!(Checkpoints, map_y), 0xA019A8 - START);
        assert_eq!(offset_of!(Checkpoints, nearest_camera_1), 0xA06A10 - START);
        assert_eq!(offset_of!(Checkpoints, nearest_camera_2), 0xA0BA78 - START);
        assert_eq!(offset_of!(Checkpoints, line_length), 0xA10AE0 - START);
        assert_eq!(offset_of!(Checkpoints, replay_cameras_1), 0xA10AE4 - START);
        assert_eq!(offset_of!(Checkpoints, replay_cameras_2), 0xA11AE4 - START);
    }
}
