//! Enter on a setting of the network settings screen starts editing it: item 1 the player's
//! name, item 3 the host's address, item 2 the most players as text. Other items do nothing.

use crate::install::owner::Install;
use crate::net::begin_edit::EditedText;
use crate::net::lobby::PALETTE;
use crate::net::owner::Net;
use rc2k_core::net::editing::{self, Setting};

/// Starts a setting's edit (item, text, width and font).
pub fn begin_setting_edit(
    net: &Net,
    install: &Install,
    list_state: &crate::menu::list_state::ListState,
) -> Result<(), crate::errors::Refused> {
    // The logic is in `rc2k_core::net::editing` (`Setting`, `WIDTH`).
    let item = list_state.choice.get();
    let text = match Setting::of(item) {
        Setting::Name => EditedText::MachineName,
        Setting::HostAddress => EditedText::HostAddress,
        Setting::MostPlayers => {
            let mut digits =
                crate::strings::format_u32::format_u32(net.setup().most_players.get(), 0);
            digits.push(0);
            install
                .paths
                .path_a
                .write_at(0, &digits, "a text line's bytes")?;
            EditedText::MostPlayers
        }
        Setting::None => return Ok(()),
    };
    crate::net::begin_edit::begin_edit(net, install, item, text, editing::WIDTH, PALETTE)
}
