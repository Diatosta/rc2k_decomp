//! `net::start_link` (0x416F50): a network game, from the menu's host (eax 1, action
//! 0x4AAAEC) and join (2, 0x4AAAE1) items (the original's third, a lobby's launch, is left
//! out with the lobbies: `net::open_lobby`):
//! "NET: Starting link", the mode to `dword_52A908`, the player's settings
//! (`net::take_player_settings`) and record (`net::fill_own_player` with `g_playerNames`),
//! the background; the connections listed (`net::find_connections_regs`; failing: message
//! 0x154). Then the saved connection (`net::find_saved_connection`)
//! or one chosen (`net::choose_connection`; none: the end), kept (`net::keep_connection`),
//! made (`net::connecting`; failing: 0x154); then as the host (mode 1, state bit 0x10) the
//! session made (`net::hosting`), else the sessions listed (`net::find_sessions_regs`), one
//! chosen (`net::choose_session`) and joined (`net::joining`; any of these failing: message
//! 0x155). Then the session on (`g_netSession` 1), the chat cleared (`net::clear_chat`, under
//! the received messages' lock), the sync counter 0, `g_netSequence` 0x80, not after a race;
//! and round: `dword_52AD78`, `dword_52AD7C` 0, the lobby's tune (`sound::play_tune_4`), the
//! lobby (`net::lobby_loop`; left: the end), new players kept out on the host
//! (`net::close_if_host`), the race (`net::lets_rally`; failing: the end). The end: "NET:
//! Stopping link", the network stopped (`net::stopping`), the menu's tune
//! (`sound::play_tune_3`).
//!
//! In:  the mode
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! Escape on the sessions' list gives the carry too, so it shows message 0x155 as a failure
//! does (section 16). Callers: 0x4AAAE1, 0x4AAAEC.

use crate::errors::Refused;
use crate::menu::names_store::NAME_BUFFERS;
use crate::net::lobby::log;
use crate::net::store::PlayerRecord;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const NO_CONNECTION: u32 = 0x154;
const NO_SESSION: u32 = 0x155;

/// How the link went.
enum Way {
    /// To the lobby.
    On,
    /// Message `n`, then the end.
    Message(u32),
    /// The end.
    End,
}

fn link() -> Result<Way, Refused> {
    use crate::net::*;
    if !find_connections::find_connections()? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    let connection = match find_saved_connection::find_saved_connection()? {
        Some(saved) => saved,
        None => {
            let chosen = choose_connection::choose_connection()?;
            if (chosen as i32) < 0 {
                return Ok(Way::End);
            }
            chosen
        }
    };
    keep_connection::keep_connection(connection)?;
    if !connecting::connecting(connection)? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    if NET.mode.get() == 1 {
        NET.own
            .head
            .set(PlayerRecord::STATE, NET.own.state() | 0x10);
        return Ok(if hosting::hosting()? {
            Way::On
        } else {
            Way::Message(NO_SESSION)
        });
    }
    if !find_sessions::find_sessions()? {
        return Ok(Way::Message(NO_SESSION));
    }
    let session = choose_session::choose_session()?;
    // Escape (a negative choice) sets the carry as a failure does, so it shows the
    // message too (the original's slip, `notes/bugs.md`).
    if (session as i32) < 0 {
        return Ok(Way::Message(NO_SESSION));
    }
    Ok(if joining::joining(session)? {
        Way::On
    } else {
        Way::Message(NO_SESSION)
    })
}

/// The Rust version: a network game in `mode` (1 host, 2 join).
pub fn start_link(mode: u32) -> Result<(), Refused> {
    use crate::net::*;
    log(VIEWER_PANEL.text_ptr(texts::NET_STARTING_LINK), true, 2);
    NET.mode.set(mode);
    take_player_settings::take_player_settings()?;
    fill_own_player::fill_own_player(&NAME_BUFFERS.player_names)?;
    show_background::show_background()?;
    match link()? {
        Way::Message(n) => show_message::show_message(n)?,
        Way::End => {}
        Way::On => {
            NET.session.set(1);
            lock_received::lock_received();
            clear_chat::clear_chat();
            unlock_received::unlock_received();
            NET.sync.set(0);
            NET.sequence.set(0x80);
            NET.after_race.set(0);
            loop {
                NET.all_raced.set(0);
                NET.race_over.set(0);
                crate::sound::play_tune_4::play_tune_4();
                if (lobby_loop::lobby_loop()? as i32) < 0 {
                    break;
                }
                close_if_host::close_if_host();
                if lets_rally::lets_rally()? {
                    break;
                }
            }
        }
    }
    log(VIEWER_PANEL.text_ptr(texts::NET_STOPPING_LINK), true, 2);
    stopping::stopping()?;
    crate::sound::play_tune_3::play_tune_3();
    Ok(())
}
