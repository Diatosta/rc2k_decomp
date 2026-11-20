//! A key for the setting being edited. When that ends the editing, an empty player name gets
//! its old text back, the most players is read from its text, held to 2 ..= 8 (4 with the
//! notice kind 2), and the port from its text, held to 0 ..= 65535 (0 is the default); the password is kept as typed.

use crate::errors::Refused;
use crate::net::lobby::movsb;
use crate::net::owner::Net;
use rc2k_core::net::chat;

/// The key `key` for the setting being edited.
pub(super) fn setting_key(
    strings: &crate::strings::owner::Strings,
    list_state: &crate::menu::list_state::ListState,
    net: &'static Net,
    key: u8,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(net, &net.settings().edit, key, gfx)? {
        return Ok(());
    }
    // The logic is in `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match list_state.read().choice {
        chat::NAME => {
            if net.setup().machine_name.get(0) == 0 {
                movsb(
                    &net.setup().machine_name,
                    &net.settings().edit.text_before_edit,
                    0x40,
                )?;
            }
        }
        chat::MOST_PLAYERS => {
            let number = {
                crate::settings::parse_number::parse_number(
                    &net.settings().most_players_text.read(),
                )
            };
            net.setup().most_players.set(chat::most_players(
                number,
                crate::strings::locale::notice_kind(strings),
            ));
        }
        chat::PORT => {
            let number = crate::settings::parse_number::parse_number(
                &net.settings().most_players_text.read(),
            );
            net.setup().port.set(chat::port(number));
        }
        chat::PASSWORD => {
            crate::net::password::set(&net.settings().most_players_text.read());
        }
        _ => {}
    }
    Ok(())
}
