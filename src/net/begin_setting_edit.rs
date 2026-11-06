//! `net::begin_setting_edit` (0x416C26): Enter on a setting of the settings screen starts
//! its editing (`net::begin_edit`, width 0x30C, `palette`): item 1 the player's name
//! (`g_machineName`), item 3 the host's address (`unk_52ABE4`), item 2 the most players as
//! a text (`strings::format_u32` of `dword_52ABD0` into `byte_95E8F8`); any other nothing.
//!
//! In:  nothing (the item is `g_listChoice`)
//! Out: eax = the item, ebx = the text, ecx = 0x30C, edx = `palette` when an edit starts;
//!      for other items ebx = `dword_52ABD0`; no `pusha`
//! Caller: `net::settings_loop`.

use crate::net::begin_edit::EditedText;
use crate::net::lobby::{Cursor, PALETTE};
use crate::net::store::NET;
use rc2k_core::net::editing::{self, Setting};

/// The Rust version. The registers the original leaves when an edit starts (eax the item,
/// ebx the text, ecx the width, edx the font) are read by no caller (`settings_loop`).
pub fn begin_setting_edit() -> Result<(), crate::errors::Refused> {
    // Converted: `rc2k_core::net::editing` (`Setting`, `WIDTH`).
    let item = crate::install::owner::install().pages.choice.get();
    let text = match Setting::of(item) {
        Setting::Name => EditedText::MachineName,
        Setting::HostAddress => EditedText::HostAddress,
        Setting::MostPlayers => {
            let mut cursor = Cursor(crate::place::Place::from(
                &crate::install::owner::install().paths.path_a,
            ));
            cursor.bytes(&crate::strings::format_u32::format_u32(
                NET.most_players.get(),
                0,
            ))?;
            cursor.nul()?;
            EditedText::MostPlayers
        }
        Setting::None => return Ok(()),
    };
    crate::net::begin_edit::begin_edit(item, text, editing::WIDTH, PALETTE)
}
