//! What a player's data that cannot be understood does: the message is dropped, the player is
//! told once (the notice waits for a menu frame when the receiving thread raised it), and the
//! session goes on. The log gets a line for each player and reason once in a while, with the
//! count of those left out, so that a peer sending nothing else cannot fill it.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use rc2k_core::net::rate::LineLimit;

/// How long a line of one kind is not repeated.
const LOG_PERIOD: Duration = Duration::from_secs(10);

/// The time the lines' periods are counted from.
static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
static LIMIT: Mutex<Option<LineLimit>> = Mutex::new(None);

/// A message of `player` (`u32::MAX`: not known) that was dropped for `why`.
pub(super) fn dropped_message(player: u32, why: &str) {
    let who = if player == u32::MAX {
        "?".to_string()
    } else {
        (player + 1).to_string()
    };
    let due = LIMIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_or_insert_with(LineLimit::default)
        .due(
            &format!("{who}:{why}"),
            START.get_or_init(Instant::now).elapsed(),
            LOG_PERIOD,
        );
    if let Some(left_out) = due {
        let more = if left_out == 0 {
            String::new()
        } else {
            format!(" ({left_out} more since the last line)")
        };
        crate::program_log::log(&format!(
            "net: a message of player {who} cannot be used and is dropped: {why}{more}"
        ));
    }
    crate::error_dialog::notice_own_later(
        &format!("net:{who}:{why}"),
        rc2k_core::strings::own::id::BAD_PEER,
        &who,
    );
}
