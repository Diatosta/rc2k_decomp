//! `menu::check_cheat_name`: the fourth player's name taken as a cheat's. The name coded by
//! rot13 into the text buffer (`strings::rot13_into_text`) is looked for in the list of coded
//! cheat names (each entry a coded name, its NUL and the cheat's bit number; a NUL where a name
//! would start ends the list), each by `strings::same_text` (case aside). Found: the bit is set
//! in the cheats, the menu's sound 8 plays, `menu::apply_cheats` runs, and the player's name
//! becomes string 0x18F.
//!
//! Refuses (before anything) a missing fourth player or name, a list without its end in 64 entries,
//! a bit past 31 (past the cheats' 32 bits), and what the coding refuses.

use crate::errors::Refused;
use crate::memory::cells::Store;
use crate::menu::owner::Play;
use crate::strings::rot13_into_text::rot13_into_text;

const HIDDEN_NAME: u32 = 0x18F;
// sound::play_menu_sound, menu::apply_cheats

/// The list's entry and the cheat's bit for the coded name in the text buffer, if it is in
/// the list.
fn find(play: &Play, coded: &[u8]) -> Result<Option<(u32, u32)>, Refused> {
    let mut list = vec![0; play.cheats.cheat_names.size()];
    play.cheats.cheat_names.load(0, &mut list)?;
    let coded: Vec<u8> = coded.iter().copied().take_while(|&b| b != 0).collect();
    let found = rc2k_core::menu::cheat_names::find(
        &list,
        &coded,
        &crate::strings::same_text::case_table(),
    )?;
    Ok(found.map(|f| (f.at as u32, f.bit as u32)))
}

pub fn check_cheat_name(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, race) = (o.menus, o.race);
    let play = &menus.play;
    let name = crate::menu::player_names::NameAt::of(crate::menu::player_names::name_of(
        &crate::menu::standings::player(menus, 3).get(),
    )?);
    let coded = rot13_into_text(o.strings, &name.tail(&menus.play))?;
    let Some((_found, bit)) = find(play, &coded)? else {
        return Ok(());
    };
    let cheats = rc2k_core::menu::cheat_names::with_cheat(play.cheats.cheats.get(), bit as u8)?;
    play.cheats.cheats.set(cheats);
    // The sound, the cheat bit and the record are handed on as the callees expect.
    crate::sound::play_menu_sound::play_menu_sound_in_game(
        o.sound,
        o.audio,
        o.menus,
        &o.system.session.picture_clock,
        8,
    );
    crate::menu::apply_cheats::apply_cheats(
        menus,
        &mut |line| {
            crate::logfile::log_write::log_nul(line, true, 1);
        },
        race,
    )?;
    let mut hidden = Vec::new();
    crate::strings::append_string_by_id::push_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &mut hidden,
        HIDDEN_NAME,
    )?;
    hidden.push(0);
    name.place(menus).write(&hidden, "a player\'s name")?;
    Ok(())
}
