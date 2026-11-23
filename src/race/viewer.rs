//! The viewer plays a replay from `var\tmprpl` (the last race's, or a saved one copied there
//! first), block by block (`replay_block`, 0 .. `index.block_count`; 100 frames a block, the
//! frame in `replay_frame`), and edits it: two marks (`mark_in`, `mark_out`), cut, copy and paste
//! of blocks (the clipboard `clip_places`, `clip_cameras`), one undo (`undo_blocks` and the tables
//! after it), the cameras of each block (`block_cameras`: 8 bytes a block, the first 8 of the
//! block's data), and saving the result as a replay slot. This module holds the table helpers the
//! viewer's actions share.

use crate::errors::BadInput;
use crate::math::cells::Cells;
use crate::race::owner::Race;

/// The slot number of `var\tmprpl` as it is.
pub const LAST_RACE: u32 = 0xDE;
/// The length of a camera table: `MOST_BLOCKS` blocks of 8 bytes.
pub const TABLE_LEN: usize = 8 * rc2k_core::race::viewer::MOST_BLOCKS as usize;
/// A byte of the blocks' cameras (8 bytes a block), met by its offset from the first block's
/// cameras. The offset is not checked against the table: a camera step from camera 0 reads the
/// byte before it, the last byte of the replay block read (`block`, which ends where the cameras
/// begin). Any other offset past the table is refused.
#[derive(Clone, Copy)]
pub struct CameraByte(u32);

/// The byte `offset` on from the first block's cameras.
pub fn camera_byte(offset: u32) -> CameraByte {
    CameraByte(offset)
}

impl CameraByte {
    /// The byte `n` blocks on (8 bytes each).
    pub(super) fn blocks_on(self, n: u32) -> CameraByte {
        CameraByte(self.0.wrapping_add(n.wrapping_mul(8)))
    }

    pub fn read(self, race: &Race, what: &'static str) -> Result<u8, BadInput> {
        let buffers = race.viewer_buffers();
        match self.0 {
            u32::MAX => Ok(buffers.block[BLOCK_LAST].get().to_le_bytes()[3]),
            n if (n as usize) < TABLE_LEN => Ok(buffers.block_cameras.get(n as usize)),
            _ => Err(BadInput(what)),
        }
    }

    pub fn write(self, race: &Race, value: u8, what: &'static str) -> Result<(), BadInput> {
        let buffers = race.viewer_buffers();
        match self.0 {
            u32::MAX => {
                let mut bytes = buffers.block[BLOCK_LAST].get().to_le_bytes();
                bytes[3] = value;
                buffers.block[BLOCK_LAST].set(u32::from_le_bytes(bytes));
                Ok(())
            }
            n if (n as usize) < TABLE_LEN => {
                buffers.block_cameras.set(n as usize, value);
                Ok(())
            }
            _ => Err(BadInput(what)),
        }
    }
}

/// The last dword of the replay block read.
const BLOCK_LAST: usize = 0xA03;

/// The byte `offset` on from the first block's cameras (8 bytes a block); the offset is not
/// checked against the table, see `CameraByte`.
pub fn camera_byte_at(race: &Race, offset: usize) -> Result<u8, BadInput> {
    camera_byte(offset as u32).read(race, "a replay camera byte")
}

/// Where the camera of the car the layout shown follows is kept: a byte of the block (the first
/// two cars', and in a replay with the extra tape the third's and the fourth's) or, for the fifth
/// to the eighth car, the choice made in the viewer (`ViewerBuffers::live_cameras`).
#[derive(Clone, Copy)]
pub enum ShownCamera {
    Block(CameraByte),
    Live(usize),
}

/// The car a layout shows alone: the player's car in a replay of the own format, else (the
/// extra tape's replays) the car of the same number as the player.
fn layout_car(race: &Race, layout: u8) -> Result<usize, BadInput> {
    if race.viewer_buffers().v2.is_set() {
        let v2 = race.replay.v2().lock();
        return v2
            .ext
            .as_ref()
            .and_then(|ext| ext.layout_player(u32::from(layout)))
            .ok_or(BadInput("a layout of a player the replay does not have"));
    }
    Ok(usize::from(layout).saturating_sub(2))
}

/// Where the camera that the camera button `camera` (0 or 1) edits in block `block` is kept: in
/// a replay without the extra tape the first two cars' own; with it, for a layout that shows one
/// of the third to the eighth player alone, that player's car's (`layouts::camera_source`).
pub fn shown_camera(race: &Race, block: u32, camera: u32) -> Result<ShownCamera, BadInput> {
    use rc2k_core::race::layouts::{CameraSource, camera_source};
    let base = block.wrapping_mul(8);
    if race.viewer_buffers().extra_tape.is_set() {
        let layout = camera_byte(5u32.wrapping_add(base)).read(race, "a replay camera byte")?;
        if (4..=9).contains(&layout) {
            let car = layout_car(race, layout)?;
            return match camera_source(car).ok_or(BadInput("a camera of a car past the eighth"))? {
                CameraSource::Head(byte) => Ok(ShownCamera::Block(camera_byte(
                    (byte as u32).wrapping_add(base),
                ))),
                CameraSource::Live(n) => Ok(ShownCamera::Live(n)),
            };
        }
    }
    Ok(ShownCamera::Block(camera_byte(camera.wrapping_add(base))))
}

impl ShownCamera {
    /// The camera byte now (a camera not yet chosen in the viewer is the first, 0).
    pub fn read(self, race: &Race) -> Result<u8, BadInput> {
        match self {
            ShownCamera::Block(at) => at.read(race, "a replay camera byte"),
            ShownCamera::Live(n) => {
                let chosen = race.viewer_buffers().live_cameras[n].get();
                Ok(if chosen == crate::race::viewer_buffers_store::NO_CAMERA {
                    0
                } else {
                    chosen as u8
                })
            }
        }
    }
}

/// Whether `n` cells from `at` lie in `table`.
fn inside(table: &dyn Cells, at: usize, n: usize) -> Result<(), BadInput> {
    match at.checked_add(n) {
        Some(end) if end <= table.count() => Ok(()),
        _ => Err(BadInput("past a replay table")),
    }
}

/// Copies `dwords` cells from `from`'s cell `from_at` on to `to`'s cell `to_at` on, forwards;
/// past either table is refused before anything is written.
pub fn move_cells(
    to: &dyn Cells,
    to_at: usize,
    from: &dyn Cells,
    from_at: usize,
    dwords: usize,
) -> Result<(), BadInput> {
    inside(from, from_at, dwords)?;
    inside(to, to_at, dwords)?;
    for n in 0..dwords {
        to.set_bits(to_at + n, from.bits(from_at + n));
    }
    Ok(())
}

/// The same move within one table from the last dword down (for an overlapping move up).
pub fn move_cells_back(
    table: &dyn Cells,
    to_at: usize,
    from_at: usize,
    dwords: usize,
) -> Result<(), BadInput> {
    inside(table, from_at, dwords)?;
    inside(table, to_at, dwords)?;
    for n in (0..dwords).rev() {
        table.set_bits(to_at + n, table.bits(from_at + n));
    }
    Ok(())
}

/// Sets `dwords` cells of `table` from `at` on to 0.
pub fn clear_cells(table: &dyn Cells, at: usize, dwords: usize) -> Result<(), BadInput> {
    inside(table, at, dwords)?;
    for n in 0..dwords {
        table.set_bits(at + n, 0);
    }
    Ok(())
}

// ---- for `net::sort_players` ----
