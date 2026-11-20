//! `race::open_viewed_replay`: opens a replay for the viewer. Its file (`race::replay_path`; the
//! last race's only when one was recorded, `replay_closed`; a saved one, below 100 after the
//! path's, is copied over `var\tmprpl` first, `files::copy_file`) is opened (mode 0) into `h`,
//! with `replay_open` 1, block and frame 0. Its header (0x430 bytes into `Replays.header`:
//! "REPL", version) and its index (the last 0x3850 bytes into "RCIB": its name, the block count
//! held to 0x708, more than 0) are read; each block is read (`drive::read_replay_frame_0`) for its
//! first 8 bytes and the block's cameras (`block_cameras`); the marks are set round the whole; the
//! first block is read again, the cars' header (0x400 bytes from `Replays.header`) goes into
//! `header_stage`, and the replay's physics patches are put in force
//! (`race::apply_replay_patches`). When anything fails the replay is closed
//! (`race::close_viewed_replay`) and carry is set.
//!
//! Takes the replay (`race::replay_path`'s number).

use crate::errors::Refused;
use crate::files::handle;
use crate::race::owner::Race;
use crate::race::replay_path::replay_path;

use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// How opening a replay went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opened {
    /// The replay is open.
    Yes,
    /// There is no such replay (an empty slot, or no race recorded yet): nothing to say.
    Missing,
    /// The file exists but cannot be used.
    Damaged,
}

impl Opened {
    /// Whether the replay did not open.
    pub fn failed(self) -> bool {
        self != Opened::Yes
    }
}

/// How `slot`'s replay opened.
pub(super) fn open_viewed_replay(
    race: &Race,
    system: &crate::system::owner::System,
    slot: u32,
) -> Result<Opened, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Logic in `rc2k_core::race::viewer_flow::{last_race_missing, copied_first, read_good,
    // version_good, index_blocks, opened_marks}`.
    use rc2k_core::race::viewer_flow::{
        Blocks, CAMERA_DWORDS, CAR_HEADER_LEN, copied_first, index_blocks, last_race_missing,
        opened_marks, read_good, version_good,
    };
    let failed = || -> Result<Opened, Refused> {
        crate::race::close_viewed_replay::close_viewed_replay(race)?;
        Ok(Opened::Damaged)
    };
    let missing = || -> Result<Opened, Refused> {
        crate::race::close_viewed_replay::close_viewed_replay(race)?;
        Ok(Opened::Missing)
    };
    let found = replay_path(race, slot)?;
    let mut path = crate::race::replay_path::file_to_load(slot, found.path);
    if last_race_missing(
        found.last_race,
        || shared.replay_file().replay_closed.get(), /* a replay was recorded */
    ) {
        return missing();
    }
    if copied_first(found.number) {
        // No file in the slot: nothing was saved there.
        match handle::open_by_mode(&path, 0)? {
            Some(file) => {
                handle::close(file);
            }
            None => return missing(),
        }
        // A saved replay is copied over the last race's first.
        let copied = crate::files::copy_file::copy_file(
            &path,
            &crate::race::replay_path::last_race(replay),
        )?;
        if !copied {
            return failed();
        }
        path = crate::race::replay_path::last_race(replay);
    }
    let Some(file) = handle::open_by_mode(&path, 0)? else {
        return if copied_first(found.number) {
            failed()
        } else {
            missing()
        };
    };
    shared.replay_file().replay_handle.set(file);
    shared.replay_file().replay_open.set(true);
    replay.replay_view().replay_block.set(0);
    replay.replay_view().replay_frame.set(0);
    // Whether a buffer's read is good (`read_good`), given its first word after.
    let good = |(n, ok): (u32, bool), len: u32, first: &dyn Fn() -> u32, header: bool| {
        read_good(!ok, n, len, first(), header)
    };
    let mut header = shared.replays().header.read();
    let got = handle::read_slice(shared.replay_file().replay_handle.get(), &mut header);
    shared.replays().header.write(&header);
    if !good(
        got,
        HEADER_LEN,
        &|| race.shared.replays().header.get_u32(0),
        true,
    ) || !version_good(shared.replays().header.get_u32(8))
    {
        return failed();
    }
    // The blocks of a replay with the extra tape hold it after their frame.
    race.viewer_buffers().extra_tape.set(
        rc2k_core::race::replay::read_version(shared.replays().header.get_u32(8)) == Some(true),
    );
    // A replay of the own format has its cars after the header and a tail to each block.
    let own = shared.replays().header.get_u32(8) == rc2k_core::race::replay_v2::VERSION;
    race.viewer_buffers().v2.set(own);
    {
        let mut v2 = replay.v2().lock();
        v2.ext = None;
        v2.tail = rc2k_core::race::replay_v2::Tail::new();
    }
    if own {
        let len = shared
            .replays()
            .header
            .get_u32(rc2k_core::race::replay_v2::EXT_LEN_AT) as usize;
        let mut bytes = vec![0u8; len.min(0x1_0000)];
        let (n, ok) = handle::read_slice(shared.replay_file().replay_handle.get(), &mut bytes);
        let extension = (ok && len <= 0x1_0000 && n as usize == len)
            .then(|| rc2k_core::race::replay_v2::Extension::parse(&bytes))
            .flatten();
        let Some(extension) = extension else {
            return failed();
        };
        replay.v2().lock().ext = Some(extension);
    }
    // The index is the file's last bytes: 0x3850 bytes in the old format, larger in the own one.
    let (index_len, most) = if own {
        (
            rc2k_core::race::replay_v2::OWN_INDEX_LEN as u32,
            rc2k_core::race::replay_v2::MOST_BLOCKS,
        )
    } else {
        (INDEX_LEN, rc2k_core::race::replay::MOST_BLOCKS)
    };
    let at = {
        handle::seek(
            shared.replay_file().replay_handle.get(),
            index_len.wrapping_neg() as i32,
            2,
        )
    };
    let places = &shared.replay_file().index.dwords()[..index_len as usize / 4];
    for cell in shared.replay_file().index.dwords() {
        cell.set(0);
    }
    if at.is_none()
        || !good(
            handle::read_cells(shared.replay_file().replay_handle.get(), places),
            index_len,
            &|| race.shared.replay_file().index.magic.get(),
            false,
        )
    {
        return failed();
    }
    match index_blocks(shared.replay_file().index.block_count.get(), most) {
        Blocks::Bad => return failed(),
        Blocks::Held(n) => shared.replay_file().index.block_count.set(n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, block) }? {
            return failed();
        }
        crate::race::viewer::move_cells(
            &race.viewer_buffers().block_cameras,
            2 * block as usize,
            &race.viewer_buffers().block,
            0,
            CAMERA_DWORDS,
        )?;
        block += 1;
        if block == shared.replay_file().index.block_count.get() {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(shared.replay_file().index.block_count.get());
    race.viewer_buffers().mark_in.set(mark_in);
    race.viewer_buffers().mark_out.set(mark_out);
    race.viewer_buffers().block_used.set(0);
    if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, 0) }? {
        return failed();
    }
    // The file's header is bytes (a byte-level copy of its stage part), kept in `Replays.header`.
    let stage =
        shared
            .replays()
            .header
            .read_at(0x30, CAR_HEADER_LEN, "the replay header's stage")?;
    put_header_stage(race, &stage);
    crate::race::replay_patches::apply_replay_patches(race)?;
    Ok(Opened::Yes)
}

/// The cars' header (`CAR_HEADER_LEN` bytes) into the replay work's fields from `header_stage` on:
/// the header's settings, the sealed part, the patch switches and the unread rest, a dword each
/// with no gap, up to the camera wishes.
fn put_header_stage(race: &Race, header: &[u8]) {
    let w = race.replay_work();
    let singles = [
        &w.header_stage,
        &w.header_mode,
        &w.header_players,
        &w.header_cars,
        &w.header_kind_0,
        &w.header_kind_1,
        &w.header_kind_2,
        &w.header_kind_3,
        &w.header_one_player,
        &w.header_codriver_sex_0,
        &w.header_codriver_sex_1,
        &w.header_start_place,
        &w.header_arcade_cars,
        &w.header_seed,
        &w.header_game_over_0,
        &w.header_game_over_1,
    ];
    let cells = singles
        .into_iter()
        .chain(&w.sealed_header)
        .chain(std::iter::once(&w.patches_in_header))
        .chain(&w.header_patches)
        .chain(&w.unknown_7d0);
    for (dword, bytes) in cells.zip(header.as_chunks::<4>().0) {
        dword.set(u32::from_le_bytes(*bytes));
    }
}
