//! The replay file (`var\tmprpl`, IDA 2026-10-17): opened with its header at the race's
//! start (`race::open_replay`, the header sealed by `race::seal_replay_header`), frames
//! gathered (`race::add_replay_frame`), written block by block (`race::write_replay_block`
//! through `race::write_replay_chunk`, which packs by runs), and closed with its index
//! ("RCIB", `race::close_replay`). The steps call each other (a failed block closes the
//! file, which writes the last frames as a block); the file primitives are
//! `files::handle`'s.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::sound::store::WheelsSlot;
use crate::store::{game_addr, game_write};

pub const PLAIN_RVA: usize = rva(0x5BEC80); // dword_5BEC80: 1 = chunks written unpacked
pub const OPEN_RVA: usize = rva(0x5BEC88); // dword_5BEC88: the file is open
pub const DONE_RVA: usize = rva(0x5BEC84); // dword_5BEC84: a replay was closed
pub const HANDLE_RVA: usize = rva(0x5BEC8C); // h
pub const PACKED_RVA: usize = rva(0x93E8B0); // g_pngInfo: the packing's buffer
pub const INDEX_RVA: usize = rva(0x5BECC8); // "RCIB": the index, 0x3850 bytes
pub const INDEX_LEN: u32 = 0x3850;
pub const BLOCKS_RVA: usize = rva(0x5BECD0); // its block count
pub const FRAMES_RVA: usize = rva(0x5BECD4); // its frame count
pub const PLACES_RVA: usize = rva(0x5BECD8); // its (offset, size) per block
pub const MOST_BLOCKS: u32 = rc2k_core::race::replay::MOST_BLOCKS;
pub const OFFSET_RVA: usize = rva(0x5C2528); // dword_5C2528: the file's length so far
pub const IN_BLOCK_RVA: usize = rva(0x5C252C); // dword_5C252C: frames in the block
pub const LAST_BLOCK_RVA: usize = rva(0x5C2530); // dword_5C2530
pub const PAD_FRAME_RVA: usize = rva(0x5C2518); // unk_5C2518: the frame padding a block
pub const CURSOR_RVA: usize = rva(0x9613A0); // dword_9613A0: where the next frame goes
pub const BLOCK_RVA: usize = rva(0x9613A4); // dword_9613A4: the block, 0x2810 bytes
pub const BLOCK_LEN: u32 = 0x2810;
pub const HEADER_RVA: usize = rva(0x528504); // dword_528504: the file's header, 0x430
pub const HEADER_LEN: u32 = 0x430;
pub const CAR_HEADER_RVA: usize = rva(0x719B14); // dword_719B14: 0x400 bytes
pub const SEALED_RVA: usize = rva(0x719B54); // dword_719B54: the sealed part, 0x100
const PATH_RVA: usize = rva(0x5295F0); // "var\tmprpl"

unsafe fn get(a: usize) -> u32 {
    unsafe { (game_addr(a) as *const u32).read_unaligned() }
}

unsafe fn set(a: impl crate::store::Where, v: u32) {
    unsafe { (crate::store::game_addr(a) as *mut u32).write_unaligned(v) };
}

/// `race::seal_replay_header` (0x41F0A3): the replay's sealed header part (`dword_719B54`,
/// 0x40 dwords): version byte 6, the player's name (`strings::copy_printable`, 0x20), the
/// settings and options copied (`HEADER_MOVES`), a checksum (from dword 1: add, `rol` 5)
/// into dword 0 and `dword_4F66F9`; then its 32 blocks of 8 chained through
/// `race::seal_block` with the key "halfmoon" (the game's key put back after). Every
/// register kept.
pub unsafe fn seal_replay_header() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::replay::{seal_checksum, chained, SEAL_KEY}`.
    use rc2k_core::race::replay::{SEAL_KEY, SEAL_VERSION, chained, seal_checksum};
    unsafe {
        game_write::<u8>(rva(0x719B58), SEAL_VERSION);
        crate::strings::copy_printable::copy_printable(
            game_addr(rva(0x755784)) as *const u8, // g_playerNames
            game_addr(rva(0x719B61)) as *mut u8,
            0x20,
        )?;
        for &(size, to, from) in &HEADER_MOVES {
            // `g_noticeKind` has left the image: its place by `crate::moved`.
            let (to, from) = (crate::moved::place(to), crate::moved::place(from));
            std::ptr::copy(from as *const u8, to as *mut u8, size);
        }
        let dwords: Vec<u32> = (1..0x40).map(|n| get(SEALED_RVA + 4 * n)).collect();
        let sum = seal_checksum(&dwords);
        set(SEALED_RVA, sum);
        set(WheelsSlot::at(0x4F66F9), sum);
        let key = [get(rva(0x4E052C)), get(rva(0x4E0530))];
        set(rva(0x4E052C), SEAL_KEY[0]);
        set(rva(0x4E0530), SEAL_KEY[1]);
        let (mut a, mut b) = (0u32, 0u32);
        for n in 0..0x20u32 {
            let at = SEALED_RVA + 8 * n as usize;
            let [x, y] = chained([get(at), get(at + 4)], [a, b]);
            set(rva(0x4E0534), x);
            set(rva(0x4E0538), y);
            // `seal_block` works on the dwords at 0x4E052C .. (no register read).
            crate::race::save_seal::seal_block();
            a = get(rva(0x4E053C));
            set(at, a);
            b = get(rva(0x4E0540));
            set(at + 4, b);
        }
        set(rva(0x4E0530), key[1]);
        set(rva(0x4E052C), key[0]);
    }
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
unsafe fn write(handle: u32, from: u32, count: u32) -> (u32, bool) {
    let (n, ok) = unsafe { crate::files::handle::write(handle, from as *const u8, count) };
    (n, !ok)
}

/// `race::write_replay_chunk` (0x41EEE4): `count` bytes at `from` written to file
/// `handle`; unless `dword_5BEC80` is 1 they are packed (`race::pack_runs` into
/// `g_pngInfo`, rounded up to a dword) and the packing written when smaller (eax then the
/// bytes written negated). Gives eax and the carry (a failed write); the original keeps
/// the rest (`pusha` / `popa`), and its zero flag is read by no caller.
pub unsafe fn write_replay_chunk(
    handle: u32,
    from: u32,
    count: u32,
) -> Result<(u32, bool), Refused> {
    unsafe {
        if get(PLAIN_RVA) == 1 {
            return Ok(write(handle, from, count));
        }
        let packed_at = game_addr(PACKED_RVA) as u32;
        let packed = crate::race::save_seal::pack_runs(from, count, packed_at);
        // Converted: `rc2k_core::race::replay::packed_size`.
        Ok(match rc2k_core::race::replay::packed_size(packed, count) {
            Some(size) => {
                let (n, failed) = write(handle, packed_at, size);
                if failed {
                    (n, true)
                } else {
                    (n.wrapping_neg(), false)
                }
            }
            None => write(handle, from, count),
        })
    }
}

/// `race::write_replay_block` (0x41E9A3): the block (`dword_9613A4`, 0x2810 bytes) written
/// as a chunk; its offset and size (negative when packed) into the index; the file's
/// length on; the block count on. A failed write or the index full (0x708 blocks) closes
/// the replay (`race::close_replay`; the carry it sets is read by no caller). Every
/// register kept.
pub unsafe fn write_replay_block() -> Result<(), Refused> {
    unsafe {
        let n = get(BLOCKS_RVA);
        if n >= MOST_BLOCKS {
            return Err(BadInput("the replay's index is full already").into());
        }
        set(PLACES_RVA + 8 * n as usize, get(OFFSET_RVA));
        let (written, failed) =
            write_replay_chunk(get(HANDLE_RVA), game_addr(BLOCK_RVA) as u32, BLOCK_LEN)?;
        let failed = failed || {
            set(PLACES_RVA + 8 * n as usize + 4, written);
            // Converted: `rc2k_core::race::replay::block_written`.
            let (size, full) = rc2k_core::race::replay::block_written(written, n);
            set(OFFSET_RVA, get(OFFSET_RVA).wrapping_add(size));
            set(BLOCKS_RVA, n + 1);
            full
        };
        if failed {
            close_replay()?;
        }
    }
    Ok(())
}

/// `race::add_replay_frame` (0x41E8DE): with the replay open, `bytes` bytes at `from` (a
/// frame when 0x10: the frame counts on) put at the cursor (`rep movsd`, `bytes` / 4
/// dwords); at 100 frames the block written (`race::write_replay_block`) and the cursor
/// and count back. Every register kept. Refuses frames past the block's room (before
/// anything).
pub unsafe fn add_replay_frame(from: u32, bytes: u32) -> Result<(), Refused> {
    unsafe {
        if get(OPEN_RVA) != 1 {
            return Ok(());
        }
        // Converted: `rc2k_core::race::replay::{fits, frame_added}`.
        use rc2k_core::race::replay::{FRAME_BYTES, fits, frame_added};
        let cursor = get(CURSOR_RVA);
        let block = game_addr(BLOCK_RVA) as u32;
        let dwords = bytes >> 2;
        if cursor < block || !fits(cursor - block, dwords, BLOCK_LEN) {
            return Err(BadInput("a frame past the replay block's room").into());
        }
        let (in_block, full) = frame_added(bytes, get(IN_BLOCK_RVA));
        if bytes == FRAME_BYTES {
            set(FRAMES_RVA, get(FRAMES_RVA).wrapping_add(1));
            set(IN_BLOCK_RVA, in_block);
        }
        std::ptr::copy(from as *const u8, cursor as *mut u8, 4 * dwords as usize);
        set(CURSOR_RVA, cursor + 4 * dwords);
        if full {
            write_replay_block()?;
            set(CURSOR_RVA, game_addr(BLOCK_RVA) as u32);
            set(IN_BLOCK_RVA, 0);
        }
    }
    Ok(())
}

/// `race::close_replay` (0x41E85A): with the replay open, the block filled up with the
/// padding frame (`race::add_replay_frame`, which writes it at 100); the last block's
/// frame count (`dword_5C252C`: 100, or for the last block the frames modulo 100 plus
/// one); the time sealed; the index ("RCIB", 0x3850 bytes) written; the file closed;
/// `dword_5BEC88` 0, `dword_5BEC84` 1. Every register kept.
pub unsafe fn close_replay() -> Result<(), Refused> {
    unsafe {
        if get(OPEN_RVA) != 1 {
            return Ok(());
        }
        while get(IN_BLOCK_RVA) != 0 {
            add_replay_frame(game_addr(PAD_FRAME_RVA) as u32, 0x10)?;
        }
        // Converted: `rc2k_core::race::replay::closing_frames`.
        let (frames, _, _) = rc2k_core::race::replay::closing_frames(
            get(LAST_BLOCK_RVA),
            get(BLOCKS_RVA),
            get(FRAMES_RVA),
        );
        set(IN_BLOCK_RVA, frames);
        crate::race::race_shell::seal_stage_time();
        write(get(HANDLE_RVA), game_addr(INDEX_RVA) as u32, INDEX_LEN);
        crate::files::handle::close(get(HANDLE_RVA));
        set(OPEN_RVA, 0);
        set(DONE_RVA, 1);
    }
    Ok(())
}

/// `race::open_replay` (0x41E6EC): `dword_5BEC84` 0; with replays on (`dword_5BEC78` 1,
/// `dword_5BEC7C` 0, `dword_52A910` not 1, `dword_75ACDC` not 1) and 30 MB free: the
/// header ("REPL", version 0x50400, the date, "tmpreplay", the race's settings), the
/// car's header (`seal_replay_header`) copied into it, the index "RCIB" cleared, the
/// cursor, counts and length 0; `var\tmprpl` created; the header written. A failed open
/// or write closes it (`race::close_replay`). Every register kept; the carry it leaves is
/// read by none of its callers (taint at both sites).
pub unsafe fn open_replay() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::replay::{replays_on, room_for_replay}` and the
        // header's and the index's words.
        use rc2k_core::race::replay::{
            HEADER_MAGIC, HEADER_NAME, HEADER_VERSION, INDEX_MAGIC, replays_on, room_for_replay,
        };
        set(DONE_RVA, 0);
        if !replays_on(
            get(rva(0x5BEC78)),
            get(rva(0x5BEC7C)),
            get(rva(0x52A910)),
            get(rva(0x75ACDC)),
        ) {
            return Ok(());
        }
        if !room_for_replay(crate::logfile::start_log::free_space_kb()) {
            return Ok(());
        }
        for k in 0..0x10C {
            set(HEADER_RVA + 4 * k, 0);
        }
        set(HEADER_RVA, HEADER_MAGIC);
        set(HEADER_RVA + 8, HEADER_VERSION);
        set(
            HEADER_RVA + 0xC,
            crate::system::packed_local_time::packed_local_time(),
        );
        for (k, &w) in HEADER_NAME.iter().enumerate() {
            set(HEADER_RVA + 0x10 + 4 * k, w);
        }
        set(
            rva(0x719B34),
            crate::store::game_read::<u32>(crate::sound::store::ObjectsSlot::at(0x514F78)),
        );
        set(rva(0x719B38), get(rva(0x75AE20)));
        set(rva(0x719B3C), get(rva(0x75AF8C)));
        set(rva(0x719C54), 6);
        std::ptr::copy(
            game_addr(rva(0x71C6D0)) as *const u8,
            game_addr(rva(0x719C58)) as *mut u8,
            0xD * 4,
        );
        seal_replay_header()?;
        std::ptr::copy(
            game_addr(CAR_HEADER_RVA) as *const u8,
            game_addr(HEADER_RVA + 0x30) as *mut u8,
            0x400,
        );
        std::ptr::write_bytes(game_addr(INDEX_RVA) as *mut u8, 0, INDEX_LEN as usize);
        set(INDEX_RVA, INDEX_MAGIC);
        set(CURSOR_RVA, game_addr(BLOCK_RVA) as u32);
        set(IN_BLOCK_RVA, 0);
        set(OFFSET_RVA, 0);
        let opened = crate::files::handle::open_by_mode(game_addr(PATH_RVA) as *const u8, 1)?;
        let failed = match opened {
            None => true,
            Some(handle) => {
                set(HANDLE_RVA, handle);
                set(OPEN_RVA, 1);
                set(OFFSET_RVA, get(OFFSET_RVA).wrapping_add(HEADER_LEN));
                write(get(HANDLE_RVA), game_addr(HEADER_RVA) as u32, HEADER_LEN).1
            }
        };
        if failed {
            close_replay()?;
        }
    }
    Ok(())
}
