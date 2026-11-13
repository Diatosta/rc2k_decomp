//! A network game from the menu's host (1) and join (2) items: the player's settings and
//! record are set up, connections listed, the saved or a chosen connection made, then the session
//! hosted, or sessions listed and one joined (failures show messages 0x154 / 0x155). Then the
//! chat is cleared and the lobby and the race run in turn until the lobby is left; the network is
//! then stopped.
//!
//! Escape on the sessions' list shows message 0x155 as a failure does (kept as is).

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::net::store::PlayerRecord;
use crate::owners::Owners;
use crate::race::viewer_screen_store::texts;

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

fn link(o: Owners<'static>) -> Result<Way, Refused> {
    let net = o.net;
    let lines = &crate::race::viewer_screen_store::NET_TEXTS;
    let vars = &net.vars;
    use crate::net::*;
    if !find_connections::find_connections(&net.vars, o.strings, lines)? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    let connection = match find_saved_connection::find_saved_connection(net)? {
        Some(saved) => saved,
        None => {
            let chosen = choose_connection::choose_connection(o)?;
            if (chosen as i32) < 0 {
                return Ok(Way::End);
            }
            chosen
        }
    };
    keep_connection::keep_connection(net, connection)?;
    if !connecting::connecting(o, lines, connection)? {
        return Ok(Way::Message(NO_CONNECTION));
    }
    if net.lobby().mode.get() == 1 {
        vars.own
            .head
            .set(PlayerRecord::STATE, vars.own.state() | 0x10);
        return Ok(if hosting::hosting(o, lines)? {
            Way::On
        } else {
            Way::Message(NO_SESSION)
        });
    }
    if !find_sessions::find_sessions(o, lines)? {
        return Ok(Way::Message(NO_SESSION));
    }
    let session = choose_session::choose_session(o, lines)?;
    // Escape (a negative choice) is taken as a failure, so it shows the message too (kept as
    // is).
    if (session as i32) < 0 {
        return Ok(Way::Message(NO_SESSION));
    }
    Ok(if joining::joining(o, lines, session)? {
        Way::On
    } else {
        Way::Message(NO_SESSION)
    })
}

/// A network game in `mode` (1 host, 2 join).
pub fn start_link(o: crate::owners::Owners<'static>, mode: u32) -> Result<(), Refused> {
    let (net, race) = (o.net, o.race);
    let vars = &net.vars;
    use crate::net::*;
    log_bytes(
        &crate::race::viewer_screen_store::NET_TEXTS.text(texts::NET_STARTING_LINK),
        true,
        2,
    );
    net.lobby().mode.set(mode);
    take_player_settings::take_player_settings(race, o.menus);
    let menus = o.menus;
    fill_own_player::fill_own_player(race, net, menus, &menus.play.names.player_names.read())?;
    show_background::show_background(o)?;
    match link(o)? {
        Way::Message(n) => show_message::show_message(o, n)?,
        Way::End => {}
        Way::On => {
            vars.session.set(1);
            if mode == 2 {
                take_messages::joined();
            }
            lock_received::lock_received(&net.vars);
            clear_chat::clear_chat(vars);
            unlock_received::unlock_received(&net.vars);
            vars.sync.set(0);
            net.lobby().sequence.set(0x80);
            net.lobby().after_race.set(0);
            loop {
                vars.all_raced.set(0);
                vars.race_over.set(0);
                crate::sound::play_tune_4::play_tune_4(o);
                if (lobby_loop::lobby_loop(o)? as i32) < 0 {
                    break;
                }
                close_if_host::close_if_host(net, &crate::race::viewer_screen_store::NET_TEXTS);
                if lets_rally::lets_rally(o)? {
                    break;
                }
            }
        }
    }
    log_bytes(
        &crate::race::viewer_screen_store::NET_TEXTS.text(texts::NET_STOPPING_LINK),
        true,
        2,
    );
    stopping::stopping(o, &crate::race::viewer_screen_store::NET_TEXTS)?;
    crate::sound::play_tune_3::play_tune_3(o);
    Ok(())
}
