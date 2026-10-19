//! Functions that lay out parts of a screen: fixed positions, fonts and string ids.

use crate::framework::{GameHook, game_ptr, rva};
use crate::text::font::Font;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub mod best_scores;
pub mod best_scores_caption;
pub mod buttons;
pub mod car_caption;
pub mod car_caption_2;
pub mod car_page_description;
pub mod car_page_dimensions;
pub mod car_page_engine;
pub mod car_page_title;
pub mod car_page_transmission;
pub mod car_pages;
pub mod championship_table;
pub mod classify_controller;
pub mod controller;
pub mod draw_background_band;
pub mod draw_best_scores;
pub mod draw_best_scores_page;
pub mod draw_bottom_panel;
pub mod draw_brake_balance_marker;
pub mod draw_button;
pub mod draw_button_alt;
pub mod draw_button_highlighted;
pub mod draw_car_figures;
pub mod draw_championship_table;
pub mod draw_class_ranking;
pub mod draw_class_times;
pub mod draw_class_times_marked;
pub mod draw_column_boxes;
pub mod draw_column_rules;
pub mod draw_control_bindings;
pub mod draw_controller_page;
pub mod draw_duel_result;
pub mod draw_empty_savegame;
pub mod draw_event_best_scores;
pub mod draw_fade_overlay;
pub mod draw_graphics_settings;
pub mod draw_highlighted_caption;
pub mod draw_inspection_status;
pub mod draw_item_button;
pub mod draw_item_button_centred;
pub mod draw_item_button_centred_highlighted;
pub mod draw_item_button_right;
pub mod draw_item_button_right_highlighted;
pub mod draw_itinerary;
pub mod draw_leave_question;
pub mod draw_load_setup_list;
pub mod draw_menu_item_text;
pub mod draw_menu_items;
pub mod draw_menu_labels;
pub mod draw_message_box;
pub mod draw_name_cursor;
pub mod draw_no_controller;
pub mod draw_note_1fd;
pub mod draw_options_checklist;
pub mod draw_overall_best_scores;
pub mod draw_player2_details;
pub mod draw_player34_details;
pub mod draw_player_details;
pub mod draw_player_rally_result;
pub mod draw_player_stage_result;
pub mod draw_players_results;
pub mod draw_players_results_points;
pub mod draw_players_standings;
pub mod draw_rally_stage_info;
pub mod draw_rally_stage_times;
pub mod draw_rally_times_heading;
pub mod draw_record_heading;
pub mod draw_record_times;
pub mod draw_redefined_keys;
pub mod draw_repair_times;
pub mod draw_replay_slot_names;
pub mod draw_ride_height_installed_marker;
pub mod draw_ride_height_marker;
pub mod draw_save_setup_list;
pub mod draw_savegame_details;
pub mod draw_savegame_names;
pub mod draw_savegame_preview;
pub mod draw_selected_stage_menu;
pub mod draw_service_area;
pub mod draw_service_boxes;
pub mod draw_service_stage_info;
pub mod draw_service_time;
pub mod draw_setup_slot_names;
pub mod draw_sound_settings;
pub mod draw_spaced_number;
pub mod draw_stage_best_and_fastest;
pub mod draw_stage_heading;
pub mod draw_stage_info;
pub mod draw_stage_records;
pub mod draw_stage_results;
pub mod draw_stage_times;
pub mod draw_stage_times_heading;
pub mod draw_steering_sensitivity_marker;
pub mod draw_suspension_stiffness_installed_marker;
pub mod draw_suspension_stiffness_marker;
pub mod draw_time_trial_stage_info;
pub mod draw_two_column_panel;
pub mod draw_visual_preferences;
pub mod draw_volume_settings;
pub mod find_history_entry;
pub mod gearing_text;
pub mod harness;
pub mod itinerary;
pub mod learn_axis;
pub mod load_audio_settings;
pub mod locked_notes;
pub mod note_locked_item_3;
pub mod note_locked_stage;
pub mod page_settings;
pub mod pick_player_1;
pub mod pick_player_2;
pub mod pick_player_3;
pub mod pick_player_4;
pub mod player_number_text;
pub mod players_results;
pub mod read_controller;
pub mod replay_range_caption;
pub mod savegame_page_numbers;
pub mod savegame_preview;
pub mod service_area_frame;
pub mod service_box_frame_1;
pub mod service_box_frame_2;
pub mod service_box_frame_3;
pub mod service_box_frame_4;
pub mod service_boxes;
pub mod set_stage_choice_titles;
pub mod setup_markers;
pub mod slideshow_title;
pub mod stage_choice_texts;
pub mod stage_info;
pub mod stage_options_texts;
pub mod stage_panel;
pub mod stage_results;
pub mod yes_no_caption;

pub static HOOKS: &[&dyn GameHook] = &[
    &draw_fade_overlay::HOOK,
    &draw_service_boxes::HOOK,
    &draw_stage_heading::HOOK,
    &draw_background_band::HOOK,
    &draw_highlighted_caption::HOOK,
    &draw_stage_records::HOOK,
    &draw_stage_info::HOOK,
    &draw_service_stage_info::HOOK,
    &draw_time_trial_stage_info::HOOK,
    &draw_selected_stage_menu::HOOK,
    &draw_service_time::HOOK,
    &draw_repair_times::HOOK,
    &draw_inspection_status::HOOK,
    &draw_steering_sensitivity_marker::HOOK,
    &draw_brake_balance_marker::HOOK,
    &draw_ride_height_marker::HOOK,
    &draw_suspension_stiffness_marker::HOOK,
    &draw_ride_height_installed_marker::HOOK,
    &draw_suspension_stiffness_installed_marker::HOOK,
    &draw_service_area::HOOK,
    &draw_replay_slot_names::HOOK,
    &draw_setup_slot_names::HOOK,
    &draw_save_setup_list::HOOK,
    &draw_load_setup_list::HOOK,
    &draw_empty_savegame::HOOK,
    &draw_savegame_details::HOOK,
    &draw_savegame_preview::HOOK,
    &draw_menu_labels::HOOK,
    &draw_button::HOOK,
    &draw_button_highlighted::HOOK,
    &draw_button_alt::HOOK,
    &draw_item_button::HOOK,
    &draw_item_button_right::HOOK,
    &draw_item_button_centred::HOOK,
    &draw_item_button_right_highlighted::HOOK,
    &draw_item_button_centred_highlighted::HOOK,
    &draw_menu_item_text::HOOK,
    &draw_menu_items::HOOK,
    &service_area_frame::HOOK,
    &service_box_frame_1::HOOK,
    &service_box_frame_2::HOOK,
    &service_box_frame_3::HOOK,
    &service_box_frame_4::HOOK,
    &yes_no_caption::HOOK,
    &draw_leave_question::HOOK,
    &savegame_page_numbers::HOOK,
    &draw_savegame_names::HOOK,
    &car_caption::HOOK,
    &car_caption_2::HOOK,
    &gearing_text::HOOK,
    &player_number_text::HOOK,
    &set_stage_choice_titles::HOOK,
    &car_page_title::HOOK,
    &slideshow_title::HOOK,
    &draw_car_figures::HOOK,
    &draw_name_cursor::HOOK,
    &draw_player_details::HOOK,
    &draw_player2_details::HOOK,
    &draw_player34_details::HOOK,
    &draw_redefined_keys::HOOK,
    &pick_player_1::HOOK,
    &pick_player_2::HOOK,
    &pick_player_3::HOOK,
    &pick_player_4::HOOK,
    &load_audio_settings::HOOK,
    &draw_sound_settings::HOOK,
    &draw_graphics_settings::HOOK,
    &draw_player_stage_result::HOOK,
    &draw_options_checklist::HOOK,
    &draw_player_rally_result::HOOK,
    &draw_itinerary::HOOK,
    &draw_players_standings::HOOK,
    &draw_rally_stage_info::HOOK,
    &draw_stage_best_and_fastest::HOOK,
    &draw_stage_results::HOOK,
    &find_history_entry::HOOK,
    &draw_championship_table::HOOK,
    &draw_control_bindings::HOOK,
    &draw_volume_settings::HOOK,
    &draw_duel_result::HOOK,
    &draw_spaced_number::HOOK,
    &draw_players_results::HOOK,
    &draw_players_results_points::HOOK,
    &draw_bottom_panel::HOOK,
    &draw_column_rules::HOOK,
    &draw_column_boxes::HOOK,
    &draw_two_column_panel::HOOK,
    &draw_message_box::HOOK,
    &note_locked_item_3::HOOK,
    &note_locked_stage::HOOK,
    &read_controller::HOOK,
    &classify_controller::HOOK,
    &learn_axis::HOOK,
    &draw_no_controller::HOOK,
    &draw_controller_page::HOOK,
    &stage_options_texts::HOOK,
    &stage_choice_texts::HOOK,
    &best_scores_caption::HOOK,
    &draw_best_scores::HOOK,
    &draw_event_best_scores::HOOK,
    &draw_overall_best_scores::HOOK,
    &draw_best_scores_page::HOOK,
    &draw_class_times::HOOK,
    &draw_class_times_marked::HOOK,
    &draw_stage_times_heading::HOOK,
    &draw_rally_times_heading::HOOK,
    &draw_stage_times::HOOK,
    &draw_rally_stage_times::HOOK,
    &draw_record_heading::HOOK,
    &draw_record_times::HOOK,
    &draw_note_1fd::HOOK,
    &replay_range_caption::HOOK,
    &draw_visual_preferences::HOOK,
    &draw_class_ranking::HOOK,
    &car_page_description::HOOK,
    &car_page_engine::HOOK,
    &car_page_transmission::HOOK,
    &car_page_dimensions::HOOK,
];

/// Console switch `blank on [prefix]` / `blank off`, kept for every hook in this module: a
/// blanked hook returns at once, running neither the original nor the Rust version, so
/// whatever it draws disappears from the screen. That is how to confirm, in the game, what a
/// screen function draws. Without a prefix every hook of this module is blanked; with one,
/// only the hooks whose name starts with it, or the one hook with exactly that name. It is harmless for these functions because they
/// preserve every register; a new hook in this module must check `blanked(name)` right after
/// `count_call()`.
static BLANK: AtomicBool = AtomicBool::new(false);
static BLANK_PREFIX: Mutex<String> = Mutex::new(String::new());

pub fn set_blank(on: bool, prefix: &str) {
    if let Ok(mut p) = BLANK_PREFIX.lock() {
        *p = prefix.to_string();
    }
    BLANK.store(on, Ordering::Relaxed);
}

/// True if the hook called `name` is blanked at the moment. A prefix that is itself the full
/// name of a hook selects only that hook (`screens::draw_time_trial_stage_info` is also how
/// `screens::draw_selected_stage_menu` starts).
pub fn blanked(name: &str) -> bool {
    BLANK.load(Ordering::Relaxed)
        && BLANK_PREFIX
            .lock()
            .is_ok_and(|prefix| crate::registry::selects(prefix.as_str(), name))
}

/// byte_A8BB54: the text buffer these functions build their lines in.
pub const LINE_BUFFER_RVA: usize = rva(0xA8BB54);

pub fn line_buffer() -> *mut u8 {
    game_ptr(LINE_BUFFER_RVA)
}

/// A font object in game memory, by its address in IDA.
pub fn font_at(addr: usize) -> *mut Font {
    game_ptr(rva(addr))
}

/// Copies `bytes` (a text with its NUL) to the line buffer.
pub unsafe fn set_line(bytes: &[u8]) {
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), line_buffer(), bytes.len()) };
}

/// True if the font object at `addr` has a glyph table, so that text can be measured with it.
pub fn font_is_loaded(addr: usize) -> bool {
    unsafe { font_at(addr).read_unaligned() }.glyph_table != 0
}
