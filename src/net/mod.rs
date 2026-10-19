//! Network play (DirectPlay), the game's side: its set-up at the start and the receiving
//! thread (agent A's), the race's messages and the way out (agent B's) (read 2026-10-19; a reading, not seen in
//! play). `dword_52A910` is 1 while a network race runs, `dword_52A920` this machine's
//! player (its car's number), `dword_52AD98` the players. Each race step of the player's
//! own car (`net::race_step`, from `drive::replay_step`) sends its inputs ('JGSM', every
//! fourth step), its place and motion ('PGSM', step 2 of 32) and nine more dwords ('XGSM',
//! step 1 of 256), and applies what the other players sent; a remote car's steps glide
//! its place to the one received (`net::follow_remote`).
//! The messages: `net::send_message` (0x41736D) puts a sequence byte (`byte_52A915`,
//! counted, top bit set) and the bytes into `byte_52AA68` and sends them to everyone
//! (`dword_5AB274`) through DirectPlay (`net::dplay_send`). The receiving thread
//! (`net::receive_thread` 0x4183C3, "NETRXTDB", slot `stru_518714`, and its loop
//! `net::take_messages` 0x4194CC: IDA's names, agent A's) keeps each player's in
//! `byte_52AD9C` + player * 0x804C: +0x34 a count, from +0x38 records of 0x100 bytes
//! (+1 the length, +2 the bytes); `net::copy_received` copies that to +0x4040 under the
//! mutex `dword_52AB6C` for `net::apply_received`.
//! The way out: `net::shut_down` (the network ended, the connection destroyed, the two
//! handles closed).
//!
//! The calls by place (the callee's IDA address): in the game the Rust versions, the
//! logging and the Win32 calls, DirectPlay's methods from Rust (`net::dplay`); in the tests the noting
//! stand-ins of `sound::turn_parts`. Shadow mode runs the originals alone (messages are
//! sent, the mutex taken).

use crate::errors::{BadInput, Refused};
use crate::framework::{GameHook, RawHook, Regs, call_game, game_addr, rva, shadow};

pub mod add_address_element;
pub mod add_chat_line;
pub mod allocate_lists;
pub mod apply_message;
pub mod apply_received;
pub mod begin_chat;
pub mod begin_edit;
pub mod begin_setting_edit;
pub mod build_address;
pub mod car_list;
pub mod chat_input_line;
pub mod chat_key;
pub mod chat_line_locked;
pub mod choose_car;
pub mod choose_connection;
pub mod choose_session;
pub mod choose_stage;
pub mod clear_car_state;
pub mod clear_chat;
pub mod close_if_host;
pub mod close_session;
pub mod connecting;
pub mod connection_entry;
pub mod connection_found;
pub mod connections_frame;
pub mod copy_received;
pub mod create_dplay;
pub mod create_group;
pub mod create_player;
pub mod destroy_connection;
pub mod destroy_player;
pub mod disable_new_players;
pub mod dp_objects;
pub mod dp_session;
pub mod dplay;
pub mod dplay_send;
pub mod draw_button;
pub mod draw_centred_item;
pub mod draw_number_value;
pub mod draw_setting_value;
pub mod edit_key;
pub mod fill_own_player;
pub mod find_connections;
pub mod find_connections_regs;
pub mod find_saved_connection;
pub mod find_sessions;
pub mod find_sessions_regs;
pub mod follow_remote;
pub mod free_messages;
pub mod group_found;
pub mod host_session;
pub mod hosting;
pub mod init_connection;
pub mod join_group;
pub mod join_session;
pub mod joining;
pub mod keep_connection;
pub mod keep_hud_options;
pub mod lets_race;
pub mod lets_rally;
pub mod list_players;
pub mod list_players_regs;
pub mod lobby;
pub mod lobby_button;
pub mod lobby_calls;
pub mod lobby_frame;
pub mod lobby_hooks;
pub mod lobby_launched_game;
pub mod lobby_loop;
pub mod lock_received;
pub mod message_frame;
pub mod message_vars;
pub mod open_lobby;
pub mod player_found;
pub mod post_call;
pub mod race_over;
pub mod race_step;
pub mod race_wait;
pub mod receive_message;
pub mod receive_thread;
pub mod release_dplay;
pub mod release_lobby;
pub mod reset_cars;
pub mod send_chat;
pub mod send_chat_regs;
pub mod send_extra;
pub mod send_inputs;
pub mod send_message;
pub mod send_position;
pub mod send_race_message;
pub mod send_turn;
pub mod session_entry;
pub mod session_found;
pub mod sessions_frame;
pub mod set_hud_options;
pub mod set_up;
pub mod set_up_race;
pub mod setting_key;
pub mod setting_label;
pub mod settings_done;
pub mod settings_frame;
pub mod settings_link;
pub mod settings_loop;
pub mod show_background;
pub mod show_message;
pub mod shut_down;
pub mod sort_players;
pub mod start_link;
pub mod stop;
pub mod stopping;
pub mod store_received;
pub mod system_message;
pub mod take_messages;
pub mod take_player_settings;
pub mod types;
pub mod unlock_received;

pub static HOOKS: &[&dyn GameHook] = &[
    &set_up::HOOK,
    &receive_thread::HOOK,
    &send_message::HOOK,
    &send_race_message::HOOK,
    &send_inputs::HOOK,
    &send_position::HOOK,
    &send_extra::HOOK,
    &send_turn::HOOK,
    &copy_received::HOOK,
    &apply_message::HOOK,
    &apply_received::HOOK,
    &follow_remote::HOOK,
    &race_step::HOOK,
    &free_messages::HOOK,
    &destroy_connection::HOOK,
    &stop::HOOK,
    &shut_down::HOOK,
    &connection_entry::HOOK,
    &session_entry::HOOK,
    &allocate_lists::HOOK,
    &add_address_element::HOOK,
    &begin_edit::HOOK,
    &edit_key::HOOK,
    &clear_chat::HOOK,
    &add_chat_line::HOOK,
    &chat_line_locked::HOOK,
    &send_chat::HOOK,
    &send_chat_regs::HOOK,
    &sort_players::HOOK,
    &fill_own_player::HOOK,
    &take_player_settings::HOOK,
    &clear_car_state::HOOK,
    &reset_cars::HOOK,
    &set_up_race::HOOK,
    &set_hud_options::HOOK,
    &keep_hud_options::HOOK,
    &lets_race::HOOK,
    &car_list::HOOK,
    &find_saved_connection::HOOK,
    &keep_connection::HOOK,
    &find_connections_regs::HOOK,
    &find_sessions_regs::HOOK,
    &list_players_regs::HOOK,
    &close_if_host::HOOK,
    &create_dplay::HOOK,
    &open_lobby::HOOK,
    &connection_found::HOOK,
    &find_connections::HOOK,
    &build_address::HOOK,
    &init_connection::HOOK,
    &post_call::HOOK,
    &find_sessions::HOOK,
    &session_found::HOOK,
    &create_player::HOOK,
    &create_group::HOOK,
    &join_group::HOOK,
    &group_found::HOOK,
    &player_found::HOOK,
    &list_players::HOOK,
    &host_session::HOOK,
    &join_session::HOOK,
    &disable_new_players::HOOK,
    &show_background::HOOK,
    &message_frame::HOOK,
    &show_message::HOOK,
    &connecting::HOOK,
    &hosting::HOOK,
    &joining::HOOK,
    &stopping::HOOK,
    &draw_button::HOOK,
    &setting_label::HOOK,
    &draw_centred_item::HOOK,
    &draw_setting_value::HOOK,
    &draw_number_value::HOOK,
    &settings_frame::HOOK,
    &begin_setting_edit::HOOK,
    &setting_key::HOOK,
    &settings_link::HOOK,
    &settings_done::HOOK,
    &settings_loop::HOOK,
    &connections_frame::HOOK,
    &sessions_frame::HOOK,
    &choose_connection::HOOK,
    &choose_session::HOOK,
    &lobby_button::HOOK,
    &chat_input_line::HOOK,
    &begin_chat::HOOK,
    &chat_key::HOOK,
    &lobby_frame::HOOK,
    &choose_car::HOOK,
    &choose_stage::HOOK,
    &lobby_loop::HOOK,
    &race_wait::HOOK,
    &race_over::HOOK,
    &lets_rally::HOOK,
    &start_link::HOOK,
    &lobby_launched_game::HOOK,
    &lock_received::HOOK,
    &unlock_received::HOOK,
    &dplay_send::HOOK,
    &destroy_player::HOOK,
    &close_session::HOOK,
    &release_dplay::HOOK,
    &release_lobby::HOOK,
    &store_received::HOOK,
    &receive_message::HOOK,
    &system_message::HOOK,
    &take_messages::HOOK,
];

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// dword_52A910: 1 while a network race runs.
pub const ON: usize = 0x52A910;
/// dword_52A920: this machine's player.
pub const PLAYER: usize = 0x52A920;
/// dword_52AD98: the players.
pub const PLAYERS: usize = 0x52AD98;
/// byte_52AD9C: each player's received messages, 0x804C bytes a player.
pub const RECEIVED: usize = 0x52AD9C;
pub const RECEIVED_LEN: u32 = std::mem::size_of::<types::NetPlayerBlock>() as u32;
/// dword_52AB6C: the mutex of the received messages; dword_52AB68: the receiving
/// thread's event.
pub const MUTEX: usize = 0x52AB6C;
pub const EVENT: usize = 0x52AB68;
/// dword_63DBA0: logged with each message (a step count, it seems).
pub const STEP: usize = 0x63DBA0;
/// unk_71C4D0: the message being built.
pub const MESSAGE: usize = 0x71C4D0;
/// The current car's block (`g_carBlocks` 0x71B3B8 point at each car's; the step works
/// on this copy): +8 its steps.
pub const CAR: usize = 0x719FD8;

// The places.
pub const LOG_TEXT: u32 = 0x401427; // logfile::log_write_l1
pub const LOG_INT: u32 = 0x401472; // logfile::log_int_l1
pub const LOG_INT_NL: u32 = 0x401476; // logfile::log_int_l1_nl
pub const LOG_TEXT_2_NL: u32 = 0x401439; // logfile::log_write_l2_nl
pub const SEND: u32 = 0x4196FF; // net::dplay_send (DirectPlay's Send)
pub const SEND_MESSAGE: u32 = 0x41736D; // net::send_message
pub const SEND_RACE: u32 = 0x46449E; // net::send_race_message
pub const SEND_INPUTS: u32 = 0x464194; // net::send_inputs
pub const SEND_POSITION: u32 = 0x464201; // net::send_position
pub const SEND_EXTRA: u32 = 0x4643FA; // net::send_extra
pub const SEND_TURN: u32 = 0x46415B; // net::send_turn
pub const APPLY_RECEIVED: u32 = 0x4644A4; // net::apply_received
pub const COPY_RECEIVED: u32 = 0x4173A0; // net::copy_received
pub const APPLY: u32 = 0x4644EC; // net::apply_message
pub const FOLLOW: u32 = 0x464883; // net::follow_remote
pub const LOCK: u32 = 0x418474; // net::lock_received: WaitForSingleObject on the mutex
pub const UNLOCK: u32 = 0x418481; // net::unlock_received: ReleaseMutex
pub const SLEEP_1MS: u32 = 0x410FFE; // sleep_1ms
pub const PUMP: u32 = 0x410535; // system::pump_messages
pub const STOP: u32 = 0x41838C; // net::stop
pub const DESTROY: u32 = 0x4187EE; // net::destroy_connection
pub const FREE: u32 = 0x4185BB; // net::free_messages
pub const FREE_LIST: u32 = 0x4028F5; // memory::free_list
pub const DESTROY_PLAYER: u32 = 0x41916B; // net::destroy_player (DirectPlay's DestroyPlayer)
pub const CLOSE_SESSION: u32 = 0x418DD3; // net::close_session (DirectPlay's Close)
pub const RELEASE_DPLAY: u32 = 0x41864E; // net::release_dplay (DirectPlay's Release)
pub const RELEASE_LOBBY: u32 = 0x41900F; // net::release_lobby (the lobby's Release)
pub const CLOSE_EVENT: u32 = 0x41104F; // CloseHandle (eax, if not 0)
pub const CLOSE_MUTEX: u32 = 0x411092; // CloseHandle (eax, if not 0)
pub const WAIT_OBJECT: u32 = 0x4110A6; // system::wait_object: WaitForSingleObject(eax, INFINITE)
pub const RELEASE_MUTEX: u32 = 0x4110B9; // system::release_mutex: ReleaseMutex(eax)
pub const TICK: u32 = 0x411024; // system::tick_count
pub const CHAT_LINE: u32 = 0x4193FF; // net::chat_line_locked
pub const TAKE_MESSAGES: u32 = 0x4194CC; // net::take_messages
pub const STORE: u32 = 0x419677; // net::store_received
pub const RECEIVE: u32 = 0x419722; // net::receive_message (DirectPlay's Receive)
pub const SYSTEM_MESSAGE: u32 = 0x41976F; // net::system_message

pub fn g(x: usize) -> u32 {
    unsafe { crate::framework::game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { crate::framework::game_write(rva(x), v) }
}

/// The address of a game variable.
pub fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::{log_int::log_int, log_write::log_write};
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{ReleaseMutex, Sleep, WaitForSingleObject};
    unsafe {
        match place {
            LOG_TEXT => log_write(r.eax as *const u8, false, 1),
            LOG_INT => log_int(r.eax as i32, false, 1),
            LOG_INT_NL => log_int(r.eax as i32, true, 1),
            LOG_TEXT_2_NL => log_write(r.eax as *const u8, true, 2),
            SEND_MESSAGE => send_message::send_message(r, &mut game_part)?,
            SEND_RACE => send_race_message::send_race_message(r, &mut game_part)?,
            SEND_INPUTS => send_inputs::send_inputs(r, &mut game_part)?,
            SEND_POSITION => send_position::send_position(r, &mut game_part)?,
            SEND_EXTRA => send_extra::send_extra(r, &mut game_part)?,
            SEND_TURN => send_turn::send_turn(r, &mut game_part)?,
            APPLY_RECEIVED => apply_received::apply_received(r, &mut game_part)?,
            COPY_RECEIVED => copy_received::copy_received(r, &mut game_part)?,
            APPLY => apply_message::apply_message(r, &mut game_part)?,
            FOLLOW => follow_remote::follow_remote(r, &mut game_part)?,
            STOP => stop::stop(r, &mut game_part)?,
            DESTROY => destroy_connection::destroy_connection(r, &mut game_part)?,
            FREE => free_messages::free_messages(r, &mut game_part)?,
            FREE_LIST => {
                crate::memory::free_list::free_list(r.ebx as *mut _)?;
                r.set_carry(false);
            }
            LOCK => lock_received::lock_received(r, &mut game_part)?,
            UNLOCK => unlock_received::unlock_received(r, &mut game_part)?,
            // `system::wait_object` and `system::release_mutex` (not hooked): every register
            // kept, the flags of `cmp result + 1, 1` and `cmp result, 1`.
            WAIT_OBJECT => {
                let result = WaitForSingleObject(r.eax as _, u32::MAX);
                crate::net::lobby::cmp_flags(r, result.wrapping_add(1), 1);
            }
            RELEASE_MUTEX => {
                let result = ReleaseMutex(r.eax as _);
                crate::net::lobby::cmp_flags(r, result as u32, 1);
            }
            TICK => r.eax = crate::system::tick_count::tick_count(),
            STORE => crate::race::loadlevel::or_in_game(STORE as usize, r, |r| {
                store_received::store_received(r, &mut game_part)
            })?,
            RECEIVE => receive_message::receive_message(r, &mut game_part)?,
            SYSTEM_MESSAGE => system_message::system_message(r, &mut game_part)?,
            CHAT_LINE => crate::race::loadlevel::or_in_game(CHAT_LINE as usize, r, |r| {
                *r = chat_line_locked::chat_line_locked(r, &mut crate::net::lobby::game_part)?;
                Ok(())
            })?,
            CLOSE_EVENT | CLOSE_MUTEX => {
                if r.eax != 0 {
                    CloseHandle(r.eax as _);
                }
            }
            SLEEP_1MS => Sleep(1),
            // The window's messages: the game's (the platform's in the standalone).
            PUMP => call_game(r, game_addr(rva(PUMP as usize))),
            // DirectPlay's methods from Rust (`net::dplay`).
            SEND => dplay_send::dplay_send(r, &mut game_part)?,
            DESTROY_PLAYER => destroy_player::destroy_player(r, &mut game_part)?,
            CLOSE_SESSION => close_session::close_session(r, &mut game_part)?,
            RELEASE_DPLAY => release_dplay::release_dplay(r, &mut game_part)?,
            RELEASE_LOBBY => release_lobby::release_lobby(r, &mut game_part)?,
            TAKE_MESSAGES => take_messages::take_messages(r, &mut game_part)?,
            _ => return Err(BadInput("a network call with no version").into()),
        }
    }
    Ok(())
}

/// The hooks' dispatch: shadow mode runs the original alone; out of it the Rust version
/// with the game's calls, the registers it leaves (the flags kept unless `flags`).
pub fn dispatch_net(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    match rust(&mut g, &mut game_part) {
        Ok(()) => *r = Regs { esp: r.esp, ..g },
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// Logs as the send functions do: "send x", the player, the step (`log_int_l1_nl`),
/// each inside `pusha` .. `popa`.
pub fn log_send(text: usize, r: &Regs, part: Part) -> Result<(), Refused> {
    let mut l = Regs { eax: a(text), ..*r };
    part(LOG_TEXT, &mut l)?;
    l.eax = g(PLAYER);
    part(LOG_INT, &mut l)?;
    l.eax = g(STEP);
    part(LOG_INT_NL, &mut l)
}
