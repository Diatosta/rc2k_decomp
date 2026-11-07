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

use crate::errors::Refused;
use crate::menu::owner::Play;
use crate::pointers::{GamePtr, Store};
use crate::strings::rot13_into_text::rot13_into_text;
use rc2k_core::strings::bytes::Bytes;

const HIDDEN_NAME: u32 = 0x18F;
// sound::play_menu_sound, menu::apply_cheats

/// The list's entry and the cheat's bit for the coded name in the text buffer, if it is in
/// the list.
fn find(play: &Play, coded: GamePtr) -> Result<Option<(u32, u32)>, Refused> {
    let mut list = vec![0; play.cheats.cheat_names.size()];
    play.cheats.cheat_names.load(0, &mut list)?;
    let text = coded.bytes("a coded name")?;
    let coded: Vec<u8> = (0..)
        .map_while(|n| text.byte(n))
        .take_while(|&b| b != 0)
        .collect();
    // Converted: `rc2k_core::menu::cheat_names::find`.
    let found = rc2k_core::menu::cheat_names::find(
        &list,
        &coded,
        &crate::strings::same_text::case_table(),
    )?;
    Ok(found.map(|f| (f.at as u32, f.bit as u32)))
}

/// The Rust version. Every register is kept (the original's `pusha` .. `popa`); the
/// calls get theirs as noted, of which only the sound reads one (eax 8).
pub fn check_cheat_name(play: &Play) -> Result<(), Refused> {
    let name = crate::menu::player_names::name_ptr(crate::menu::player_names::name_of(
        &crate::menu::standings::player(3).get(),
    )?);
    let coded = rot13_into_text(name)?;
    let Some((_found, bit)) = find(play, coded)? else {
        return Ok(());
    };
    // Converted: `rc2k_core::menu::cheat_names::with_cheat`.
    let cheats = rc2k_core::menu::cheat_names::with_cheat(play.cheats.cheats.get(), bit as u8)?;
    play.cheats.cheats.set(cheats);
    // The original's registers at the calls: eax 8, ebx the entry, ecx the bit, edi
    // the record.
    crate::sound::play_menu_sound::play_menu_sound_in_game(8);
    crate::menu::apply_cheats::apply_cheats(&mut |line| {
        crate::logfile::log_write::log_nul(line, true, 1);
    })?;
    let mut hidden = Vec::new();
    crate::strings::append_string_by_id::push_string(&mut hidden, HIDDEN_NAME)?;
    hidden.push(0);
    name.write_bytes(&hidden, "a player\'s name")?;
    Ok(())
}
