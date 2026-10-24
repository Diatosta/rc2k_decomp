//! `menu::check_cheat_name` (0x480CBF): the fourth player's name taken as a cheat's: the
//! name coded by rot13 into the text buffer (`strings::rot13_into_text`) is looked for in
//! the list at `aNepnqrNpgvba` (0x7585E0: each entry a coded name, its NUL and the cheat's
//! bit number; a NUL where a name would start ends it), each by `strings::same_text`
//! (case aside). Found: the bit set in `g_cheats`, the menu's sound 8, `menu::apply_cheats`,
//! and the player's name made string 0x18F.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `menu::player4_details_loop` once the name's editing ends. Refuses (before anything) a
//! missing fourth player or name, a list without its end in 64 entries, a bit past 31 (the
//! original's `bts` would reach past `g_cheats`), and what the coding refuses.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::player::player_at;
use crate::strings::rot13_into_text::rot13_into_text;

const LIST_RVA: usize = rva(0x7585E0); // aNepnqrNpgvba
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const HIDDEN_NAME: u32 = 0x18F;
// sound::play_menu_sound, menu::apply_cheats

/// The list's entry and the cheat's bit for the coded name in the text buffer, if it is in
/// the list.
unsafe fn find(coded: *const u8) -> Result<Option<(u32, u32)>, Refused> {
    let list = game_addr(LIST_RVA) as *const u8;
    let coded = unsafe { std::ffi::CStr::from_ptr(coded.cast()) }.to_bytes();
    // Converted: `rc2k_core::menu::cheat_names::find`.
    let found = rc2k_core::menu::cheat_names::find(
        &unsafe { crate::bridge::Raw::new(list) },
        coded,
        &crate::strings::same_text::case_table(),
    )?;
    Ok(found.map(|f| (list as u32 + f.at as u32, f.bit as u32)))
}

/// The Rust version. Every register is kept (the original's `pusha` .. `popa`); the
/// calls get theirs as noted, of which only the sound reads one (eax 8).
pub unsafe fn check_cheat_name() -> Result<(), Refused> {
    let player = player_at(3);
    if player == 0 {
        return Err(BadInput("no fourth player").into());
    }
    let name = unsafe { ((player + 0x18) as *const u32).read_unaligned() };
    unsafe {
        let coded = rot13_into_text(name as *const u8)?;
        let Some((_found, bit)) = find(coded as *const u8)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::menu::cheat_names::with_cheat`.
        let cheats = rc2k_core::menu::cheat_names::with_cheat(game_read(CHEATS_RVA), bit as u8)?;
        game_write(CHEATS_RVA, cheats);
        // The original's registers at the calls: eax 8, ebx the entry, ecx the bit, edi
        // the record.
        crate::sound::play_menu_sound::play_menu_sound_in_game(8);
        crate::menu::apply_cheats::apply_cheats(&mut |line| {
            crate::menu::text_entry::log_l1(line, true)
        })?;
        let mut hidden = Vec::new();
        crate::strings::append_string_by_id::push_string(&mut hidden, HIDDEN_NAME);
        hidden.push(0);
        std::ptr::copy_nonoverlapping(hidden.as_ptr(), name as *mut u8, hidden.len());
    }
    Ok(())
}
