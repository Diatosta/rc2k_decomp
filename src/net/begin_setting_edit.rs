//! Enter on a setting of the network settings screen starts editing it: item 1 the player's
//! name, item 3 the host's address, item 2 the most players and item 4 the port as text, item 5 the password. Other items do nothing.

use crate::net::begin_edit::EditedText;
use crate::net::lobby::PALETTE;
use crate::net::owner::Net;
use rc2k_core::net::editing::{self, Setting};

/// Starts a setting's edit (item, text, width and font).
pub fn begin_setting_edit(
    net: &Net,
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
            net.settings()
                .most_players_text
                .write_at(0, &digits, "a text line's bytes")?;
            EditedText::MostPlayers
        }
        Setting::Port => {
            let port = rc2k_core::net::chat::shown_port(net.setup().port.get());
            let mut digits = crate::strings::format_u32::format_u32(port, 0);
            digits.push(0);
            net.settings()
                .most_players_text
                .write_at(0, &digits, "a text line's bytes")?;
            EditedText::MostPlayers
        }
        Setting::Password => {
            // The old password is what Escape brings back; the typing starts from nothing.
            let mut text = crate::net::password::get();
            text.push(0);
            net.settings()
                .most_players_text
                .write_at(0, &text, "a text line's bytes")?;
            crate::net::begin_edit::begin_edit(
                net,
                &net.settings().edit,
                item,
                EditedText::MostPlayers,
                editing::WIDTH,
                PALETTE,
            )?;
            net.settings()
                .most_players_text
                .write_at(0, &[0], "a text line's bytes")?;
            return Ok(());
        }
        Setting::None => return Ok(()),
    };
    crate::net::begin_edit::begin_edit(
        net,
        &net.settings().edit,
        item,
        text,
        editing::WIDTH,
        PALETTE,
    )
}
