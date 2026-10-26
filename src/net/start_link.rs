//! `net::start_link` (0x416F50): a network game, from the menu's host (eax 1, action
//! 0x4AAAEC) and join (2, 0x4AAAE1) items or a lobby's launch (3, `net::lobby_launched_game`):
//! "NET: Starting link", the mode to `dword_52A908`, the player's settings
//! (`net::take_player_settings`) and record (`net::fill_own_player` with `g_playerNames`),
//! the background; the connections listed (`net::find_connections_regs`; failing: message
//! 0x154). Unless a lobby launched the game: the saved connection (`net::find_saved_connection`)
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
//! In:  eax = the mode
//! Out: every register kept (`pusha` .. `popa`); the flags the menu's tune leaves
//! Escape on the sessions' list gives the carry too, so it shows message 0x155 as a failure
//! does (section 16). Callers: 0x4AAAE1, 0x4AAAEC, `net::lobby_launched_game`. Its calls
//! through `part`. The whole network game runs inside it: ungated. Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (each failure scripted, the
//! lobby left at the second round), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    ALL_RACED, BACK_FROM_RACE, LOBBY_LAUNCHED, LOCK_RECEIVED, LOG_L2_NL, MODE, OWN, Part,
    RACE_OVER, SEQUENCE, SESSION, SYNC, TUNE_3, TUNE_4, UNLOCK_RECEIVED, a, g, gb, lobby_entry, p,
    pb,
};

const STARTING: usize = 0x5C2E04; // "NET: Starting link"
const STOPPING: usize = 0x5C2E17; // "NET: Stopping link"
const PLAYER_NAMES: usize = 0x755784; // g_playerNames
const TAKE_SETTINGS: u32 = 0x48755D; // net::take_player_settings
const FILL_OWN: u32 = 0x4181A3; // net::fill_own_player
const SHOW_BACKGROUND: u32 = 0x4173E7; // net::show_background
const FIND_CONNECTIONS: u32 = 0x4174BD; // net::find_connections_regs
const FIND_SAVED: u32 = 0x4185DF; // net::find_saved_connection
const CHOOSE_CONNECTION: u32 = 0x4174C5; // net::choose_connection
const KEEP_CONNECTION: u32 = 0x417661; // net::keep_connection
const CONNECTING: u32 = 0x417688; // net::connecting
const HOSTING: u32 = 0x4176D5; // net::hosting
const FIND_SESSIONS: u32 = 0x41771E; // net::find_sessions_regs
const CHOOSE_SESSION: u32 = 0x417726; // net::choose_session
const JOINING: u32 = 0x4179B6; // net::joining
const CLEAR_CHAT: u32 = 0x419411; // net::clear_chat
const LOBBY_LOOP: u32 = 0x417A0D; // net::lobby_loop
const CLOSE_IF_HOST: u32 = 0x41837B; // net::close_if_host
const LETS_RALLY: u32 = 0x4170E0; // net::lets_rally
const STOPPING_SCREEN: u32 = 0x417089; // net::stopping
const SHOW_MESSAGE: u32 = 0x41745C; // net::show_message
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

fn link(r: &mut Regs, part: Part) -> Result<Way, Refused> {
    part(FIND_CONNECTIONS, r)?;
    if r.carry() {
        return Ok(Way::Message(NO_CONNECTION));
    }
    if g(LOBBY_LAUNCHED) == 1 {
        return Ok(Way::On);
    }
    part(FIND_SAVED, r)?;
    if r.carry() {
        part(CHOOSE_CONNECTION, r)?;
        if r.carry() {
            return Ok(Way::End);
        }
    }
    part(KEEP_CONNECTION, r)?;
    part(CONNECTING, r)?;
    if r.carry() {
        return Ok(Way::Message(NO_CONNECTION));
    }
    if g(MODE) == 1 {
        pb(OWN, gb(OWN) | 0x10);
        part(HOSTING, r)?;
        return Ok(if r.carry() {
            Way::Message(NO_SESSION)
        } else {
            Way::On
        });
    }
    part(FIND_SESSIONS, r)?;
    if r.carry() {
        return Ok(Way::Message(NO_SESSION));
    }
    part(CHOOSE_SESSION, r)?;
    if r.carry() {
        return Ok(Way::Message(NO_SESSION));
    }
    if (r.eax as i32) < 0 {
        return Ok(Way::End);
    }
    part(JOINING, r)?;
    Ok(if r.carry() {
        Way::Message(NO_SESSION)
    } else {
        Way::On
    })
}

/// The Rust version: the registers it leaves (the entry's).
pub fn start_link(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    crate::net::lobby::log_text(LOG_L2_NL, a(STARTING), &r, part)?;
    p(MODE, r.eax);
    part(TAKE_SETTINGS, &mut r)?;
    r.eax = a(PLAYER_NAMES);
    part(FILL_OWN, &mut r)?;
    part(SHOW_BACKGROUND, &mut r)?;
    match link(&mut r, part)? {
        Way::Message(n) => {
            r.eax = n;
            part(SHOW_MESSAGE, &mut r)?;
        }
        Way::End => {}
        Way::On => {
            p(SESSION, 1);
            part(LOCK_RECEIVED, &mut r)?;
            part(CLEAR_CHAT, &mut r)?;
            part(UNLOCK_RECEIVED, &mut r)?;
            pb(SYNC, 0);
            pb(SEQUENCE, 0x80);
            p(BACK_FROM_RACE, 0);
            loop {
                p(ALL_RACED, 0);
                p(RACE_OVER, 0);
                part(TUNE_4, &mut r)?;
                part(LOBBY_LOOP, &mut r)?;
                if r.carry() || (r.eax as i32) < 0 {
                    break;
                }
                part(CLOSE_IF_HOST, &mut r)?;
                part(LETS_RALLY, &mut r)?;
                if r.carry() {
                    break;
                }
            }
        }
    }
    r.set_carry(true);
    crate::net::lobby::log_text(LOG_L2_NL, a(STOPPING), &r, part)?;
    part(STOPPING_SCREEN, &mut r)?;
    part(TUNE_3, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, start_link)
}
