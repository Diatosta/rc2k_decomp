//! Network play (DirectPlay), the game's side: its set-up at the start, the receiving
//! thread, the race's messages and the way out (a reading, not seen in play). `session` is
//! 1 while a network race runs, `player` is this machine's player (its car's number),
//! `players` the players. Each race step of the player's
//! own car (`net::race_step`, from `drive::replay_step`) sends its inputs ('JGSM', every
//! fourth step), its place and motion ('PGSM', step 2 of 32) and nine more dwords ('XGSM',
//! step 1 of 256), and applies what the other players sent; a remote car's steps glide
//! its place to the one received (`net::follow_remote`).
//! The messages: `net::send_message` puts a sequence byte (`sequence`, counted, top bit
//! set) and the bytes into `send_buffer` and sends them to everyone (`send_to`) through
//! DirectPlay (`net::dplay_send`). The receiving thread (`net::receive_thread`, "NETRXTDB";
//! its loop is `net::take_messages`) keeps each player's in `received`, a block of
//! 0x804C bytes a player: +0x34 a count, from +0x38 records of 0x100 bytes
//! (+1 the length, +2 the bytes); `net::copy_received` copies that to +0x4040 under the
//! mutex (`mutex`) for `net::apply_received`.
//! The way out: `net::shut_down` (the network ended, the connection destroyed, the two
//! handles closed).
//!
//! The race's messages, the receiving thread, the way out and the DirectPlay set-up are plain
//! functions calling each other directly; the screens' loops and frames of the lobby too (no
//! registers, no calls by place). DirectPlay's methods from Rust (`net::dplay`).

use crate::net::store::NetShared;
use crate::race::owner::Race;

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
pub mod find_saved_connection;
pub mod find_sessions;
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
pub mod lobby;
pub mod lobby_button;
pub mod lobby_frame;
pub mod lobby_loop;
pub mod lock_received;
pub mod message_frame;
pub mod open_lobby;
pub mod owner;
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
pub mod send_extra;
pub mod send_inputs;
pub mod send_message;
pub mod send_position;
pub mod send_turn;
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
pub mod store;
pub mod store_received;
pub mod system_message;
pub mod tables;
pub mod take_messages;
pub mod take_player_settings;
pub mod types;
pub mod unlock_received;

/// Logs as the send functions do: "send x", the player, the step (`log_int_l1_nl`).
pub fn log_send(race: &Race, vars: &NetShared, text: &std::ffi::CStr) {
    use crate::logfile::log_int::log_int;
    crate::logfile::log_write::log_nul(&text, false, 1);
    log_int(vars.player.get() as i32, false, 1);
    log_int(race.shared.view.get().local_car.get() as i32, true, 1);
}
