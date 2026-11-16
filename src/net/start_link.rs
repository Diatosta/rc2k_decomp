//! A network game from the menu's host (1) and join (2) items: the player's settings and
//! record are set up, connections listed, the saved or a chosen connection made, then the session
//! hosted, or sessions listed and one joined (failures show messages 0x154 / 0x155). Then the
//! chat is cleared and the lobby and the race run in turn until the lobby is left; the network is
//! then stopped.
//!
//! Escape on the sessions' list shows message 0x155 as a failure does (kept as is).

use crate::errors::Refused;
use crate::net::lobby::log_bytes;
use crate::owners::Owners;
use crate::race::viewer_screen_store::texts;
use rc2k_core::net::players::State;

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

fn link(o: Owners<'static>, launched: bool) -> Result<Way, Refused> {
    let net = o.net;
    let lines = &crate::race::viewer_screen_store::NET_TEXTS;
    let vars = &net.vars;
    use crate::net::*;
    if !find_connections::find_connections(net, o.strings, lines)? {
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
        vars.own.set_state(vars.own.state().with(State::HOST));
        return Ok(if hosting::hosting(o, lines)? {
            Way::On
        } else {
            Way::Message(NO_SESSION)
        });
    }
    if !find_sessions::find_sessions(o, lines)? {
        return Ok(Way::Message(NO_SESSION));
    }
    let session = choose_session::choose_session(o, lines, launched)?;
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

/// A network game in `mode` (1 host, 2 join), as the menu's items start it.
pub fn start_link(o: crate::owners::Owners<'static>, mode: u32) -> Result<(), Refused> {
    run(o, mode, false)
}

/// A network game from the command line (`menu::launch`): as the menu's, with the session
/// joined as soon as one is listed, instead of waiting for a choice from the sessions' list.
pub fn start_link_launched(o: crate::owners::Owners<'static>, mode: u32) -> Result<(), Refused> {
    run(o, mode, true)
}

fn run(o: crate::owners::Owners<'static>, mode: u32, launched: bool) -> Result<(), Refused> {
    let (net, race) = (o.net, o.race);
    let vars = &net.vars;
    use crate::net::*;
    log_bytes(
        &crate::race::viewer_screen_store::NET_TEXTS.text(texts::NET_STARTING_LINK),
        true,
        2,
    );
    net.lobby().mode.set(mode);
    let menus = o.menus;
    take_player_settings::take_player_settings(race, &menus.play.slots);
    // The first car took the first player's controls.
    o.input.race_players[0].store(0, std::sync::atomic::Ordering::Relaxed);
    fill_own_player::fill_own_player(race, net, &menus.play.slots, &menus.options.snapshot())?;
    show_background::show_background(o)?;
    match link(o, launched)? {
        Way::Message(n) => show_message::show_message(o, n)?,
        Way::End => {}
        Way::On => {
            net.race_slot.begin();
            if mode == 2 {
                take_messages::joined(net);
            }
            {
                let _received = net.lock_received();
                clear_chat::clear_chat(vars);
                net.take_waiting_chat();
            }
            vars.receiver.sync.set(0);
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
