//! `net::setting_key` (0x416C76): a key for the setting being edited (`net::edit_key`); when
//! that ends the editing: the player's name (item 1) left empty gets its old text back
//! (`script_path`, 64 bytes); the most players (item 2) read from its text
//! (`settings::parse_number`), held to 2 .. 8 (signed), and to 4 with `g_noticeKind` 2, into
//! `most_players`.
//!
//! Takes the key.
//! Caller: `net::settings_loop`.

use crate::errors::Refused;
use crate::net::lobby::movsb;
use crate::net::owner::Net;
use rc2k_core::net::chat;

/// The key `key` for the setting being edited.
pub fn setting_key(
    install: &'static crate::install::owner::Install,
    net: &'static Net,
    key: u8,
) -> Result<(), Refused> {
    if !crate::net::edit_key::edit_key(net, key)? {
        return Ok(());
    }
    // Converted: `rc2k_core::net::chat` (`NAME`, `MOST_PLAYERS`, `most_players`).
    match install.pages.choice.get() {
        chat::NAME => {
            if net.vars.machine_name.get(0) == 0 {
                movsb(&net.vars.machine_name, &install.paths.script_path, 0x40)?;
            }
        }
        chat::MOST_PLAYERS => {
            let number =
                { crate::settings::parse_number::parse_number(&install.paths.path_a.read()) };
            net.vars.most_players.set(chat::most_players(
                number,
                crate::strings::load_language::notice_kind(),
            ));
        }
        _ => {}
    }
    Ok(())
}
