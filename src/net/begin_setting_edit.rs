//! `net::begin_setting_edit` (0x416C26): Enter on a setting of the settings screen starts
//! its editing (`net::begin_edit`, width 0x30C, `palette`): item 1 the player's name
//! (`g_machineName`), item 3 the host's address (`unk_52ABE4`), item 2 the most players as
//! a text (`strings::format_u32` of `dword_52ABD0` into `byte_95E8F8`); any other nothing.
//!
//! Takes nothing (the item is `g_listChoice`).
//! Caller: `net::settings_loop`.

use crate::net::begin_edit::EditedText;
use crate::net::lobby::PALETTE;
use crate::net::store::NetVars;
use rc2k_core::net::editing::{self, Setting};

/// Starts a setting's edit (item, text, width and font as the original passes them).
pub fn begin_setting_edit(
    install: &'static crate::install::owner::Install,
    vars: &NetVars,
) -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::net::editing` (`Setting`, `WIDTH`).
    let item = install.pages.choice.get();
    let text = match Setting::of(item) {
        Setting::Name => EditedText::MachineName,
        Setting::HostAddress => EditedText::HostAddress,
        Setting::MostPlayers => {
            let mut digits = crate::strings::format_u32::format_u32(vars.most_players.get(), 0);
            digits.push(0);
            install
                .paths
                .path_a
                .write_at(0, &digits, "a text line's bytes")?;
            EditedText::MostPlayers
        }
        Setting::None => return Ok(()),
    };
    crate::net::begin_edit::begin_edit(vars, item, text, editing::WIDTH, PALETTE)
}
