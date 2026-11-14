//! A key for the setting being edited. When that ends the editing, an empty player name gets
//! its old text back and the most players is read from its text, held to 2 ..= 8 (4 with the
//! notice kind 2).

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::lobby::movsb;
use crate::net::owner::Net;
use rc2k_core::net::chat;

/// The key `key` for the setting being edited.
pub fn setting_key(
    input: &crate::input::owner::Input,
    strings: &crate::strings::owner::Strings,
    install: &'static Install,
    list_state: &crate::menu::list_state::ListState,
    net: &'static Net,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(input, net, install, key, gfx)? {
        return Ok(());
    }
    // The logic is in `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match list_state.choice.get() {
        chat::NAME => {
            if net.setup().machine_name.get(0) == 0 {
                movsb(&net.setup().machine_name, &install.paths.script_path, 0x40)?;
            }
        }
        chat::MOST_PLAYERS => {
            let number =
                { crate::settings::parse_number::parse_number(&install.paths.path_a.read()) };
            net.setup().most_players.set(chat::most_players(
                number,
                crate::strings::load_language::notice_kind(strings),
            ));
        }
        _ => {}
    }
    Ok(())
}
