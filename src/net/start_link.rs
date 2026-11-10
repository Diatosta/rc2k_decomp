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
//! Takes the mode.
//! Escape on the sessions' list gives the carry too, so it shows message 0x155 as a failure
//! does (section 16). Callers: 0x4AAAE1, 0x4AAAEC.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::net::lobby::log;
use crate::net::owner::Net;
use crate::net::owner::net;
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

fn link(install: &'static crate::install::owner::Install, net: &Net) -> Result<Way, Refused> {
    let vars = &net.vars;
    use crate::net::*;
    if !find_connections::find_connections(&net.vars)? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    let connection = match find_saved_connection::find_saved_connection()? {
        Some(saved) => saved,
        None => {
            let chosen = choose_connection::choose_connection(install, &net.vars)?;
            if (chosen as i32) < 0 {
                return Ok(Way::End);
            }
            chosen
        }
    };
    keep_connection::keep_connection(connection)?;
    if !connecting::connecting(net, connection)? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    if vars.mode.get() == 1 {
        vars.own
            .head
            .set(PlayerRecord::STATE, vars.own.state() | 0x10);
        return Ok(if hosting::hosting(net)? {
            Way::On
        } else {
            Way::Message(NO_SESSION)
        });
    }
    if !find_sessions::find_sessions(net)? {
        return Ok(Way::Message(NO_SESSION));
    }
    let session = choose_session::choose_session(install, net)?;
    // Escape (a negative choice) sets the carry as a failure does, so it shows the
    // message too (the original's slip, `notes/bugs.md`).
    if (session as i32) < 0 {
        return Ok(Way::Message(NO_SESSION));
    }
    Ok(if joining::joining(net, session)? {
        Way::On
    } else {
        Way::Message(NO_SESSION)
    })
}

/// A network game in `mode` (1 host, 2 join).
pub fn start_link(mode: u32) -> Result<(), Refused> {
    let install = crate::install::owner::install();
    let net = net();
    let race = crate::race::owner::race();
    let vars = &net.vars;
    use crate::net::*;
    log(VIEWER_PANEL.text_ptr(texts::NET_STARTING_LINK), true, 2);
    vars.mode.set(mode);
    take_player_settings::take_player_settings(race);
    let play = &menus().play;
    fill_own_player::fill_own_player(race, net, play, &play.names.player_names.read())?;
    show_background::show_background(&net.vars)?;
    match link(install, net)? {
        Way::Message(n) => show_message::show_message(n)?,
        Way::End => {}
        Way::On => {
            vars.session.set(1);
            if mode == 2 {
                take_messages::joined();
            }
            lock_received::lock_received(&net.vars);
            clear_chat::clear_chat();
            unlock_received::unlock_received(&net.vars);
            vars.sync.set(0);
            vars.sequence.set(0x80);
            vars.after_race.set(0);
            loop {
                vars.all_raced.set(0);
                vars.race_over.set(0);
                crate::sound::play_tune_4::play_tune_4();
                if (lobby_loop::lobby_loop(install, race, net)? as i32) < 0 {
                    break;
                }
                close_if_host::close_if_host(net);
                if lets_rally::lets_rally(race, net)? {
                    break;
                }
            }
        }
    }
    log(VIEWER_PANEL.text_ptr(texts::NET_STOPPING_LINK), true, 2);
    stopping::stopping(&net.vars)?;
    crate::sound::play_tune_3::play_tune_3();
    Ok(())
}
