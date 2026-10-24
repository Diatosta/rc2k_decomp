//! `saves::load_best_times` (0x4AB155): reads the best times and scores, `var\hst.dat`.
//!
//! In:  nothing
//! Out: carry = not loaded (loaded: carry and zero clear); every register preserved
//! First `saves::allocate_best_times`; the file is opened (mode 0); its head of 0x10 bytes
//! goes to the scratch block at 0x93E8B0 and must start "R5HS" and have 0x50004 at +8; then
//! six of the tables are read straight from the file, each to the address and with the size
//! its pair of variables holds (tables 1, 2, 3, 4, 7 and 8 of `allocate_best_times`: 0xBB8
//! bytes, an entry of 0x618 a stage, 0x15E0, 0xDB60, 0xDB60, an entry a stage again); the
//! file is closed and `dword_747614` = 1.
//! A read that fails or gives fewer bytes than asked for (`sub_4AB21C`, inlined) closes
//! the file and ends it there, with what was read so far left in the tables. A head that
//! is not the right one ends it without the file being closed: the original's slip, kept.
//! One caller, `sub_411239`. Refuses what `resolve_path` and `allocate_best_times` refuse.

use crate::errors::{BadInput, Refused};
use crate::files::handle::{close, open_by_mode, read};
use crate::framework::{game_addr, rva};
use crate::memory::MemoryList;
use crate::saves::allocate_best_times::{VARS_RVA, allocate_best_times, pair};
use crate::settings::TEXT_RVA;

// ---- game data ----

const PATH_RVA: usize = rva(0x5295E4); // "var\hst.dat"
const LOADED_RVA: usize = rva(0x747614); // dword_747614
const LIST_RVA: usize = rva(0x7475D0); // stru_7475D0
const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
// "R5HS"
const HEAD_LEN: u32 = 0x10;
/// The tables of `allocate_best_times` the file holds, in the file's order.
const READ: [usize; 6] = [1, 2, 3, 4, 7, 8];

// ---- Rust version ----

/// What the function is given: the game's own.
pub struct Places<'a> {
    pub path: *const u8,
    /// `saves::allocate_best_times`.
    pub allocate: &'a mut dyn FnMut() -> Result<(), Refused>,
    /// `files::open_by_mode` with mode 0.
    pub open: &'a mut dyn FnMut(*const u8) -> Result<Option<u32>, BadInput>,
    /// The address and the size of the `n`th table of the file, as they are after
    /// `allocate`.
    pub table: &'a dyn Fn(usize) -> (u32, u32),
    pub loaded: *mut u32,
}

/// Reads exactly `count` bytes, or closes the file and says false.
unsafe fn read_or_close(file: u32, to: *mut u8, count: u32) -> bool {
    unsafe {
        let (got, ok) = read(file, to, count);
        if !ok || got != count {
            close(file);
            return false;
        }
        true
    }
}

/// True if the file was loaded.
pub unsafe fn load_best_times(places: &mut Places) -> Result<bool, Refused> {
    unsafe {
        (places.allocate)()?;
        let Some(file) = (places.open)(places.path)? else {
            return Ok(false);
        };
        if !read_or_close(file, game_addr(TEXT_RVA) as *mut u8, HEAD_LEN) {
            return Ok(false);
        }
        // Converted: `rc2k_core::saves::kept::head_ok`.
        let head = std::slice::from_raw_parts(game_addr(TEXT_RVA) as *const u8, HEAD_LEN as usize);
        if !rc2k_core::saves::kept::head_ok(head, rc2k_core::saves::kept::BEST_TIMES) {
            // The file stays open, as in the original.
            return Ok(false);
        }
        for n in 0..READ.len() {
            let (to, count) = (places.table)(n);
            if !read_or_close(file, to as *mut u8, count) {
                return Ok(false);
            }
        }
        close(file);
        places.loaded.write_unaligned(1);
        Ok(true)
    }
}

fn game_table(n: usize) -> (u32, u32) {
    unsafe {
        let pair = pair(game_addr(VARS_RVA) as *mut u32, READ[n]);
        (pair.read_unaligned(), pair.add(1).read_unaligned())
    }
}

/// `load_best_times` on the game's own file and tables.
pub unsafe fn load_game_best_times() -> Result<bool, Refused> {
    unsafe {
        load_best_times(&mut Places {
            path: game_addr(PATH_RVA) as *const u8,
            allocate: &mut || {
                allocate_best_times(
                    game_addr(VARS_RVA) as *mut u32,
                    game_addr(LIST_RVA) as *mut MemoryList,
                    game_addr(RECORD_TABLE_RVA) as *const u8,
                )
            },
            open: &mut |path| open_by_mode(path, 0),
            table: &game_table,
            loaded: game_addr(LOADED_RVA) as *mut u32,
        })
    }
}
