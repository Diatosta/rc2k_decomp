//! Network play. The game's side (the lobby screens, set-up at the start, the receiving thread,
//! the race's messages, the host's computer drivers (`net::bots`) and the way out) sits on the project's own session layer (`dp_session`),
//! which stands where DirectPlay stood: its calls keep DirectPlay's shape (`dp_objects`).
//!
//! The network's `NetRaceSlot` (`Net::race_slot`) says whether a link is open, this machine's player (its car's number) and the players' count. Each race step
//! of the player's own car (`net::race_step`) sends its inputs ('JGSM', every fourth step), its place and motion
//! ('PGSM', step 2 of 32) and nine more dwords ('XGSM', step 1 of 256), and applies what the
//! other players sent; a remote car's steps glide its place to the one received.
//!
//! `net::send_message` puts a sequence byte (top bit set) and the bytes into `send_buffer` and
//! sends them to everyone through the session layer. The receiving thread (`net::receive_thread`) keeps
//! each player's messages in its inbox (`Net::inboxes`); `net::copy_received` copies them under
//! the lock for `net::apply_received`. `net::shut_down` ends the network at the game's end.

use crate::race::owner::Race;

mod add_address_element;
mod add_chat_line;
mod allocate_lists;
mod apply_message;
mod apply_received;
mod bad_peer;
mod begin_chat;
mod begin_edit;
mod begin_setting_edit;
mod bonjour;
pub(crate) mod bots;
mod build_address;
mod car_list;
mod chat_input_line;
mod chat_key;
mod chat_line_locked;
mod choose_car;
mod choose_connection;
mod choose_session;
mod choose_stage;
pub(crate) mod clear_car_state;
mod clear_chat;
mod close_if_host;
mod close_session;
mod connecting;
mod connection_found;
mod connections_frame;
mod copy_received;
mod create_dplay;
mod create_group;
mod create_player;
mod destroy_connection;
mod destroy_player;
mod disable_new_players;
mod dp_objects;
mod dp_session;
mod dplay;
mod dplay_send;
mod drain_chat;
mod draw_button;
mod draw_centred_item;
mod draw_number_value;
mod draw_setting_value;
mod edit_key;
mod fill_own_player;
mod find_connections;
mod find_saved_connection;
mod find_sessions;
mod follow_remote;
mod free_messages;
#[cfg(test)]
mod frozen_formats;
mod group_found;
mod host_session;
mod hosting;
mod init_connection;
pub(crate) mod invite;
mod join_group;
mod join_session;
mod joining;
mod keep_connection;
mod keep_hud_options;
mod kick_command;
pub(crate) mod last_host;
pub(crate) mod left;
mod lets_race;
mod lets_rally;
mod link;
mod list_players;
pub(crate) mod loading;
mod lobby;
mod lobby_button;
mod lobby_frame;
mod lobby_loop;
mod message_frame;
mod open_lobby;
pub(crate) mod owner;
pub(crate) mod password;
mod password_prompt;
mod player_found;
mod post_call;
mod progress;
mod race_over;
pub(crate) mod race_step;
mod race_wait;
mod receive_message;
pub(crate) mod receive_thread;
mod relay_chat;
mod release_dplay;
mod release_lobby;
mod reset_cars;
mod send_bot_turn;
mod send_chat;
mod send_extra;
mod send_inputs;
mod send_message;
mod send_position;
mod send_turn;
mod session_found;
mod sessions_frame;
mod set_hud_options;
pub(crate) mod set_up;
mod set_up_race;
mod setting_key;
mod setting_label;
mod settings_done;
mod settings_frame;
mod settings_link;
pub(crate) mod settings_loop;
mod show_background;
mod show_message;
pub(crate) mod shut_down;
mod sort_players;
pub(crate) mod start_link;
mod steam;
mod stop;
mod stopping;
pub(crate) mod store;
mod store_received;
mod system_message;
mod tables;
mod take_messages;
mod take_player_settings;
mod types;

/// Logs as the send functions do: "send x", the player, the step.
fn log_send(race: &Race, net: &owner::Net, text: &std::ffi::CStr) {
    use crate::logfile::log_int::log_int;
    crate::logfile::log_write::log_nul(&text, false, 1);
    log_int(net.race_slot.player() as i32, false, 1);
    log_int(race.shared.view.get().local_car.get() as i32, true, 1);
}
