//! What a player's data that cannot be understood does: the message is dropped, the player is
//! told once (the notice waits for a menu frame when the receiving thread raised it), and the
//! session goes on.

/// A message of `player` (`u32::MAX`: not known) that was dropped for `why`.
pub fn dropped_message(player: u32, why: &str) {
    let who = if player == u32::MAX {
        "?".to_string()
    } else {
        (player + 1).to_string()
    };
    crate::logfile::program_log::log(&format!(
        "net: a message of player {who} cannot be used and is dropped: {why}"
    ));
    crate::error_dialog::notice_own_later(
        &format!("net:{who}:{why}"),
        rc2k_core::strings::own::id::BAD_PEER,
        &who,
    );
}
