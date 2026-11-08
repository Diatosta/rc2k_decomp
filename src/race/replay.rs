//! The replay file (`var\tmprpl`, IDA 2026-10-17): opened with its header at the race's
//! start (`race::open_replay`, the header sealed by `race::seal_replay_header`), frames
//! gathered (`race::add_replay_frame`), written block by block (`race::write_replay_block`
//! through `race::write_replay_chunk`, which packs by runs), and closed with its index
//! ("RCIB", `race::close_replay`). The steps call each other (a failed block closes the
//! file, which writes the last frames as a block); the file primitives are
//! `files::handle`'s.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::net::owner::net;
use crate::pointers::GamePtr;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::owner_shared::Shared;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;
use crate::session::{Bytes, Dword};

/// `dword_4F66F9`: the race record's seal key (the replay header's checksum,
/// `race::seal_replay_header`; `race::seal_stage_time` turns the first car's time by it).
/// Split 2026-10-25 from sound's dwords around it; `moved::MOVED` names IDA 0x4F66F9 here
/// (`race::owner_replay::Replay::seal_key`).
// The index (`crate::race::owner::race().shared.replay_file.index()`) is 0x3850 bytes.
const _: () = assert!(crate::race::replay_file_store::INDEX_DWORDS * 4 == 0x3850);
pub const MOST_BLOCKS: u32 = rc2k_core::race::replay::MOST_BLOCKS;
pub const BLOCK_LEN: u32 = 0x2810;
pub const HEADER_LEN: u32 = 0x430;

/// IDA's address of the replays' buffers.
pub const START: usize = 0x528504;
/// Their length.
pub const LEN: usize = 0x5295B4 - START;
/// A replay's name: 0x20 bytes.
pub const NAME_BYTES: usize = 0x20;
/// The replays that can be saved.
pub const SLOTS: usize = 100;

/// The replays' buffers.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Replays {
    /// `dword_528504`: the file's header (a dword array, `HEADER_LEN` bytes).
    pub header: Bytes<{ HEADER_LEN as usize }>,
    /// `byte_528934`: the saved replays' names, 100 of 0x20 bytes (twenty to a page).
    pub names: Bytes<{ SLOTS * NAME_BYTES }>,
}

impl Replays {
    /// The replays' buffers; the first values are the image's (all zero).
    pub const fn new() -> Self {
        Replays {
            header: Bytes::new([0; HEADER_LEN as usize]),
            names: Bytes::new([0; SLOTS * NAME_BYTES]),
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use crate::race::owner::race;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&race().shared.replays, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let shared = &race().shared;
        assert_eq!(offset_of!(Replays, names), 0x528934 - START);
        assert_eq!(
            crate::moved::place(START),
            shared.replays.header.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x528934),
            shared.replays.names.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x528510),
            (shared.replays.header.addr() + 0xC) as usize
        );
    }
}

/// `race::seal_replay_header` (0x41F0A3): the replay's sealed header part (`dword_719B54`,
/// 0x40 dwords): version byte 6, the player's name (`strings::copy_printable`, 0x20), the
/// settings and options copied (`HEADER_MOVES`), a checksum (from dword 1: add, `rol` 5)
/// into dword 0 and `dword_4F66F9`; then its 32 blocks of 8 chained through
/// `race::seal_block` with the key "halfmoon" (the game's key put back after). Every
/// register kept.
pub fn seal_replay_header(replay: &Replay) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::replay::{seal_checksum, chained, SEAL_KEY}`.
    use rc2k_core::race::replay::{SEAL_KEY, SEAL_VERSION, chained, seal_checksum};
    // The sealed header is the file's bytes: the version byte and the name sit inside its
    // dwords.
    let sealed = GamePtr::to(&REPLAY_WORK.sealed_header);
    sealed
        .offset(0x719B58 - 0x719B54)
        .write::<u8>(SEAL_VERSION, "the replay header's version")?;
    crate::strings::copy_printable::copy_printable(
        crate::pointers::GamePtr::to(&menus().play.names.player_names), // g_playerNames
        sealed.offset(0x719B61 - 0x719B54),
        0x20,
    )?;
    for &(size, to, from) in &HEADER_MOVES {
        // IDA's names of the header's fields and the settings they copy.
        let (to, from) = (crate::moved::named(to)?, crate::moved::named(from)?);
        from.copy_to(to, size, "the replay header")?;
    }
    let sealed_dwords = &REPLAY_WORK.sealed_header;
    let dwords: Vec<u32> = (1..0x40).map(|n| sealed_dwords[n].get()).collect();
    let sum = seal_checksum(&dwords);
    sealed_dwords[0].set(sum);
    replay.seal_key.set(sum);
    let w = &replay.seal_work;
    let key = [w.key[0].get(), w.key[1].get()];
    w.key[0].set(SEAL_KEY[0]);
    w.key[1].set(SEAL_KEY[1]);
    let (mut a, mut b) = (0u32, 0u32);
    for n in 0..0x20u32 {
        let at = 2 * n as usize;
        let [x, y] = chained(
            [sealed_dwords[at].get(), sealed_dwords[at + 1].get()],
            [a, b],
        );
        w.block[0].set(x);
        w.block[1].set(y);
        // `seal_block` works on the dwords at 0x4E052C .. (no register read).
        crate::race::save_seal::seal_block(w);
        a = w.sealed[0].get();
        sealed_dwords[at].set(a);
        b = w.sealed[1].get();
        sealed_dwords[at + 1].set(b);
    }
    w.key[1].set(key[1]);
    w.key[0].set(key[0]);
    Ok(())
}

/// (bytes, to, from), in the original's order (made from the disassembly by script).
const HEADER_MOVES: [(usize, usize, usize); 31] = [
    (1, 0x719B81, 0x4E07A0),
    (4, 0x719B59, 0x528510),
    (1, 0x719B86, 0x75ACE8),
    (1, 0x719B87, 0x75ACD4),
    (1, 0x719B88, 0x75ACF0),
    (1, 0x719B89, 0x75ACD8),
    (1, 0x719B8A, 0x75AE1C),
    (1, 0x719C8C, 0x75AE1C),
    (1, 0x719B8B, 0x75AE24),
    (4, 0x719B5D, 0x5BECAC),
    (4, 0x719B82, 0x75870C),
    (1, 0x719B94, 0x75AE2C),
    (1, 0x719B95, 0x75AD30),
    (1, 0x719B96, 0x75AD1C),
    (1, 0x719B97, 0x75AD20),
    (1, 0x719B98, 0x75AD2C),
    (1, 0x719B99, 0x75AD24),
    (1, 0x719B9A, 0x75AD28),
    (1, 0x719B9B, 0x71C6D0),
    (1, 0x719B9C, 0x71C6D4),
    (1, 0x719B9D, 0x71C6D8),
    (2, 0x719B9E, 0x71C6DC),
    (4, 0x719BA0, 0x71C6E0),
    (2, 0x719BA4, 0x71C6E4),
    (2, 0x719BA6, 0x71C6E8),
    (2, 0x719BA8, 0x71C6EC),
    (2, 0x719BAA, 0x71C6F0),
    (2, 0x719BAC, 0x71C6F4),
    (2, 0x719BAE, 0x71C6F8),
    (2, 0x719BB0, 0x71C6FC),
    (2, 0x719BB2, 0x71C700),
];

/// `files::write`: the bytes written and the carry (a failed write).
fn write(handle: u32, bytes: &[u8]) -> (u32, bool) {
    let (n, ok) = crate::files::handle::write_slice(handle, bytes);
    (n, !ok)
}

/// `race::write_replay_chunk` (0x41EEE4): `count` bytes at `from` written to file
/// `handle`; unless `dword_5BEC80` is 1 they are packed (`race::pack_runs` into
/// `g_pngInfo`, rounded up to a dword) and the packing written when smaller (eax then the
/// bytes written negated). Gives eax and the carry (a failed write); the original keeps
/// the rest, and its zero flag is read by no caller. Both callers write
/// the whole replay block (`dword_9613A4`, 0x2810 bytes): here it is that block.
pub fn write_replay_chunk(shared: &Shared, handle: u32) -> Result<(u32, bool), Refused> {
    let system = crate::system::owner::system();
    let block = VIEWER_BUFFERS.block_bytes();
    if shared.replay_file.chunks_plain.get() == 1 {
        return Ok(write(handle, &block));
    }
    let (packing, packed) = crate::race::save_seal::pack_runs(&block, system)?;
    // Converted: `rc2k_core::race::replay::packed_size`.
    Ok(
        match rc2k_core::race::replay::packed_size(packed, BLOCK_LEN) {
            Some(size) => {
                // The size rounded up to a dword: within the end mark's three zeros.
                let (n, failed) = write(handle, &packing[..size as usize]);
                if failed {
                    (n, true)
                } else {
                    (n.wrapping_neg(), false)
                }
            }
            None => write(handle, &block),
        },
    )
}

/// The replay index's dword `k` (`lDistanceToMove` ..: offset and size a block). A block
/// count past the index's 1800 (a broken file) reads on past it, as the original does.
pub fn block_place(race: &Race, k: usize) -> Result<u32, BadInput> {
    let shared = &race.shared;
    let places = &shared.replay_file.block_places;
    crate::session::read_on_in(&shared.replay_file, places, k, "the replay index")
}

/// `race::write_replay_block` (0x41E9A3): the block (`dword_9613A4`, 0x2810 bytes) written
/// as a chunk; its offset and size (negative when packed) into the index; the file's
/// length on; the block count on. A failed write or the index full (0x708 blocks) closes
/// the replay (`race::close_replay`; the carry it sets is read by no caller). Every
/// register kept.
pub fn write_replay_block(race: &Race, replay: &Replay) -> Result<(), Refused> {
    let shared = &race.shared;
    let n = shared.replay_file.index_block_count.get();
    if n >= MOST_BLOCKS {
        return Err(BadInput("the replay's index is full already").into());
    }
    shared.replay_file.block_places[2 * n as usize].set(replay.replay_view.file_offset.get());
    let (written, failed) = write_replay_chunk(shared, shared.replay_file.replay_handle.get())?;
    let failed = failed || {
        shared.replay_file.block_places[2 * n as usize + 1].set(written);
        // Converted: `rc2k_core::race::replay::block_written`.
        let (size, full) = rc2k_core::race::replay::block_written(written, n);
        replay
            .replay_view
            .file_offset
            .set(replay.replay_view.file_offset.get().wrapping_add(size));
        shared.replay_file.index_block_count.set(n + 1);
        full
    };
    if failed {
        close_replay(race)?;
    }
    Ok(())
}

/// `race::add_replay_frame` (0x41E8DE): with the replay open, `bytes` bytes at `from` (a
/// frame when 0x10: the frame counts on) put at the cursor (`rep movsd`, `bytes` / 4
/// dwords); at 100 frames the block written (`race::write_replay_block`) and the cursor
/// and count back. Every register kept. Refuses frames past the block's room (before
/// anything). Its one caller adds the padding frame (`REPLAY_VIEW.pad_frame`): here the
/// frame's cells, `bytes` their size.
pub fn add_replay_frame(race: &Race, frame: &[Dword]) -> Result<(), Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    if shared.replay_file.replay_open.get() != 1 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::replay::{fits, frame_added}`.
    use rc2k_core::race::replay::{FRAME_BYTES, fits, frame_added};
    let bytes = 4 * frame.len() as u32;
    let used = VIEWER_BUFFERS.block_used.get();
    let dwords = bytes >> 2;
    if !fits(used, dwords, BLOCK_LEN) {
        return Err(BadInput("a frame past the replay block's room").into());
    }
    let (in_block, full) = frame_added(bytes, replay.replay_view.frames_in_block.get());
    if bytes == FRAME_BYTES {
        shared
            .replay_file
            .index_frame_count
            .set(shared.replay_file.index_frame_count.get().wrapping_add(1));
        replay.replay_view.frames_in_block.set(in_block);
    }
    let first = used as usize / 4;
    for (k, cell) in frame.iter().enumerate() {
        VIEWER_BUFFERS.block[first + k].set(cell.get());
    }
    VIEWER_BUFFERS.block_used.set(used + bytes);
    if full {
        write_replay_block(race, replay)?;
        VIEWER_BUFFERS.block_used.set(0);
        replay.replay_view.frames_in_block.set(0);
    }
    Ok(())
}

/// `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
/// padding frame (`race::add_replay_frame`, which writes it at 100); the last block's
/// frame count (`dword_5C252C`: 100, or for the last block the frames modulo 100 plus
/// one); the time sealed; the index ("RCIB", 0x3850 bytes) written; the file closed;
/// `dword_5BEC88` 0, `dword_5BEC84` 1. Every register kept.
pub fn close_replay(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    if shared.replay_file.replay_open.get() != 1 {
        return Ok(());
    }
    while replay.replay_view.frames_in_block.get() != 0 {
        add_replay_frame(race, &replay.replay_view.pad_frame)?;
    }
    // Converted: `rc2k_core::race::replay::closing_frames`.
    let (frames, _, _) = rc2k_core::race::replay::closing_frames(
        replay.replay_view.replay_block.get(),
        shared.replay_file.index_block_count.get(),
        shared.replay_file.index_frame_count.get(),
    );
    replay.replay_view.frames_in_block.set(frames);
    crate::race::race_shell::seal_stage_time(race)?;
    crate::files::handle::write_cells(
        shared.replay_file.replay_handle.get(),
        shared.replay_file.index(),
    );
    crate::files::handle::close(shared.replay_file.replay_handle.get());
    shared.replay_file.replay_open.set(0);
    shared.replay_file.replay_closed.set(1);
    Ok(())
}

/// `race::open_replay` (0x41E6EC): `dword_5BEC84` 0; with replays on (`dword_5BEC78` 1,
/// `dword_5BEC7C` 0, `dword_52A910` not 1, `dword_75ACDC` not 1) and 30 MB free: the
/// header ("REPL", version 0x50400, the date, "tmpreplay", the race's settings), the
/// car's header (`seal_replay_header`) copied into it, the index "RCIB" cleared, the
/// cursor, counts and length 0; `var\tmprpl` created; the header written. A failed open
/// or write closes it (`race::close_replay`). Every register kept; the carry it leaves is
/// read by none of its callers (taint at both sites).
pub fn open_replay(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let replay = &race.replay;
    // Converted: `rc2k_core::race::replay::{replays_on, room_for_replay}` and the
    // header's and the index's words.
    use rc2k_core::race::replay::{
        HEADER_MAGIC, HEADER_NAME, HEADER_VERSION, INDEX_MAGIC, replays_on, room_for_replay,
    };
    shared.replay_file.replay_closed.set(0);
    if !replays_on(
        shared.replay_file.replays_on.get(),
        shared.replay_file.no_replay_switch.get(),
        net().vars.session.get(),
        shared.state.settings.no_replay.get(),
    ) {
        return Ok(());
    }
    if !room_for_replay(crate::logfile::start_log::free_space_kb()) {
        return Ok(());
    }
    let header = &shared.replays.header;
    for k in 0..0x10C {
        header.set_u32(4 * k, 0);
    }
    header.set_u32(0, HEADER_MAGIC);
    header.set_u32(8, HEADER_VERSION);
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    for (k, &w) in HEADER_NAME.iter().enumerate() {
        header.set_u32(0x10 + 4 * k, w);
    }
    let w = &REPLAY_WORK;
    w.header_one_player
        .set(crate::sound::owner::sound().race.sounds.one_player.get());
    w.header_codriver_sex_0
        .set(shared.state.settings.cars[0].codriver_sex.get());
    w.header_codriver_sex_1
        .set(shared.state.settings.cars[1].codriver_sex.get());
    w.patches_in_header.set(6);
    for (to, from) in w
        .header_patches
        .iter()
        .zip(crate::race::replay_patches::switches(shared))
    {
        to.set(from.get());
    }
    seal_replay_header(replay)?;
    // The store types the stage's 0x400 bytes by their first dword.
    // The file's header is bytes.
    let stage = shared
        .replays
        .header
        .read_at(0x30, 0x400, "the replay header's stage")?;
    GamePtr::to(&REPLAY_WORK.header_stage)
        .unbounded()
        .write_bytes(&stage, "the replay header's stage")?;
    for cell in shared.replay_file.index() {
        cell.set(0);
    }
    shared.replay_file.index_magic.set(INDEX_MAGIC);
    VIEWER_BUFFERS.block_used.set(0);
    replay.replay_view.frames_in_block.set(0);
    replay.replay_view.file_offset.set(0);
    let opened =
        crate::files::handle::open_by_mode(&crate::race::replay_path::last_race(replay), 1)?;
    let failed = match opened {
        None => true,
        Some(handle) => {
            shared.replay_file.replay_handle.set(handle);
            shared.replay_file.replay_open.set(1);
            replay.replay_view.file_offset.set(
                replay
                    .replay_view
                    .file_offset
                    .get()
                    .wrapping_add(HEADER_LEN),
            );
            !crate::files::handle::write_slice(
                shared.replay_file.replay_handle.get(),
                &shared.replays.header.read(),
            )
            .1
        }
    };
    if failed {
        close_replay(race)?;
    }
    Ok(())
}

#[cfg(test)]
mod seal_key_tests {
    use crate::race::owner::race;

    #[test]
    fn seal_key_starts_as_the_games() {
        assert_eq!(crate::moved::image_bytes(0x4F66F9, 4), [0; 4]);
        assert_eq!(race().replay.seal_key.get(), 0);
    }

    #[test]
    fn sealed_header_sits_at_the_games_address() {
        use crate::race::replay_work_store::{ReplayWork, START};
        assert_eq!(
            core::mem::offset_of!(ReplayWork, sealed_header),
            0x719B54 - START
        );
    }
}
