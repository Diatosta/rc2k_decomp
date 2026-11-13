//! The menus, out of RAL.EXE's image: the 98 lists of labels (IDA 0x747AD8 .. 0x74826C) as `#[repr(C)]` cells in the image's layout (the screens that
//! draw a menu's labels, and the original's slip that writes into them, walk them by offset), and the 96 menus as handles into `rc2k_core::menu::tables::MENUS`,
//! the one source of the menus' items (their places, links, functions and starting values).
//!
//! What the game changes in play is kept here, per item (`MenuItem`, 567 of them in one
//! array, the menus' in `MENUS`' order): the string id (+0x14), the font (+0x18), +0x1C, the
//! sprite (+0x20), the font a highlighted item pulses in (+0x50) and the skip mark (+0x48),
//! each starting as the table has it. Nothing else of an item is written anywhere (the
//! compiler says so: the rest is read from `MenuItem::fixed`). Menus are named for the screen that
//! shows them (`menu::screen_loop`'s `Screen::menu`), the ones no screen names by their
//! handle in `MenuTables`.

use crate::gfx::typefaces::{FontCell, FontId, fonts};
use crate::menu::item_sprites_store::SpriteCell;
#[cfg(test)]
use crate::menu::owner::menus;
use crate::session::PlainDword;
use rc2k_core::menu::tables::{MENUS, MenuFont, MenuItem as Fixed};

/// A label: x, y, the font, the string id.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct MenuLabel {
    pub x: PlainDword,
    pub y: PlainDword,
    /// The font.
    pub font: FontCell,
    pub string: PlainDword,
}

impl MenuLabel {
    pub const fn new(x: u32, y: u32, font: FontId, string: u32) -> Self {
        Self {
            x: PlainDword::new(x),
            y: PlainDword::new(y),
            font: FontCell::new(font),
            string: PlainDword::new(string),
        }
    }
}

/// A list of labels: the label and the -1 that ends the list.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct LabelList {
    pub label: MenuLabel,
    pub end: PlainDword,
}

impl LabelList {
    pub const fn new(x: u32, y: u32, font: FontId, string: u32) -> Self {
        Self {
            label: MenuLabel::new(x, y, font, string),
            end: PlainDword::new(0xFFFF_FFFF),
        }
    }
}

/// The label lists in the image's layout.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct LabelTables {
    /// Label list at IDA 0x747AD8 (the labels of the single_rally_review menu).
    pub single_rally_review_labels: LabelList,
    /// Label list at IDA 0x747AEC (the labels of the arcade_review_options menu).
    pub arcade_review_options_labels: LabelList,
    /// Label list at IDA 0x747B00 (the labels of the arcade_next_level_review menu).
    pub arcade_next_level_review_labels: LabelList,
    /// Label list at IDA 0x747B14 (the labels of the arcade_retry_review menu).
    pub arcade_retry_review_labels: LabelList,
    /// Label list at IDA 0x747B28 (the labels of the menu after the previous one in the menu array).
    pub after_arcade_retry_review_labels: LabelList,
    /// Label list at IDA 0x747B3C (the labels of the stage_result menu).
    pub stage_result_labels: LabelList,
    /// Label list at IDA 0x747B50.
    pub labels_747b50: LabelList,
    /// Label list at IDA 0x747B64 (the labels of the single_rally_two_results menu).
    pub single_rally_two_results_labels: LabelList,
    /// Label list at IDA 0x747B78 (the labels of the menu after the previous one in the menu array).
    pub after_single_rally_two_results_labels: LabelList,
    /// Label list at IDA 0x747B8C (the labels of the single_rally_three_four_results menu).
    pub single_rally_three_four_results_labels: LabelList,
    /// Label list at IDA 0x747BA0 (the labels of the single_rally_overall_results menu).
    pub single_rally_overall_results_labels: LabelList,
    /// Label list at IDA 0x747BB4 (the labels of the time_trial_results menu).
    pub time_trial_results_labels: LabelList,
    /// Label list at IDA 0x747BC8.
    pub labels_747bc8: LabelList,
    /// Label list at IDA 0x747BDC (the labels of the main_menu menu).
    pub main_menu_labels: LabelList,
    /// Label list at IDA 0x747BF0 (the labels of the network_rally menu).
    pub network_rally_labels: LabelList,
    /// Label list at IDA 0x747C04 (the labels of the start_rally menu).
    pub start_rally_labels: LabelList,
    /// Label list at IDA 0x747C18 (the labels of the championship_new_simulation menu).
    pub championship_new_simulation_labels: LabelList,
    /// Label list at IDA 0x747C2C (the labels of the new_simulation menu).
    pub new_simulation_labels: LabelList,
    /// Label list at IDA 0x747C40 (the labels of the menu after the previous one in the menu array).
    pub after_new_simulation_labels: LabelList,
    /// Label list at IDA 0x747C54 (the labels of the menu after the previous one in the menu array).
    pub second_after_new_simulation_labels: LabelList,
    /// Label list at IDA 0x747C68 (the labels of the simple_select_players menu).
    pub simple_select_players_labels: LabelList,
    /// Label list at IDA 0x747C7C (the labels of the time_trial_select_players menu).
    pub time_trial_select_players_labels: LabelList,
    /// Label list at IDA 0x747C90.
    pub labels_747c90: LabelList,
    /// Label list at IDA 0x747CA4 (the labels of the arcade_select_level menu).
    pub arcade_select_level_labels: LabelList,
    /// Label list at IDA 0x747CB8 (the labels of the service_area menu).
    pub service_area_labels: LabelList,
    /// Label list at IDA 0x747CCC (the labels of the menu after the previous one in the menu array).
    pub after_service_area_labels: LabelList,
    /// Label list at IDA 0x747CE0 (the labels of the in_game_options menu).
    pub in_game_options_labels: LabelList,
    /// Label list at IDA 0x747CF4 (the labels of the menu after the previous one in the menu array).
    pub after_in_game_options_labels: LabelList,
    /// Label list at IDA 0x747D08 (the labels of the audio_setup menu).
    pub audio_setup_labels: LabelList,
    /// Label list at IDA 0x747D1C (the labels of the advanced_audio menu).
    pub advanced_audio_labels: LabelList,
    /// Label list at IDA 0x747D30 (the labels of the leave_question menu).
    pub leave_question_labels: LabelList,
    /// Label list at IDA 0x747D44 (the labels of the car_specification menu).
    pub car_specification_labels: LabelList,
    /// Label list at IDA 0x747D58 (the labels of the slideshow menu).
    pub slideshow_labels: LabelList,
    /// Label list at IDA 0x747D6C (the labels of the select_car menu).
    pub select_car_labels: LabelList,
    /// Label list at IDA 0x747D80 (the labels of the car_specs_display menu).
    pub car_specs_display_labels: LabelList,
    /// Label list at IDA 0x747D94 (the labels of the cspecs_menu menu).
    pub menu_74ba3c_labels: LabelList,
    /// Label list at IDA 0x747DA8 (the labels of the inspection_area menu).
    pub inspection_area_labels: LabelList,
    /// Label list at IDA 0x747DBC (the labels of the welcome menu).
    pub welcome_labels: LabelList,
    /// Label list at IDA 0x747DD0 (the labels of the arcade_next_stage menu).
    pub arcade_next_stage_labels: LabelList,
    /// Label list at IDA 0x747DE4 (the labels of the arcade_level_end_results menu).
    pub arcade_level_end_results_labels: LabelList,
    /// Label list at IDA 0x747DF8 (the labels of the arcade_results menu).
    pub arcade_results_labels: LabelList,
    /// Label list at IDA 0x747E0C (the labels of the rally_stages menu).
    pub rally_stages_labels: LabelList,
    /// Label list at IDA 0x747E20 (the labels of the class_stage_results menu).
    pub class_stage_results_labels: LabelList,
    /// Label list at IDA 0x747E34 (the labels of the overall_stage_results menu).
    pub overall_stage_results_labels: LabelList,
    /// Label list at IDA 0x747E48 (the labels of the class_rally_results menu).
    pub class_rally_results_labels: LabelList,
    /// Label list at IDA 0x747E5C (the labels of the overall_rally_results menu).
    pub overall_rally_results_labels: LabelList,
    /// Label list at IDA 0x747E70 (the labels of the menu after the previous one in the menu array).
    pub after_overall_rally_results_labels: LabelList,
    /// Label list at IDA 0x747E84 (the labels of the menu after the previous one in the menu array).
    pub second_after_overall_rally_results_labels: LabelList,
    /// Label list at IDA 0x747E98 (the labels of the class_championship_standings menu).
    pub class_championship_standings_labels: LabelList,
    /// Label list at IDA 0x747EAC (the labels of the championship_standings menu).
    pub championship_standings_labels: LabelList,
    /// Label list at IDA 0x747EC0 (the labels of the section_times_menu menu).
    pub menu_74fea8_labels: LabelList,
    /// Label list at IDA 0x747ED4 (the labels of the championship_service_review menu).
    pub championship_service_review_labels: LabelList,
    /// Label list at IDA 0x747EE8 (the labels of the championship_standings_review menu).
    pub championship_standings_review_labels: LabelList,
    /// Label list at IDA 0x747EFC (the labels of the championship_races_review menu).
    pub championship_races_review_labels: LabelList,
    /// Label list at IDA 0x747F10 (the labels of the championship_end_review menu).
    pub championship_end_review_labels: LabelList,
    /// Label list at IDA 0x747F24 (the labels of the select_stage menu).
    pub select_stage_labels: LabelList,
    /// Label list at IDA 0x747F38 (the labels of the time_trial_select_stage menu).
    pub time_trial_select_stage_labels: LabelList,
    /// Label list at IDA 0x747F4C (the labels of the next_stage menu).
    pub next_stage_labels: LabelList,
    /// Label list at IDA 0x747F60 (the labels of the view_stage menu).
    pub view_stage_labels: LabelList,
    /// Label list at IDA 0x747F74 (the labels of the view_courses menu).
    pub view_courses_labels: LabelList,
    /// Label list at IDA 0x747F88.
    pub labels_747f88: LabelList,
    /// Label list at IDA 0x747F9C (the labels of the player1_controller menu).
    pub player1_controller_labels: LabelList,
    /// Label list at IDA 0x747FB0 (the labels of the player1_buttons menu).
    pub player1_buttons_labels: LabelList,
    /// Label list at IDA 0x747FC4 (the labels of the player2_controller menu).
    pub player2_controller_labels: LabelList,
    /// Label list at IDA 0x747FD8 (the labels of the player2_buttons menu).
    pub player2_buttons_labels: LabelList,
    /// Label list at IDA 0x747FEC (the labels of the player3_controller menu).
    pub player3_controller_labels: LabelList,
    /// Label list at IDA 0x748000 (the labels of the player3_buttons menu).
    pub player3_buttons_labels: LabelList,
    /// Label list at IDA 0x748014 (the labels of the player4_controller menu).
    pub player4_controller_labels: LabelList,
    /// Label list at IDA 0x748028 (the labels of the player4_buttons menu).
    pub player4_buttons_labels: LabelList,
    /// Label list at IDA 0x74803C (the labels of the redefine_keys_player1 menu).
    pub redefine_keys_player1_labels: LabelList,
    /// Label list at IDA 0x748050 (the labels of the redefine_keys_player2 menu).
    pub redefine_keys_player2_labels: LabelList,
    /// Label list at IDA 0x748064 (the labels of the redefine_keys_player3 menu).
    pub redefine_keys_player3_labels: LabelList,
    /// Label list at IDA 0x748078 (the labels of the redefine_keys_player4 menu).
    pub redefine_keys_player4_labels: LabelList,
    /// Label list at IDA 0x74808C (the labels of the view_best_times menu).
    pub view_best_times_labels: LabelList,
    /// Label list at IDA 0x7480A0 (the labels of the view_best_scores menu).
    pub view_best_scores_labels: LabelList,
    /// Label list at IDA 0x7480B4 (the labels of the best_times menu).
    pub best_times_labels: LabelList,
    /// Label list at IDA 0x7480C8 (the labels of the arcade_best_times menu).
    pub arcade_best_times_labels: LabelList,
    /// Label list at IDA 0x7480DC (the labels of the time_trial_best_times menu).
    pub time_trial_best_times_labels: LabelList,
    /// Label list at IDA 0x7480F0 (the labels of the arcade_best_scores menu).
    pub arcade_best_scores_labels: LabelList,
    /// Label list at IDA 0x748104 (the labels of the best_class_scores menu).
    pub best_class_scores_labels: LabelList,
    /// Label list at IDA 0x748118 (the labels of the best_overall_scores menu).
    pub best_overall_scores_labels: LabelList,
    /// Label list at IDA 0x74812C (the labels of the menu after the previous one in the menu array).
    pub after_best_overall_scores_labels: LabelList,
    /// Label list at IDA 0x748140 (the labels of the options menu).
    pub options_labels: LabelList,
    /// Label list at IDA 0x748154 (the labels of the menu after the previous one in the menu array).
    pub after_options_labels: LabelList,
    /// Label list at IDA 0x748168 (the labels of the player1_details menu).
    pub player1_details_labels: LabelList,
    /// Label list at IDA 0x74817C (the labels of the player2_details menu).
    pub player2_details_labels: LabelList,
    /// Label list at IDA 0x748190 (the labels of the player3_details menu).
    pub player3_details_labels: LabelList,
    /// Label list at IDA 0x7481A4 (the labels of the player4_details menu).
    pub player4_details_labels: LabelList,
    /// Label list at IDA 0x7481B8 (the labels of the player_details menu).
    pub player_details_labels: LabelList,
    /// Label list at IDA 0x7481CC (the labels of the information menu).
    pub information_labels: LabelList,
    /// Label list at IDA 0x7481E0 (the labels of the load_championship menu).
    pub load_championship_labels: LabelList,
    /// Label list at IDA 0x7481F4 (the labels of the save_championship menu).
    pub save_championship_labels: LabelList,
    /// Label list at IDA 0x748208 (the labels of the load_replay menu).
    pub load_replay_labels: LabelList,
    /// Label list at IDA 0x74821C (the labels of the save_replay menu).
    pub save_replay_labels: LabelList,
    /// Label list at IDA 0x748230.
    pub labels_748230: LabelList,
    /// Label list at IDA 0x748244 (the labels of the save_setup menu).
    pub save_setup_labels: LabelList,
    /// Label list at IDA 0x748258 (the labels of the visual_preferences menu).
    pub visual_preferences_labels: LabelList,
    /// Label list at IDA 0x74826C (the labels of the visual_options menu).
    pub visual_options_labels: LabelList,
}

#[rustfmt::skip]
impl LabelTables {
    pub const fn new() -> Self {
        LabelTables {
        single_rally_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_review_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_next_level_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_retry_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        after_arcade_retry_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        stage_result_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        labels_747b50: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_two_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        after_single_rally_two_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_three_four_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_overall_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x205),
        time_trial_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        labels_747bc8: LabelList::new(0x310, 0x30, fonts::BOX_TEXT, 0x94),
        main_menu_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x12F),
        network_rally_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x147),
        start_rally_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1D7),
        championship_new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1EC),
        new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1EC),
        after_new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E0),
        second_after_new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E0),
        simple_select_players_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        time_trial_select_players_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        labels_747c90: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        arcade_select_level_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1F5),
        service_area_labels: LabelList::new(0x310, 0x1D, fonts::HEAD, 0x0),
        after_service_area_labels: LabelList::new(0x310, 0x1D, fonts::HEAD, 0x0),
        in_game_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x521),
        after_in_game_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x521),
        audio_setup_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x53B),
        advanced_audio_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x549),
        leave_question_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x53A),
        car_specification_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x21E),
        slideshow_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33C),
        select_car_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
        car_specs_display_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
        menu_74ba3c_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
        inspection_area_labels: LabelList::new(0x190, 0x30, fonts::BOX_TEXT, 0x522),
        welcome_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E4),
        arcade_next_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x357),
        arcade_level_end_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        arcade_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        rally_stages_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x342),
        class_stage_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4E8),
        overall_stage_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4E9),
        class_rally_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        overall_rally_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        after_overall_rally_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4ED),
        second_after_overall_rally_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4EE),
        class_championship_standings_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4F0),
        championship_standings_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4F1),
        menu_74fea8_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4EF),
        championship_service_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_standings_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_races_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_end_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        select_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33F),
        time_trial_select_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33F),
        next_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x357),
        view_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x533),
        view_courses_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x343),
        labels_747f88: LabelList::new(0x310, 0x30, fonts::HEAD, 0x343),
        player1_controller_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player1_buttons_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player2_controller_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player2_buttons_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player3_controller_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player3_buttons_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player4_controller_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player4_buttons_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        redefine_keys_player1_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x3C6),
        redefine_keys_player2_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x3C6),
        redefine_keys_player3_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x3C6),
        redefine_keys_player4_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x3C6),
        view_best_times_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35A),
        view_best_scores_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35B),
        best_times_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35A),
        arcade_best_times_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35A),
        time_trial_best_times_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35A),
        arcade_best_scores_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35B),
        best_class_scores_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35C),
        best_overall_scores_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x35D),
        after_best_overall_scores_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x532),
        options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x13B),
        after_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1A3),
        player1_details_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x176),
        player2_details_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x177),
        player3_details_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x178),
        player4_details_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x179),
        player_details_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x17A),
        information_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x195),
        load_championship_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4DA),
        save_championship_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4DB),
        load_replay_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4DF),
        save_replay_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4DE),
        labels_748230: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        save_setup_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        visual_preferences_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x369),
        visual_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x37F),
        }
    }
}

/// The 98 label lists, in the image's order (the first 98 fields of `MenuTables`, one after
/// the other from IDA 0x747AD8).
pub fn label_lists(labels: &LabelTables) -> [&LabelList; 98] {
    [
        &labels.single_rally_review_labels,
        &labels.arcade_review_options_labels,
        &labels.arcade_next_level_review_labels,
        &labels.arcade_retry_review_labels,
        &labels.after_arcade_retry_review_labels,
        &labels.stage_result_labels,
        &labels.labels_747b50,
        &labels.single_rally_two_results_labels,
        &labels.after_single_rally_two_results_labels,
        &labels.single_rally_three_four_results_labels,
        &labels.single_rally_overall_results_labels,
        &labels.time_trial_results_labels,
        &labels.labels_747bc8,
        &labels.main_menu_labels,
        &labels.network_rally_labels,
        &labels.start_rally_labels,
        &labels.championship_new_simulation_labels,
        &labels.new_simulation_labels,
        &labels.after_new_simulation_labels,
        &labels.second_after_new_simulation_labels,
        &labels.simple_select_players_labels,
        &labels.time_trial_select_players_labels,
        &labels.labels_747c90,
        &labels.arcade_select_level_labels,
        &labels.service_area_labels,
        &labels.after_service_area_labels,
        &labels.in_game_options_labels,
        &labels.after_in_game_options_labels,
        &labels.audio_setup_labels,
        &labels.advanced_audio_labels,
        &labels.leave_question_labels,
        &labels.car_specification_labels,
        &labels.slideshow_labels,
        &labels.select_car_labels,
        &labels.car_specs_display_labels,
        &labels.menu_74ba3c_labels,
        &labels.inspection_area_labels,
        &labels.welcome_labels,
        &labels.arcade_next_stage_labels,
        &labels.arcade_level_end_results_labels,
        &labels.arcade_results_labels,
        &labels.rally_stages_labels,
        &labels.class_stage_results_labels,
        &labels.overall_stage_results_labels,
        &labels.class_rally_results_labels,
        &labels.overall_rally_results_labels,
        &labels.after_overall_rally_results_labels,
        &labels.second_after_overall_rally_results_labels,
        &labels.class_championship_standings_labels,
        &labels.championship_standings_labels,
        &labels.menu_74fea8_labels,
        &labels.championship_service_review_labels,
        &labels.championship_standings_review_labels,
        &labels.championship_races_review_labels,
        &labels.championship_end_review_labels,
        &labels.select_stage_labels,
        &labels.time_trial_select_stage_labels,
        &labels.next_stage_labels,
        &labels.view_stage_labels,
        &labels.view_courses_labels,
        &labels.labels_747f88,
        &labels.player1_controller_labels,
        &labels.player1_buttons_labels,
        &labels.player2_controller_labels,
        &labels.player2_buttons_labels,
        &labels.player3_controller_labels,
        &labels.player3_buttons_labels,
        &labels.player4_controller_labels,
        &labels.player4_buttons_labels,
        &labels.redefine_keys_player1_labels,
        &labels.redefine_keys_player2_labels,
        &labels.redefine_keys_player3_labels,
        &labels.redefine_keys_player4_labels,
        &labels.view_best_times_labels,
        &labels.view_best_scores_labels,
        &labels.best_times_labels,
        &labels.arcade_best_times_labels,
        &labels.time_trial_best_times_labels,
        &labels.arcade_best_scores_labels,
        &labels.best_class_scores_labels,
        &labels.best_overall_scores_labels,
        &labels.after_best_overall_scores_labels,
        &labels.options_labels,
        &labels.after_options_labels,
        &labels.player1_details_labels,
        &labels.player2_details_labels,
        &labels.player3_details_labels,
        &labels.player4_details_labels,
        &labels.player_details_labels,
        &labels.information_labels,
        &labels.load_championship_labels,
        &labels.save_championship_labels,
        &labels.load_replay_labels,
        &labels.save_replay_labels,
        &labels.labels_748230,
        &labels.save_setup_labels,
        &labels.visual_preferences_labels,
        &labels.visual_options_labels,
    ]
}
/// The number of label lists.
const LABEL_LIST_COUNT: usize = 98;
/// A label list's length.
const LABEL_LIST_LEN: usize = core::mem::size_of::<LabelList>();

const _: () = assert!(LABEL_LIST_LEN == 0x14);
const _: () = assert!(core::mem::offset_of!(MenuLabel, font) == 8);
const _: () = assert!(core::mem::offset_of!(LabelTables, visual_options_labels) == 97 * 0x14);
const _: () = assert!(core::mem::size_of::<LabelTables>() == 98 * LABEL_LIST_LEN);

/// Where the original kept the label lists (IDA 0x747AD8 ..): core's menu table names a menu's
/// labels by their address there.
const LABELS_FIRST_ADDRESS: usize = 0x747AD8;

/// The byte `offset` bytes on from the labels of `menu`, as an offset in the label lists
/// (`MenuTables::labels`, whose bytes the readers and writers are bounded to); None for a menu
/// without labels. For the original's slip of writing item 1 of the "menu" its labels would be
/// (`notes/bugs.md`): the place lands in the labels' memory.
pub fn labels_slot(menu: MenuId, offset: u32) -> Result<Option<usize>, crate::errors::BadInput> {
    let Some(at) = menu.labels_place()? else {
        return Ok(None);
    };
    Ok(Some(at.wrapping_add(offset) as usize))
}

/// The font of the label whose font field is `at` bytes into the label lists (`screens::
/// draw_menu_labels` walks them by offset). Refused: a place that is no label's font (the
/// walk gone off the lists' grid: the original would take what is there for a font object's
/// address).
pub fn label_font(
    labels: &LabelTables,
    at: u32,
) -> Result<crate::gfx::typefaces::FontId, crate::errors::BadInput> {
    let off = at as usize;
    let lists = label_lists(labels);
    if off >= lists.len() * LABEL_LIST_LEN || off % LABEL_LIST_LEN != 8 {
        return Err(crate::errors::BadInput("a label's font read off a label"));
    }
    Ok(lists[off / LABEL_LIST_LEN].label.font.get())
}

/// One item of a menu: its place in core's table (`fixed()`, never changed) and what the game
/// changes in play. The fields that change start as the table has them.
pub struct MenuItem {
    /// The menu's number and the item's in `MENUS` (kept as numbers: a reference made while
    /// the states are built at compile time points at a copy of the table, not at `MENUS`).
    menu: u16,
    item: u16,
    /// The string id (+0x14).
    pub string: PlainDword,
    /// The font (+0x18; `menu::disable_item` / `enable_item`).
    pub font: FontCell,
    /// +0x1C.
    pub behind: PlainDword,
    /// +0x20: the sprite behind it, or a button's sprite list (`menu::stage_view_steps::
    /// style_stage_item`).
    pub sprites: SpriteCell,
    /// +0x48: passed over when the moves up and down meet it.
    pub skip_mark: PlainDword,
    /// +0x50: the font a highlighted item pulses in (`style_stage_item`).
    pub highlight_font: FontCell,
}

impl MenuItem {
    /// The dword after a menu's last item, and an item's `x` that ends the walk.
    pub const END: u32 = 0xFFFF_FFFF;

    /// The item as core's table has it.
    pub fn fixed(&self) -> &'static Fixed {
        &MENUS[self.menu as usize].items[self.item as usize]
    }

    const fn of(menu: usize, item: usize) -> Self {
        let fixed = &MENUS[menu].items[item];
        Self {
            menu: menu as u16,
            item: item as u16,
            string: PlainDword::new(fixed.string),
            font: FontCell::new(font_of(fixed.font)),
            behind: PlainDword::new(fixed.behind),
            sprites: SpriteCell::at(fixed.sprites),
            skip_mark: PlainDword::new(fixed.skip_mark),
            highlight_font: FontCell::new(font_of(fixed.highlight_font)),
        }
    }
}

/// The font of the block a font the menus' table names is (its IDA address in a comment).
pub const fn font_of(font: MenuFont) -> FontId {
    match font {
        MenuFont::Measure => fonts::MEASURE,    // 0x51E824
        MenuFont::CaptionB => fonts::CAPTION_B, // 0x51E844
        MenuFont::ArcadeButtonPulse => fonts::ARCADE_BUTTON_PULSE, // 0x51E864
        MenuFont::StageItemB => fonts::STAGE_ITEM_B, // 0x51E884
        MenuFont::Title => fonts::TITLE,        // 0x51E8A4
        MenuFont::ItemPlain => fonts::ITEM_PLAIN, // 0x51E8E4
        MenuFont::BoxText => fonts::BOX_TEXT,   // 0x51E904
        MenuFont::Head => fonts::HEAD,          // 0x51E944
        MenuFont::CaptionA => fonts::CAPTION_A, // 0x51E964
        MenuFont::ArcadeButton => fonts::ARCADE_BUTTON, // 0x51E984
        MenuFont::StageTitle => fonts::STAGE_TITLE, // 0x51E9A4
        MenuFont::ItemGrey => fonts::ITEM_GREY, // 0x51EA04
    }
}

/// The number of items of all menus.
const ITEM_COUNT: usize = {
    let mut n = 0;
    let mut m = 0;
    while m < MENUS.len() {
        n += MENUS[m].items.len();
        m += 1;
    }
    n
};

/// Where each menu's items start in `MenuTables::items` (and one past the last).
const FIRST_ITEM: [u16; MENUS.len() + 1] = {
    let mut first = [0u16; MENUS.len() + 1];
    let mut m = 0;
    while m < MENUS.len() {
        first[m + 1] = first[m] + MENUS[m].items.len() as u16;
        m += 1;
    }
    first
};

const _: () = assert!(ITEM_COUNT == 567 && MENUS.len() == MENU_COUNT as usize);

/// The state of every item, in `MENUS`' order.
const fn item_states() -> [MenuItem; ITEM_COUNT] {
    let mut items = [const { MenuItem::of(0, 0) }; ITEM_COUNT];
    let mut k = 0;
    let mut m = 0;
    while m < MENUS.len() {
        let mut i = 0;
        while i < MENUS[m].items.len() {
            items[k] = MenuItem::of(m, i);
            k += 1;
            i += 1;
        }
        m += 1;
    }
    items
}

/// A menu's place among the menus' handles (`MenuTables`' fields name them).
#[derive(Clone, Copy, Debug)]
pub struct MenuHandle(u16);

impl MenuHandle {
    /// The menu (`MenuId`).
    pub const fn id(&self) -> MenuId {
        MenuId(self.0)
    }
}

/// The menus: the label lists, every item's run-time state and a handle per menu.
pub struct MenuTables {
    pub labels: LabelTables,
    pub items: [MenuItem; ITEM_COUNT],
    pub visual_preferences: MenuHandle,
    pub select_stage: MenuHandle,
    pub time_trial_select_stage: MenuHandle,
    pub time_trial_select_players: MenuHandle,
    pub simple_select_players: MenuHandle,
    pub main_menu: MenuHandle,
    pub arcade_select_level: MenuHandle,
    pub network_rally: MenuHandle,
    pub championship_new_simulation: MenuHandle,
    pub new_simulation: MenuHandle,
    pub player1_controller: MenuHandle,
    pub player1_buttons: MenuHandle,
    pub player2_controller: MenuHandle,
    pub player2_buttons: MenuHandle,
    pub player3_controller: MenuHandle,
    pub player3_buttons: MenuHandle,
    pub player4_controller: MenuHandle,
    pub player4_buttons: MenuHandle,
    pub car_specification: MenuHandle,
    pub slideshow: MenuHandle,
    pub select_car: MenuHandle,
    pub car_specs_display: MenuHandle,
    pub cspecs_menu: MenuHandle,
    pub redefine_keys_player1: MenuHandle,
    pub redefine_keys_player2: MenuHandle,
    pub redefine_keys_player3: MenuHandle,
    pub redefine_keys_player4: MenuHandle,
    pub audio_setup: MenuHandle,
    pub arcade_next_stage: MenuHandle,
    pub arcade_level_end_results: MenuHandle,
    pub arcade_results: MenuHandle,
    pub service_area: MenuHandle,
    pub next_stage: MenuHandle,
    pub view_stage: MenuHandle,
    pub rally_stages: MenuHandle,
    pub view_courses: MenuHandle,
    pub view_best_times: MenuHandle,
    pub best_times: MenuHandle,
    pub arcade_best_times: MenuHandle,
    pub time_trial_best_times: MenuHandle,
    pub arcade_best_scores: MenuHandle,
    pub best_class_scores: MenuHandle,
    pub best_overall_scores: MenuHandle,
    pub start_rally: MenuHandle,
    pub options: MenuHandle,
    pub player_details: MenuHandle,
    pub player1_details: MenuHandle,
    pub player2_details: MenuHandle,
    pub player3_details: MenuHandle,
    pub player4_details: MenuHandle,
    pub information: MenuHandle,
    pub class_stage_results: MenuHandle,
    pub overall_stage_results: MenuHandle,
    pub class_rally_results: MenuHandle,
    pub overall_rally_results: MenuHandle,
    pub section_times_menu: MenuHandle,
    pub class_championship_standings: MenuHandle,
    pub championship_standings: MenuHandle,
    pub championship_service_review: MenuHandle,
    pub championship_standings_review: MenuHandle,
    pub inspection_area: MenuHandle,
    pub welcome: MenuHandle,
    pub in_game_options: MenuHandle,
    pub stage_result: MenuHandle,
    pub single_rally_two_results: MenuHandle,
    pub single_rally_three_four_results: MenuHandle,
    pub time_trial_results: MenuHandle,
    pub single_rally_overall_results: MenuHandle,
    pub single_rally_review: MenuHandle,
    pub arcade_review_options: MenuHandle,
    pub arcade_next_level_review: MenuHandle,
    pub arcade_retry_review: MenuHandle,
    pub view_best_scores: MenuHandle,
    pub load_championship: MenuHandle,
    pub save_championship: MenuHandle,
    pub load_championship_table: MenuHandle,
    pub save_setup: MenuHandle,
    pub save_replay: MenuHandle,
    pub load_replay: MenuHandle,
    pub leave_question: MenuHandle,
    pub visual_options: MenuHandle,
    pub advanced_audio: MenuHandle,
    pub time_trial_stage_details: MenuHandle,
    pub championship_races_review: MenuHandle,
    pub championship_end_review: MenuHandle,
}

impl MenuTables {
    pub const fn new() -> Self {
        MenuTables {
            labels: LabelTables::new(),
            items: item_states(),
            visual_preferences: MenuHandle(0),
            select_stage: MenuHandle(1),
            time_trial_select_stage: MenuHandle(2),
            time_trial_select_players: MenuHandle(3),
            simple_select_players: MenuHandle(4),
            main_menu: MenuHandle(5),
            arcade_select_level: MenuHandle(6),
            network_rally: MenuHandle(7),
            championship_new_simulation: MenuHandle(10),
            new_simulation: MenuHandle(11),
            player1_controller: MenuHandle(12),
            player1_buttons: MenuHandle(13),
            player2_controller: MenuHandle(14),
            player2_buttons: MenuHandle(15),
            player3_controller: MenuHandle(16),
            player3_buttons: MenuHandle(17),
            player4_controller: MenuHandle(18),
            player4_buttons: MenuHandle(19),
            car_specification: MenuHandle(20),
            slideshow: MenuHandle(21),
            select_car: MenuHandle(22),
            car_specs_display: MenuHandle(23),
            cspecs_menu: MenuHandle(24),
            redefine_keys_player1: MenuHandle(26),
            redefine_keys_player2: MenuHandle(27),
            redefine_keys_player3: MenuHandle(28),
            redefine_keys_player4: MenuHandle(29),
            audio_setup: MenuHandle(30),
            arcade_next_stage: MenuHandle(31),
            arcade_level_end_results: MenuHandle(32),
            arcade_results: MenuHandle(33),
            service_area: MenuHandle(35),
            next_stage: MenuHandle(36),
            view_stage: MenuHandle(37),
            rally_stages: MenuHandle(38),
            view_courses: MenuHandle(39),
            view_best_times: MenuHandle(40),
            best_times: MenuHandle(41),
            arcade_best_times: MenuHandle(42),
            time_trial_best_times: MenuHandle(43),
            arcade_best_scores: MenuHandle(44),
            best_class_scores: MenuHandle(45),
            best_overall_scores: MenuHandle(46),
            start_rally: MenuHandle(48),
            options: MenuHandle(49),
            player_details: MenuHandle(51),
            player1_details: MenuHandle(52),
            player2_details: MenuHandle(53),
            player3_details: MenuHandle(54),
            player4_details: MenuHandle(55),
            information: MenuHandle(56),
            class_stage_results: MenuHandle(57),
            overall_stage_results: MenuHandle(58),
            class_rally_results: MenuHandle(59),
            overall_rally_results: MenuHandle(60),
            section_times_menu: MenuHandle(63),
            class_championship_standings: MenuHandle(64),
            championship_standings: MenuHandle(65),
            championship_service_review: MenuHandle(66),
            championship_standings_review: MenuHandle(67),
            inspection_area: MenuHandle(68),
            welcome: MenuHandle(69),
            in_game_options: MenuHandle(70),
            stage_result: MenuHandle(72),
            single_rally_two_results: MenuHandle(73),
            single_rally_three_four_results: MenuHandle(75),
            time_trial_results: MenuHandle(76),
            single_rally_overall_results: MenuHandle(77),
            single_rally_review: MenuHandle(78),
            arcade_review_options: MenuHandle(79),
            arcade_next_level_review: MenuHandle(80),
            arcade_retry_review: MenuHandle(81),
            view_best_scores: MenuHandle(82),
            load_championship: MenuHandle(83),
            save_championship: MenuHandle(84),
            load_championship_table: MenuHandle(85),
            save_setup: MenuHandle(86),
            save_replay: MenuHandle(87),
            load_replay: MenuHandle(88),
            leave_question: MenuHandle(89),
            visual_options: MenuHandle(90),
            advanced_audio: MenuHandle(91),
            time_trial_stage_details: MenuHandle(93),
            championship_races_review: MenuHandle(94),
            championship_end_review: MenuHandle(95),
        }
    }
}

/// A menu by handle: its number among `MENUS`' menus, in the image's
/// order. The current menu (`LOOP_FLAGS.menu`), the keys menu and the item calls hold
/// one in place of the menu's address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuId(u16);

/// The number of menus.
pub const MENU_COUNT: u16 = 96;

impl MenuId {
    fn table(self) -> &'static rc2k_core::menu::tables::Menu {
        &MENUS[self.0 as usize]
    }

    /// The offset of the menu's labels in the label lists; None for a menu without any.
    pub fn labels_place(self) -> Result<Option<u32>, crate::errors::BadInput> {
        match self.table().labels_at {
            0 => Ok(None),
            at if at as usize >= LABELS_FIRST_ADDRESS
                && ((at as usize - LABELS_FIRST_ADDRESS) < LABEL_LIST_COUNT * LABEL_LIST_LEN) =>
            {
                Ok(Some(at - LABELS_FIRST_ADDRESS as u32))
            }
            _ => Err(crate::errors::BadInput(
                "a menu's labels off the label lists",
            )),
        }
    }

    /// The menu's items (the -1 after them is `MenuItem::END`'s place: not among them).
    pub fn items(self, tables: &MenuTables) -> &[MenuItem] {
        let from = FIRST_ITEM[self.0 as usize] as usize;
        let to = FIRST_ITEM[self.0 as usize + 1] as usize;
        &tables.items[from..to]
    }

    /// Item `number` (from 1). Refused: item 0, for which the original counts through all
    /// 32 bits, and one past the menu's end (the original reads on into what follows).
    pub fn item(
        self,
        tables: &MenuTables,
        number: u32,
    ) -> Result<&MenuItem, crate::errors::BadInput> {
        if number == 0 {
            return Err(crate::errors::BadInput(
                "item number 0: the original counts through all 32 bits",
            ));
        }
        self.items(tables)
            .get(number as usize - 1)
            .ok_or(crate::errors::BadInput("an item past its menu's end"))
    }

    /// What a cell holds for it (0 is none).
    pub fn code(self) -> u32 {
        self.0 as u32 + 1
    }

    /// The menu a cell's `code` names.
    pub fn from_code(code: u32) -> Result<MenuId, crate::errors::BadInput> {
        match code {
            1..=0xFFFF if code - 1 < MENU_COUNT as u32 => Ok(MenuId((code - 1) as u16)),
            _ => Err(crate::errors::BadInput("no current menu")),
        }
    }
}

/// A menu pointer: none (0, as the image starts) or a menu.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct MenuCell(PlainDword);

impl MenuCell {
    pub const fn new() -> Self {
        MenuCell(PlainDword::new(0))
    }

    pub fn get(&self) -> Result<MenuId, crate::errors::BadInput> {
        MenuId::from_code(self.0.get())
    }

    pub fn set(&self, menu: MenuId) {
        self.0.set(menu.code());
    }
}

impl Default for MenuCell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menus_fonts_are_the_fonts_at_the_addresses_the_table_names() {
        for &f in MenuFont::ALL {
            assert_eq!(font_of(f).ida() as u32, f.ral(), "{f:?}");
        }
    }

    #[test]
    fn menu_labels_are_label_lists() {
        for n in 0..MENU_COUNT {
            let m = MenuId(n);
            assert_eq!(
                m.labels_place().unwrap().is_some(),
                m.table().labels_at != 0
            );
            if let Some(at) = m.labels_place().unwrap() {
                assert_eq!(at as usize % LABEL_LIST_LEN, 0, "menu {n}");
            }
        }
    }

    #[test]
    fn menus_by_handle() {
        let tables = &menus().tables.menus;
        let mut total = 0;
        for n in 0..MENU_COUNT {
            let m = MenuId(n);
            assert_eq!(MenuId::from_code(m.code()).unwrap(), m);
            assert_eq!(m.items(tables).len(), m.table().items.len());
            for (state, fixed) in m.items(tables).iter().zip(m.table().items) {
                assert!(std::ptr::eq(state.fixed(), fixed));
                assert_eq!(state.string.get(), fixed.string);
                assert_eq!(state.behind.get(), fixed.behind);
                assert_eq!(state.skip_mark.get(), fixed.skip_mark);
                assert_eq!(state.font.get().ida() as u32, fixed.font.ral());
                assert_eq!(
                    state.highlight_font.get().ida() as u32,
                    fixed.highlight_font.ral()
                );
            }
            total += m.items(tables).len();
        }
        assert_eq!(total, ITEM_COUNT);
        assert_eq!(
            tables.stage_result.id().items(tables).len(),
            MENUS[tables.stage_result.id().0 as usize].items.len()
        );
        assert!(MenuId::from_code(0).is_err());
        assert!(tables.select_stage.id().item(tables, 0).is_err());
        assert!(tables.select_stage.id().item(tables, 1).is_ok());
        let c = MenuCell::new();
        assert!(c.get().is_err());
        c.set(tables.service_area.id());
        assert_eq!(c.get().unwrap(), tables.service_area.id());
    }

    #[test]
    fn handles_follow_the_tables() {
        // Handle fields are in `MENUS`' order: the first and the last.
        let tables = &menus().tables.menus;
        assert_eq!(tables.visual_preferences.id().0, 0);
        assert_eq!(tables.championship_end_review.id().0, MENU_COUNT - 1);
    }
}
