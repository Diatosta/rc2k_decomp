//! `system::restore_error_mode` (0x402B95): at the game's end, if the error mode was
//! changed (`dword_4E059C` 1, `sub_402B7C`, agent A's), the switch 0 and the mode it had
//! before (`uMode`) put back (`SetErrorMode`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out (0x401116, also the short one). `SetErrorMode` through
//! `api` (the same function in the standalone).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::system::api_stand_ins::Api;
use crate::system::exit_parts::{Part, SET_ERROR_MODE, exit_entry};

const CHANGED: usize = 0x4E059C; // dword_4E059C: 1 after `sub_402B7C` set the mode
const MODE: usize = 0x4E05A0; // uMode: the mode before

/// The Rust version on the switch at `changed` and the mode at `mode`.
pub fn restore_error_mode_with(
    _r: &mut Regs,
    changed: u32,
    mode: u32,
    api: Api,
) -> Result<(), Refused> {
    unsafe {
        if (changed as *const u32).read_volatile() != 1 {
            return Ok(());
        }
        (changed as *mut u32).write_volatile(0);
        api(SET_ERROR_MODE, &[(mode as *const u32).read_volatile()]);
    }
    Ok(())
}

fn in_game(r: &mut Regs, _part: Part, api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    restore_error_mode_with(r, a(CHANGED), a(MODE), api)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}
