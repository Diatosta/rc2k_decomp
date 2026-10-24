//! Functions that lay out parts of a screen: fixed positions, fonts and string ids.

use crate::framework::{game_ptr, rva};
use crate::text::font::Font;

pub mod best_scores;
pub mod buttons;
pub mod car_caption;
pub mod car_page_description;
pub mod car_page_dimensions;
pub mod car_page_engine;
pub mod car_page_title;
pub mod car_page_transmission;
pub mod championship_table;
pub mod controller;
pub mod draw_arcade_stage_title;
pub mod draw_background_band;
pub mod draw_best_scores;
pub mod draw_bottom_panel;
pub mod draw_brake_balance_marker;
pub mod draw_button;
pub mod draw_button_alt;
pub mod draw_button_highlighted;
pub mod draw_car_figures;
pub mod draw_championship_standings;
pub mod draw_class_ranking;
pub mod draw_class_times;
pub mod draw_class_times_marked;
pub mod draw_column_boxes;
pub mod draw_column_rules;
pub mod draw_control_bindings;
pub mod draw_duel_result;
pub mod draw_empty_savegame;
pub mod draw_event_best_scores;
pub mod draw_fade_overlay;
pub mod draw_graphics_settings;
pub mod draw_highlighted_caption;
pub mod draw_inspection_status;
pub mod draw_item_button;
pub mod draw_leave_question;
pub mod draw_load_setup_list;
pub mod draw_menu_items;
pub mod draw_menu_labels;
pub mod draw_message_box;
pub mod draw_name_cursor;
pub mod draw_note_1fd;
pub mod draw_options_checklist;
pub mod draw_overall_best_scores;
pub mod draw_player_details;
pub mod draw_player_rally_result;
pub mod draw_player_stage_result;
pub mod draw_players_results;
pub mod draw_players_results_points;
pub mod draw_players_standings;
pub mod draw_rally_stage_times;
pub mod draw_rally_times_heading;
pub mod draw_record_heading;
pub mod draw_redefined_keys;
pub mod draw_repair_times;
pub mod draw_replay_slot_names;
pub mod draw_ride_height_installed_marker;
pub mod draw_ride_height_marker;
pub mod draw_save_setup_list;
pub mod draw_savegame_details;
pub mod draw_savegame_names;
pub mod draw_savegame_preview;
pub mod draw_section_best_times;
pub mod draw_selected_stage_menu;
pub mod draw_service_area;
pub mod draw_service_boxes;
pub mod draw_service_stage_info;
pub mod draw_service_time;
pub mod draw_sound_settings;
pub mod draw_spaced_number;
pub mod draw_stage_best_and_fastest;
pub mod draw_stage_heading;
pub mod draw_stage_info;
pub mod draw_stage_records;
pub mod draw_stage_times;
pub mod draw_stage_times_heading;
pub mod draw_steering_sensitivity_marker;
pub mod draw_suspension_stiffness_installed_marker;
pub mod draw_suspension_stiffness_marker;
pub mod draw_time_trial_stage_info;
pub mod draw_time_trial_stage_select;
pub mod draw_two_column_panel;
pub mod draw_visual_preferences;
pub mod draw_volume_settings;
pub mod game_screen;
pub mod gearing_text;
pub mod harness;
pub mod itinerary;
pub mod locked_notes;
pub mod page_settings;
pub mod player_number_text;
pub mod players_results;
pub mod results_pages;
pub mod savegame_page_numbers;
pub mod service_area_frame;
pub mod service_box_frame_1;
pub mod service_box_frame_2;
pub mod service_box_frame_3;
pub mod service_box_frame_4;
pub mod service_boxes;
pub mod set_stage_choice_titles;
pub mod setup_markers;
pub mod slideshow_title;
pub mod stage_info;
pub mod stage_panel;
pub mod stage_results;
pub mod yes_no_caption;

/// byte_A8BB54: the text buffer these functions build their lines in.
pub const LINE_BUFFER_RVA: usize = rva(0xA8BB54);

pub fn line_buffer() -> *mut u8 {
    game_ptr(LINE_BUFFER_RVA)
}

/// A font object in game memory, by its address in IDA (where it lives now, if it has moved:
/// `crate::moved::place`).
pub fn font_at(addr: usize) -> *mut Font {
    crate::moved::place(addr) as *mut Font
}
