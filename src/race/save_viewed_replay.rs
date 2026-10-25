//! `race::save_viewed_replay` (0x41ED1C): the viewed (and edited) replay saved as replay
//! ebx: with blocks to save, the header made ("REPL", 0, version 0x50400, the date
//! `system::packed_local_time`, the slot's name: 32 bytes of `byte_528934`); the last
//! race's file (`var\tmprpl`) opened to read (mode 0, `dword_975508`) and the slot's
//! (`race::replay_path`) made (mode 1, `dword_97550C`); the undo emptied
//! (`dword_9673F4` 0, its places zeroed: they become the new index); the header written,
//! then each block in the viewer's order: read from its place (`files::seek`,
//! `drive::read_packed` into `dword_9613A4`, neither checked), its first 8 bytes the
//! block's cameras, written packed (`race::write_replay_chunk`), its offset and size
//! (negative when packed) into the new index; then "RCIB" and the count (16 bytes) and the
//! new index (0x3840 bytes). Both closed; the undo emptied again.
//!
//! In:  ebx the slot (`race::replay_path`'s number)
//! Out: carry set when it failed (nothing to save, an open or a write failing); ZF when it
//!      worked; every register kept (`pusha` / `popa`)
//!
//! Callers: `race::viewer_action` (0x15 back into a saved slot, 0x16 into a chosen one),
//! `race::save_last_replay`. Saving into slot 222 would make the file it reads from anew (the
//! original's way, kept).

use crate::errors::Refused;
use crate::files::handle;
use crate::physics::flow::a;
use crate::race::replay_path::{LAST_RACE, replay_path};
use crate::race::viewer::{
    BLOCK_DATA, BLOCKS, CAMERAS, INDEX, PLACES, TABLE_LEN, UNDO_BLOCKS, UNDO_PLACES, get,
    move_dwords, put,
};

const FROM: usize = 0x975508; // dword_975508
const TO: usize = 0x97550C; // dword_97550C
const OFFSET: usize = 0x5C2528; // dword_5C2528: the new file's length so far

/// The Rust version: whether it was saved (the original leaves the carry clear and the
/// zero flag set for a yes, and keeps the registers). `slot` the slot to save into (the
/// original's ebx).
pub fn save_viewed_replay(slot: u32) -> Result<bool, Refused> {
    let done = save(slot)?;
    put(UNDO_BLOCKS, 0);
    Ok(done)
}

/// Whether it was saved (`edx` 1).
fn save(slot: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{has_blocks, SAVED_HEADER, name_offset,
    // next_offset}`.
    use rc2k_core::race::viewer_flow::{
        BLOCK_LEN, CAMERA_DWORDS, HEADER_LEN, INDEX_HEAD_LEN, NAME_LEN, SAVED_HEADER, has_blocks,
        name_offset, next_offset,
    };
    if !has_blocks(get(BLOCKS)) {
        return Ok(false);
    }
    unsafe {
        let header = &crate::race::replay::REPLAYS.header;
        for (k, word) in SAVED_HEADER.into_iter().enumerate() {
            header.set_u32(4 * k, word);
        }
        header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
        let name = crate::race::replay::REPLAYS
            .names
            .as_ptr()
            .wrapping_add(name_offset(slot) as usize);
        std::ptr::copy(name as *const u8, header.as_ptr().add(0x10), NAME_LEN);
        // The last race's file opened to read from, the slot's to write to.
        let Some(from) = handle::open_by_mode(LAST_RACE.addr() as *const u8, 0)? else {
            return Ok(false);
        };
        put(FROM, from);
        let path = replay_path(slot)?.path;
        let Some(to) = handle::open_by_mode(path as *const u8, 1)? else {
            handle::close(get(FROM));
            return Ok(false);
        };
        put(TO, to);
        put(UNDO_BLOCKS, 0);
        crate::system::zero_memory::zero_memory(a(UNDO_PLACES) as *mut u8, TABLE_LEN as u32)?;
        // Both files closed on the way out (the new one only when it was made).
        let close = || {
            handle::close(get(TO));
            handle::close(get(FROM));
        };
        let write = |buffer: u32, len: u32| handle::write(get(TO), buffer as *const u8, len).1;
        put(OFFSET, HEADER_LEN);
        if !write(
            crate::race::replay::REPLAYS.header.addr() as u32,
            HEADER_LEN,
        ) {
            close();
            return Ok(false);
        }
        let mut block = 0u32;
        loop {
            handle::seek(get(FROM), get(PLACES + 8 * block as usize) as i32, 0);
            crate::drive::read_packed::read_packed(
                get(FROM),
                a(BLOCK_DATA),
                get(PLACES + 8 * block as usize + 4),
            )?;
            move_dwords(BLOCK_DATA, CAMERAS + 8 * block as usize, CAMERA_DWORDS);
            put(UNDO_PLACES + 8 * block as usize, get(OFFSET));
            let (written, failed) =
                crate::race::replay::write_replay_chunk(get(TO), a(BLOCK_DATA), BLOCK_LEN)?;
            if failed {
                close();
                return Ok(false);
            }
            put(UNDO_PLACES + 8 * block as usize + 4, written);
            put(OFFSET, next_offset(get(OFFSET), written));
            block += 1;
            if block == get(BLOCKS) {
                break;
            }
        }
        let done = write(a(INDEX), INDEX_HEAD_LEN) && write(a(UNDO_PLACES), TABLE_LEN as u32);
        close();
        Ok(done)
    }
}
