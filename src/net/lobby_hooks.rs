//! The lobby's own functions as places of `net::lobby::game_part`: their Rust versions.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{game_part, not_known};

type Rust = fn(&Regs, crate::net::Part) -> Result<Regs, Refused>;

/// The Rust version of a lobby function by its IDA address.
fn rust_of(place: u32) -> Option<Rust> {
    use crate::net::*;
    Some(match place {
        0x418530 => connection_entry::connection_entry,
        0x418553 => session_entry::session_entry,
        0x418576 => allocate_lists::allocate_lists,
        0x41896E => add_address_element::add_address_element,
        0x416E1A => begin_edit::begin_edit,
        0x416E43 => edit_key::edit_key,
        0x419411 => clear_chat::clear_chat,
        0x419428 => add_chat_line::add_chat_line,
        0x4193FF => chat_line_locked::chat_line_locked,
        0x4193C8 => send_chat::send_chat,
        0x41819B => send_chat_regs::send_chat_regs,
        0x418108 => sort_players::sort_players,
        0x4181A3 => fill_own_player::fill_own_player,
        0x48755D => take_player_settings::take_player_settings,
        0x463EB6 => clear_car_state::clear_car_state,
        0x463E52 => reset_cars::reset_cars,
        0x463A94 => set_up_race::set_up_race,
        0x463D3F => set_hud_options::set_hud_options,
        0x463E39 => keep_hud_options::keep_hud_options,
        0x4640AA => lets_race::lets_race,
        0x4AAAF7 => car_list::car_list,
        0x4185DF => find_saved_connection::find_saved_connection,
        0x417661 => keep_connection::keep_connection,
        0x4174BD => find_connections_regs::find_connections_regs,
        0x41771E => find_sessions_regs::find_sessions_regs,
        0x417A03 => list_players_regs::list_players_regs,
        0x41837B => close_if_host::close_if_host,
        0x41861D => create_dplay::create_dplay,
        0x418E9B => open_lobby::open_lobby,
        0x41867C => find_connections::find_connections,
        0x41882C => build_address::build_address,
        0x418794 => init_connection::init_connection,
        0x418999 => find_sessions::find_sessions,
        0x4190FB => create_player::create_player,
        0x4192EA => create_group::create_group,
        0x419354 => join_group::join_group,
        0x419192 => list_players::list_players,
        0x418B13 => host_session::host_session,
        0x418C6D => join_session::join_session,
        0x418D8A => disable_new_players::disable_new_players,
        0x4184F8 => post_call::post_call,
        0x4173E7 => show_background::show_background,
        0x41748D => message_frame::message_frame,
        0x41745C => show_message::show_message,
        0x417688 => connecting::connecting,
        0x4176D5 => hosting::hosting,
        0x4179B6 => joining::joining,
        0x417089 => stopping::stopping,
        0x416CE0 => draw_button::draw_button,
        0x416AF8 => setting_label::setting_label,
        0x416B53 => draw_centred_item::draw_centred_item,
        0x416BC4 => draw_setting_value::draw_setting_value,
        0x416B8E => draw_number_value::draw_number_value,
        0x4169E2 => settings_frame::settings_frame,
        0x416C26 => begin_setting_edit::begin_setting_edit,
        0x416C76 => setting_key::setting_key,
        0x416EEC => settings_link::settings_link,
        0x416F0B => settings_done::settings_done,
        0x4168CF => settings_loop::settings_loop,
        0x4175C7 => connections_frame::connections_frame,
        0x41790E => sessions_frame::sessions_frame,
        0x4174C5 => choose_connection::choose_connection,
        0x417726 => choose_session::choose_session,
        0x41802F => lobby_button::lobby_button,
        0x418083 => chat_input_line::chat_input_line,
        0x4180C5 => begin_chat::begin_chat,
        0x4180E4 => chat_key::chat_key,
        0x417D95 => lobby_frame::lobby_frame,
        0x4182FA => choose_car::choose_car,
        0x418347 => choose_stage::choose_stage,
        0x417A0D => lobby_loop::lobby_loop,
        0x4171FE => race_wait::race_wait,
        0x41732E => race_over::race_over,
        0x4170E0 => lets_rally::lets_rally,
        0x416F50 => start_link::start_link,
        0x418DF4 => lobby_launched_game::lobby_launched_game,
        _ => return None,
    })
}

pub fn call(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match rust_of(place) {
        Some(rust) => crate::race::loadlevel::or_in_game(place as usize, r, |r| {
            *r = rust(r, &mut game_part)?;
            Ok(())
        }),
        None => Err(not_known(place)),
    }
}
