//! The replay file (`var\tmprpl`, IDA 2026-10-17): opened with its header at the race's
//! start (`race::open_replay`, the header sealed by `race::seal_replay_header`), frames
//! gathered (`race::add_replay_frame`), written block by block (`race::write_replay_block`
//! through `race::write_replay_chunk`, which packs by runs), and closed with its index
//! ("RCIB", `race::close_replay`). The steps call each other (a failed block closes the
//! file, which writes the last frames as a block); the file primitives are
//! `files::handle`'s.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::menu::names_store::NAME_BUFFERS;
use crate::pointers::GamePtr;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::seal_store::SEAL_WORK;
use crate::race::state_store::RACE_STATE;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;
use crate::race::viewer_state_store::REPLAY_VIEW;
use crate::session::{Bytes, Dword};
use crate::store::{game_addr, game_write};

/// `dword_4F66F9`: the race record's seal key (the replay header's checksum,
/// `race::seal_replay_header`; `race::seal_stage_time` turns the first car's time by it).
/// Split 2026-10-25 from sound's dwords around it; `moved::MOVED` names IDA 0x4F66F9 here.
pub static SEAL_KEY: crate::session::Dword = crate::session::Dword::new(0);

// The index (`REPLAY_FILE.index()`) is 0x3850 bytes.
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
pub struct Replays {
    /// `dword_528504`: the file's header (a dword array, `HEADER_LEN` bytes).
    pub header: Bytes<{ HEADER_LEN as usize }>,
    /// `byte_528934`: the saved replays' names, 100 of 0x20 bytes (twenty to a page).
    pub names: Bytes<{ SLOTS * NAME_BYTES }>,
}

/// The replays' buffers; the first values are the image's (all zero).
pub static REPLAYS: Replays = Replays {
    header: Bytes::new([0; HEADER_LEN as usize]),
    names: Bytes::new([0; SLOTS * NAME_BYTES]),
};

/// Where the block is (`moved::MOVED`'s home for IDA 0x528504).
pub fn home() -> usize {
    crate::pointers::game_address(&raw const REPLAYS) as usize
}

#[cfg(test)]
mod block_tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&REPLAYS, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(offset_of!(Replays, names), 0x528934 - START);
        assert_eq!(crate::moved::place(START), REPLAYS.header.addr() as usize);
        assert_eq!(crate::moved::place(0x528934), REPLAYS.names.addr() as usize);
        assert_eq!(
            crate::moved::place(0x528510),
            (REPLAYS.header.addr() + 0xC) as usize
        );
    }
}
pub const SEALED_RVA: usize = rva(0x719B54); // dword_719B54: the sealed part, 0x100

fn get(a: usize) -> Result<u32, BadInput> {
    GamePtr::new(game_addr(a)).read("a replay header word")
}

fn set(a: impl crate::store::Where, v: u32) -> Result<(), BadInput> {
    GamePtr::new(crate::store::game_addr(a)).write(v, "a replay header word")
}

/// `race::seal_replay_header` (0x41F0A3): the replay's sealed header part (`dword_719B54`,
/// 0x40 dwords): version byte 6, the player's name (`strings::copy_printable`, 0x20), the
/// settings and options copied (`HEADER_MOVES`), a checksum (from dword 1: add, `rol` 5)
/// into dword 0 and `dword_4F66F9`; then its 32 blocks of 8 chained through
/// `race::seal_block` with the key "halfmoon" (the game's key put back after). Every
/// register kept.
pub fn seal_replay_header() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::replay::{seal_checksum, chained, SEAL_KEY}`.
    use rc2k_core::race::replay::{SEAL_KEY, SEAL_VERSION, chained, seal_checksum};
    game_write::<u8>(rva(0x719B58), SEAL_VERSION);
    crate::strings::copy_printable::copy_printable(
        crate::pointers::GamePtr::to(&NAME_BUFFERS.player_names), // g_playerNames
        crate::pointers::GamePtr::new(game_addr(rva(0x719B61))),
        0x20,
    )?;
    for &(size, to, from) in &HEADER_MOVES {
        // `g_noticeKind` has left the image: its place by `crate::moved`.
        let (to, from) = (crate::moved::place(to), crate::moved::place(from));
        GamePtr::new(from as u32).copy_to(GamePtr::new(to as u32), size, "the replay header")?;
    }
    let dwords: Vec<u32> = (1..0x40)
        .map(|n| get(SEALED_RVA + 4 * n))
        .collect::<Result<_, _>>()?;
    let sum = seal_checksum(&dwords);
    set(SEALED_RVA, sum)?;
    crate::race::replay::SEAL_KEY.set(sum);
    let w = &SEAL_WORK;
    let key = [w.key[0].get(), w.key[1].get()];
    w.key[0].set(SEAL_KEY[0]);
    w.key[1].set(SEAL_KEY[1]);
    let (mut a, mut b) = (0u32, 0u32);
    for n in 0..0x20u32 {
        let at = SEALED_RVA + 8 * n as usize;
        let [x, y] = chained([get(at)?, get(at + 4)?], [a, b]);
        w.block[0].set(x);
        w.block[1].set(y);
        // `seal_block` works on the dwords at 0x4E052C .. (no register read).
        crate::race::save_seal::seal_block();
        a = w.sealed[0].get();
        set(at, a)?;
        b = w.sealed[1].get();
        set(at + 4, b)?;
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
/// the rest (`pusha` / `popa`), and its zero flag is read by no caller. Both callers write
/// the whole replay block (`dword_9613A4`, 0x2810 bytes): here it is that block.
pub fn write_replay_chunk(handle: u32) -> Result<(u32, bool), Refused> {
    let block = VIEWER_BUFFERS.block_bytes();
    if REPLAY_FILE.chunks_plain.get() == 1 {
        return Ok(write(handle, &block));
    }
    let (packing, packed) = crate::race::save_seal::pack_runs(&block)?;
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
pub fn block_place(k: usize) -> Result<u32, BadInput> {
    let places = &REPLAY_FILE.block_places;
    crate::session::read_on(places, k, "the replay index")
}

/// `race::write_replay_block` (0x41E9A3): the block (`dword_9613A4`, 0x2810 bytes) written
/// as a chunk; its offset and size (negative when packed) into the index; the file's
/// length on; the block count on. A failed write or the index full (0x708 blocks) closes
/// the replay (`race::close_replay`; the carry it sets is read by no caller). Every
/// register kept.
pub fn write_replay_block() -> Result<(), Refused> {
    let n = REPLAY_FILE.index_block_count.get();
    if n >= MOST_BLOCKS {
        return Err(BadInput("the replay's index is full already").into());
    }
    REPLAY_FILE.block_places[2 * n as usize].set(REPLAY_VIEW.file_offset.get());
    let (written, failed) = write_replay_chunk(REPLAY_FILE.replay_handle.get())?;
    let failed = failed || {
        REPLAY_FILE.block_places[2 * n as usize + 1].set(written);
        // Converted: `rc2k_core::race::replay::block_written`.
        let (size, full) = rc2k_core::race::replay::block_written(written, n);
        REPLAY_VIEW
            .file_offset
            .set(REPLAY_VIEW.file_offset.get().wrapping_add(size));
        REPLAY_FILE.index_block_count.set(n + 1);
        full
    };
    if failed {
        close_replay()?;
    }
    Ok(())
}

/// `race::add_replay_frame` (0x41E8DE): with the replay open, `bytes` bytes at `from` (a
/// frame when 0x10: the frame counts on) put at the cursor (`rep movsd`, `bytes` / 4
/// dwords); at 100 frames the block written (`race::write_replay_block`) and the cursor
/// and count back. Every register kept. Refuses frames past the block's room (before
/// anything). Its one caller adds the padding frame (`REPLAY_VIEW.pad_frame`): here the
/// frame's cells, `bytes` their size.
pub fn add_replay_frame(frame: &[Dword]) -> Result<(), Refused> {
    if REPLAY_FILE.replay_open.get() != 1 {
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
    let (in_block, full) = frame_added(bytes, REPLAY_VIEW.frames_in_block.get());
    if bytes == FRAME_BYTES {
        REPLAY_FILE
            .index_frame_count
            .set(REPLAY_FILE.index_frame_count.get().wrapping_add(1));
        REPLAY_VIEW.frames_in_block.set(in_block);
    }
    let first = used as usize / 4;
    for (k, cell) in frame.iter().enumerate() {
        VIEWER_BUFFERS.block[first + k].set(cell.get());
    }
    VIEWER_BUFFERS.block_used.set(used + bytes);
    if full {
        write_replay_block()?;
        VIEWER_BUFFERS.block_used.set(0);
        REPLAY_VIEW.frames_in_block.set(0);
    }
    Ok(())
}

/// `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
/// padding frame (`race::add_replay_frame`, which writes it at 100); the last block's
/// frame count (`dword_5C252C`: 100, or for the last block the frames modulo 100 plus
/// one); the time sealed; the index ("RCIB", 0x3850 bytes) written; the file closed;
/// `dword_5BEC88` 0, `dword_5BEC84` 1. Every register kept.
pub fn close_replay() -> Result<(), Refused> {
    if REPLAY_FILE.replay_open.get() != 1 {
        return Ok(());
    }
    while REPLAY_VIEW.frames_in_block.get() != 0 {
        add_replay_frame(&REPLAY_VIEW.pad_frame)?;
    }
    // Converted: `rc2k_core::race::replay::closing_frames`.
    let (frames, _, _) = rc2k_core::race::replay::closing_frames(
        REPLAY_VIEW.replay_block.get(),
        REPLAY_FILE.index_block_count.get(),
        REPLAY_FILE.index_frame_count.get(),
    );
    REPLAY_VIEW.frames_in_block.set(frames);
    crate::race::race_shell::seal_stage_time()?;
    crate::files::handle::write_cells(REPLAY_FILE.replay_handle.get(), REPLAY_FILE.index());
    crate::files::handle::close(REPLAY_FILE.replay_handle.get());
    REPLAY_FILE.replay_open.set(0);
    REPLAY_FILE.replay_closed.set(1);
    Ok(())
}

/// `race::open_replay` (0x41E6EC): `dword_5BEC84` 0; with replays on (`dword_5BEC78` 1,
/// `dword_5BEC7C` 0, `dword_52A910` not 1, `dword_75ACDC` not 1) and 30 MB free: the
/// header ("REPL", version 0x50400, the date, "tmpreplay", the race's settings), the
/// car's header (`seal_replay_header`) copied into it, the index "RCIB" cleared, the
/// cursor, counts and length 0; `var\tmprpl` created; the header written. A failed open
/// or write closes it (`race::close_replay`). Every register kept; the carry it leaves is
/// read by none of its callers (taint at both sites).
pub fn open_replay() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::replay::{replays_on, room_for_replay}` and the
    // header's and the index's words.
    use rc2k_core::race::replay::{
        HEADER_MAGIC, HEADER_NAME, HEADER_VERSION, INDEX_MAGIC, replays_on, room_for_replay,
    };
    REPLAY_FILE.replay_closed.set(0);
    if !replays_on(
        REPLAY_FILE.replays_on.get(),
        REPLAY_FILE.no_replay_switch.get(),
        crate::net::store::NET.session.get(),
        RACE_STATE.settings.no_replay.get(),
    ) {
        return Ok(());
    }
    if !room_for_replay(crate::logfile::start_log::free_space_kb()) {
        return Ok(());
    }
    let header = &REPLAYS.header;
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
        .set(crate::sound::store::RACE_SOUNDS.one_player.get());
    w.header_codriver_sex_0
        .set(RACE_STATE.settings.cars[0].codriver_sex.get());
    w.header_codriver_sex_1
        .set(RACE_STATE.settings.cars[1].codriver_sex.get());
    w.patches_in_header.set(6);
    for (to, from) in w
        .header_patches
        .iter()
        .zip(crate::race::replay_patches::switches())
    {
        to.set(from.get());
    }
    seal_replay_header()?;
    GamePtr::to(&REPLAY_WORK.header_stage).copy_to(
        GamePtr::new(REPLAYS.header.addr() + 0x30),
        0x400,
        "the replay header's stage",
    )?;
    for cell in REPLAY_FILE.index() {
        cell.set(0);
    }
    REPLAY_FILE.index_magic.set(INDEX_MAGIC);
    VIEWER_BUFFERS.block_used.set(0);
    REPLAY_VIEW.frames_in_block.set(0);
    REPLAY_VIEW.file_offset.set(0);
    let opened = crate::files::handle::open_by_mode(&crate::race::replay_path::last_race(), 1)?;
    let failed = match opened {
        None => true,
        Some(handle) => {
            REPLAY_FILE.replay_handle.set(handle);
            REPLAY_FILE.replay_open.set(1);
            REPLAY_VIEW
                .file_offset
                .set(REPLAY_VIEW.file_offset.get().wrapping_add(HEADER_LEN));
            !crate::files::handle::write_slice(
                REPLAY_FILE.replay_handle.get(),
                &REPLAYS.header.read(),
            )
            .1
        }
    };
    if failed {
        close_replay()?;
    }
    Ok(())
}

#[cfg(test)]
mod seal_key_tests {
    #[test]
    fn seal_key_starts_as_the_games() {
        assert_eq!(crate::moved::image_bytes(0x4F66F9, 4), [0; 4]);
        assert_eq!(super::SEAL_KEY.get(), 0);
    }
}
