//! `gfx::free_picture_buffer` (0x4060B7): the picture buffer given back at the game's end:
//! `g_pictureFlags` and `g_pictureBuffer` 0, its list (`stru_4F49E4`) freed
//! (`memory::free_list`). The other half of `gfx::allocate_picture_buffer`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Its call through `part` (the Rust `free_list`). Refuses
//! (before anything) what `free_list` refuses: the game's own heap with blocks on the list.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::memory::OWN_HEAP_RVA;
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{FREE_LIST, Part, exit_entry};

const FLAGS: usize = 0x4F49CC; // g_pictureFlags
const BUFFER: usize = 0x4F49DC; // g_pictureBuffer
const LIST: usize = 0x4F49E4; // stru_4F49E4

/// The Rust version on the variables at `flags`, `buffer` and the list at `list`.
pub fn free_picture_buffer_with(
    r: &mut Regs,
    flags: u32,
    buffer: u32,
    list: u32,
    part: Part,
) -> Result<(), Refused> {
    let head = unsafe { (list as *const u32).read_volatile() };
    if head != 0 && unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)").into());
    }
    let mut g = Regs {
        eax: 0,
        ebx: list,
        ..*r
    };
    unsafe {
        (flags as *mut u32).write_volatile(0);
        (buffer as *mut u32).write_volatile(0);
    }
    part(FREE_LIST, &mut g)
}

fn in_game(r: &mut Regs, part: Part, _api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    free_picture_buffer_with(r, a(FLAGS), a(BUFFER), a(LIST), part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}
