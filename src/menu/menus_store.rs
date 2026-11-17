//! The menus: the 100 lists of labels as cells (the screens that draw a menu's labels, and a
//! 1999 slip that writes into them, walk them as a run of dwords, five to a list:
//! `LabelTables::dword`), and the 96 menus as handles into `rc2k_core::menu::tables::MENUS`,
//! the one source of the menus' items (their places, links, functions and starting values).
//!
//! What the game changes in play is kept here, per item (`MenuItem`, 566 of them in one
//! array, the menus' in `MENUS`' order): the string id (+0x14), the font (+0x18), +0x1C, the
//! sprite (+0x20), the font a highlighted item pulses in (+0x50) and the skip mark (+0x48),
//! each starting as the table has it. Nothing else of an item is written anywhere (the
//! compiler says so: the rest is read from `MenuItem::fixed`). Menus are named for the screen
//! that shows them (`menu::screen_loop`'s `Screen::menu`), the ones no screen names by their
//! handle in `MenuTables`.

use crate::gfx::typefaces::{FontCell, FontId, fonts};
use crate::memory::session::PlainDword;
use crate::menu::item_sprites_store::SpriteCell;
use rc2k_core::menu::tables::{MENUS, MenuFont, MenuItem as Fixed};

/// A label: x, y, the font, the string id.
#[derive(rc2k_derive::Cells)]
pub struct MenuLabel {
    pub x: PlainDword,
    pub y: PlainDword,
    /// The font.
    pub font: FontCell,
    /// The string id (+0x14).
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
#[derive(rc2k_derive::Cells)]
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

/// The label lists in the image's order.
#[derive(rc2k_derive::Cells)]
pub struct LabelTables {
    pub single_rally_review_labels: LabelList,
    pub arcade_review_options_labels: LabelList,
    pub arcade_next_level_review_labels: LabelList,
    pub arcade_retry_review_labels: LabelList,
    /// An unreferenced menu with no title and one "continue" item; named from that item.
    pub unused_continue_labels: LabelList,
    pub stage_result_labels: LabelList,
    /// No menu uses it; named from its title text (results).
    pub unused_results_title_labels: LabelList,
    pub single_rally_two_results_labels: LabelList,
    /// An unreferenced menu with the results title and a back item; named from its title text.
    pub unused_results_back_labels: LabelList,
    pub single_rally_three_four_results_labels: LabelList,
    pub single_rally_overall_results_labels: LabelList,
    pub time_trial_results_labels: LabelList,
    /// No menu uses it; named from its title text (the rally championship title).
    pub unused_rally_title_labels: LabelList,
    pub main_menu_labels: LabelList,
    pub network_rally_labels: LabelList,
    pub start_rally_labels: LabelList,
    pub championship_new_simulation_labels: LabelList,
    pub new_simulation_labels: LabelList,
    /// The "simulation" menu offering a new championship simulation or a loaded championship.
    pub championship_simulation_labels: LabelList,
    /// The "simulation" menu offering a new simulation.
    pub simulation_labels: LabelList,
    pub simple_select_players_labels: LabelList,
    pub time_trial_select_players_labels: LabelList,
    /// No menu uses it; named from its title text (select the number of players).
    pub unused_select_players_labels: LabelList,
    pub arcade_select_level_labels: LabelList,
    pub service_area_labels: LabelList,
    /// An unreferenced menu with no title and a back item, laid out like the service area.
    pub unused_service_area_back_labels: LabelList,
    pub in_game_options_labels: LabelList,
    /// The in-game options menu whose last item opens the visual preferences.
    pub in_game_options_visual_preferences_labels: LabelList,
    pub audio_setup_labels: LabelList,
    pub advanced_audio_labels: LabelList,
    pub leave_question_labels: LabelList,
    pub car_specification_labels: LabelList,
    pub slideshow_labels: LabelList,
    pub select_car_labels: LabelList,
    pub car_specs_display_labels: LabelList,
    pub cspecs_menu_labels: LabelList,
    pub inspection_area_labels: LabelList,
    pub welcome_labels: LabelList,
    pub arcade_next_stage_labels: LabelList,
    pub arcade_level_end_results_labels: LabelList,
    pub arcade_results_labels: LabelList,
    pub rally_stages_labels: LabelList,
    pub class_stage_results_labels: LabelList,
    pub overall_stage_results_labels: LabelList,
    pub class_rally_results_labels: LabelList,
    pub overall_rally_results_labels: LabelList,
    /// An unreferenced menu titled with the final class leaderboard, with a back item.
    pub final_class_leaderboard_labels: LabelList,
    /// An unreferenced menu titled with the final overall leaderboard, with a back item.
    pub final_overall_leaderboard_labels: LabelList,
    pub class_championship_standings_labels: LabelList,
    pub championship_standings_labels: LabelList,
    pub section_times_menu_labels: LabelList,
    pub championship_service_review_labels: LabelList,
    pub championship_standings_review_labels: LabelList,
    pub championship_races_review_labels: LabelList,
    pub championship_end_review_labels: LabelList,
    pub select_stage_labels: LabelList,
    pub time_trial_select_stage_labels: LabelList,
    pub next_stage_labels: LabelList,
    pub view_stage_labels: LabelList,
    pub view_courses_labels: LabelList,
    /// No menu uses it; named from its title text (course information).
    pub unused_view_courses_labels: LabelList,
    pub player1_controller_labels: LabelList,
    pub player1_buttons_labels: LabelList,
    pub player2_controller_labels: LabelList,
    pub player2_buttons_labels: LabelList,
    pub player3_controller_labels: LabelList,
    pub player3_buttons_labels: LabelList,
    pub player4_controller_labels: LabelList,
    pub player4_buttons_labels: LabelList,
    pub redefine_keys_player1_labels: LabelList,
    pub redefine_keys_player2_labels: LabelList,
    pub redefine_keys_player3_labels: LabelList,
    pub redefine_keys_player4_labels: LabelList,
    pub view_best_times_labels: LabelList,
    pub view_best_scores_labels: LabelList,
    pub best_times_labels: LabelList,
    pub arcade_best_times_labels: LabelList,
    pub time_trial_best_times_labels: LabelList,
    pub arcade_best_scores_labels: LabelList,
    pub best_class_scores_labels: LabelList,
    pub best_overall_scores_labels: LabelList,
    /// An unreferenced menu titled car setup (view stage, load or save the setup, continue).
    pub car_setup_labels: LabelList,
    pub options_labels: LabelList,
    /// An unreferenced menu titled video replay with a return item.
    pub video_replay_labels: LabelList,
    pub player1_details_labels: LabelList,
    pub player2_details_labels: LabelList,
    pub player3_details_labels: LabelList,
    pub player4_details_labels: LabelList,
    pub player_details_labels: LabelList,
    pub information_labels: LabelList,
    pub load_championship_labels: LabelList,
    pub save_championship_labels: LabelList,
    pub load_replay_labels: LabelList,
    pub save_replay_labels: LabelList,
    /// No menu uses it; its title text is empty.
    pub unused_empty_title_labels: LabelList,
    pub save_setup_labels: LabelList,
    pub visual_preferences_labels: LabelList,
    pub visual_options_labels: LabelList,
    /// The controller axes page's, past the image's 98.
    pub player_axes_labels: LabelList,
    /// One axis's page's, past those.
    pub player_axis_labels: LabelList,
}

#[rustfmt::skip]
impl LabelTables {
    pub const fn new() -> Self {
        LabelTables {
        single_rally_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_review_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_next_level_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        arcade_retry_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        unused_continue_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        stage_result_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        unused_results_title_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_two_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        unused_results_back_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_three_four_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        single_rally_overall_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x205),
        time_trial_results_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x204),
        unused_rally_title_labels: LabelList::new(0x310, 0x30, fonts::BOX_TEXT, 0x94),
        main_menu_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x12F),
        network_rally_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x147),
        start_rally_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1D7),
        championship_new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1EC),
        new_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1EC),
        championship_simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E0),
        simulation_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E0),
        simple_select_players_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        time_trial_select_players_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        unused_select_players_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1E5),
        arcade_select_level_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1F5),
        service_area_labels: LabelList::new(0x310, 0x1D, fonts::HEAD, 0x0),
        unused_service_area_back_labels: LabelList::new(0x310, 0x1D, fonts::HEAD, 0x0),
        in_game_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x521),
        in_game_options_visual_preferences_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x521),
        audio_setup_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x53B),
        advanced_audio_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x549),
        leave_question_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x53A),
        car_specification_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x21E),
        slideshow_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33C),
        select_car_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
        car_specs_display_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
        cspecs_menu_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1006),
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
        final_class_leaderboard_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4ED),
        final_overall_leaderboard_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4EE),
        class_championship_standings_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4F0),
        championship_standings_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4F1),
        section_times_menu_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x4EF),
        championship_service_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_standings_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_races_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        championship_end_review_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x514),
        select_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33F),
        time_trial_select_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x33F),
        next_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x357),
        view_stage_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x533),
        view_courses_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x343),
        unused_view_courses_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x343),
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
        car_setup_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x532),
        options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x13B),
        video_replay_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x1A3),
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
        unused_empty_title_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        save_setup_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x0),
        visual_preferences_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x369),
        visual_options_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x37F),
        player_axes_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        player_axis_labels: LabelList::new(0x310, 0x30, fonts::HEAD, 0x398),
        }
    }
}

/// The 100 label lists: the image's 98 in its order, then the axes pages' (the first 100 fields of
/// `LabelTables`, one after the other).
pub fn label_lists(labels: &LabelTables) -> [&LabelList; 100] {
    [
        &labels.single_rally_review_labels,
        &labels.arcade_review_options_labels,
        &labels.arcade_next_level_review_labels,
        &labels.arcade_retry_review_labels,
        &labels.unused_continue_labels,
        &labels.stage_result_labels,
        &labels.unused_results_title_labels,
        &labels.single_rally_two_results_labels,
        &labels.unused_results_back_labels,
        &labels.single_rally_three_four_results_labels,
        &labels.single_rally_overall_results_labels,
        &labels.time_trial_results_labels,
        &labels.unused_rally_title_labels,
        &labels.main_menu_labels,
        &labels.network_rally_labels,
        &labels.start_rally_labels,
        &labels.championship_new_simulation_labels,
        &labels.new_simulation_labels,
        &labels.championship_simulation_labels,
        &labels.simulation_labels,
        &labels.simple_select_players_labels,
        &labels.time_trial_select_players_labels,
        &labels.unused_select_players_labels,
        &labels.arcade_select_level_labels,
        &labels.service_area_labels,
        &labels.unused_service_area_back_labels,
        &labels.in_game_options_labels,
        &labels.in_game_options_visual_preferences_labels,
        &labels.audio_setup_labels,
        &labels.advanced_audio_labels,
        &labels.leave_question_labels,
        &labels.car_specification_labels,
        &labels.slideshow_labels,
        &labels.select_car_labels,
        &labels.car_specs_display_labels,
        &labels.cspecs_menu_labels,
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
        &labels.final_class_leaderboard_labels,
        &labels.final_overall_leaderboard_labels,
        &labels.class_championship_standings_labels,
        &labels.championship_standings_labels,
        &labels.section_times_menu_labels,
        &labels.championship_service_review_labels,
        &labels.championship_standings_review_labels,
        &labels.championship_races_review_labels,
        &labels.championship_end_review_labels,
        &labels.select_stage_labels,
        &labels.time_trial_select_stage_labels,
        &labels.next_stage_labels,
        &labels.view_stage_labels,
        &labels.view_courses_labels,
        &labels.unused_view_courses_labels,
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
        &labels.car_setup_labels,
        &labels.options_labels,
        &labels.video_replay_labels,
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
        &labels.unused_empty_title_labels,
        &labels.save_setup_labels,
        &labels.visual_preferences_labels,
        &labels.visual_options_labels,
        &labels.player_axes_labels,
        &labels.player_axis_labels,
    ]
}
impl LabelList {
    /// Dword `i` of the list (0 .. 5): x, y, the font's number, the string id, the end mark.
    fn dword(&self, i: usize) -> u32 {
        match i {
            0 => self.label.x.get(),
            1 => self.label.y.get(),
            2 => self.label.font.get().number(),
            3 => self.label.string.get(),
            _ => self.end.get(),
        }
    }

    /// Dword `i` set. The font cell holds a font's number: another value is refused (a 1999
    /// write reaches the end marks, never a font).
    fn set_dword(&self, i: usize, v: u32) -> Result<(), crate::errors::BadInput> {
        match i {
            0 => self.label.x.set(v),
            1 => self.label.y.set(v),
            2 => self.label.font.set(FontId::try_numbered(v as usize).ok_or(
                crate::errors::BadInput("a label's font set to a number that is no font"),
            )?),
            3 => self.label.string.set(v),
            _ => self.end.set(v),
        }
        Ok(())
    }
}

impl LabelTables {
    /// The dword at byte `at` of the lists laid end to end (any alignment, over the lists'
    /// borders), refused past the 100th list's end.
    pub fn dword(&self, at: usize) -> Result<u32, crate::errors::BadInput> {
        let lists = label_lists(self);
        let mut bytes = [0; 4];
        for (k, b) in bytes.iter_mut().enumerate() {
            *b = Self::byte_place(at, k)
                .and_then(|(list, i, byte)| Some(lists.get(list)?.dword(i).to_le_bytes()[byte]))
                .ok_or(crate::errors::BadInput("a label read past the label lists"))?;
        }
        Ok(u32::from_le_bytes(bytes))
    }

    /// `dword`'s write.
    pub fn set_dword(&self, at: usize, value: u32) -> Result<(), crate::errors::BadInput> {
        let lists = label_lists(self);
        let places: [_; 4] = std::array::from_fn(|k| Self::byte_place(at, k));
        if places
            .iter()
            .any(|p| p.is_none_or(|(list, ..)| list >= lists.len()))
        {
            return Err(crate::errors::BadInput(
                "a label write past the label lists",
            ));
        }
        for (k, place) in places.into_iter().enumerate() {
            let (list, i, byte) = place.unwrap_or_default();
            let mut word = lists[list].dword(i).to_le_bytes();
            word[byte] = value.to_le_bytes()[k];
            lists[list].set_dword(i, u32::from_le_bytes(word))?;
        }
        Ok(())
    }

    /// Byte `k` past `at`: its list, the dword of the list, and the byte of the dword.
    fn byte_place(at: usize, k: usize) -> Option<(usize, usize, usize)> {
        let byte = at.checked_add(k)?;
        let (list, in_list) = (byte / LABEL_LIST_LEN, byte % LABEL_LIST_LEN);
        Some((list, in_list / 4, in_list % 4))
    }
}

const LABEL_LIST_COUNT: usize = 100;
/// A label list's length.
const LABEL_LIST_LEN: usize = 4 * LABEL_LIST_DWORDS;

/// The dwords of a label list: x, y, the font, the string id, the end mark.
const LABEL_LIST_DWORDS: usize = 5;

const _: () = assert!(LABEL_LIST_DWORDS * 4 == 0x14);

/// The byte `offset` bytes on from the labels of `menu`, as an offset in the label lists
/// (`MenuTables::labels`, whose bytes the readers and writers are bounded to); None for a menu
/// without labels. A game slip kept as is: item 1 is written to the "menu" its labels would
/// be, a place that lands in the labels' memory.
pub fn labels_slot(menu: MenuId, offset: u32) -> Result<Option<usize>, crate::errors::BadInput> {
    let Some(at) = menu.labels_place()? else {
        return Ok(None);
    };
    Ok(Some(at.wrapping_add(offset) as usize))
}

/// The font of the label whose font field is `at` bytes into the label lists (`screens::
/// draw_menu_labels` walks them by offset). Refused: a place that is no label's font (the
/// walk gone off the lists' grid).
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
    /// How many pixels a page has moved the item up from its table place (0 for nearly every
    /// item; the axes list packs its rows).
    pub raised: PlainDword,
}

impl MenuItem {
    /// The item's y as drawn: its table place less what the page raised it by.
    pub fn y(&self) -> u32 {
        (self.fixed().y as u32).wrapping_sub(self.raised.get())
    }

    /// The y of the item's text as drawn, moved with the item.
    pub fn text_y(&self) -> u32 {
        (self.fixed().alignment_or_text_y as u32).wrapping_sub(self.raised.get())
    }

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
            sprites: SpriteCell::new(fixed.sprites),
            skip_mark: PlainDword::new(fixed.skip_mark),
            highlight_font: FontCell::new(font_of(fixed.highlight_font)),
            raised: PlainDword::new(0),
        }
    }
}

/// The font block of a font the menus' table names.
pub const fn font_of(font: MenuFont) -> FontId {
    match font {
        MenuFont::Measure => fonts::MEASURE,
        MenuFont::CaptionB => fonts::CAPTION_B,
        MenuFont::ArcadeButtonPulse => fonts::ARCADE_BUTTON_PULSE,
        MenuFont::StageItemB => fonts::STAGE_ITEM_B,
        MenuFont::Title => fonts::TITLE,
        MenuFont::ItemPlain => fonts::ITEM_PLAIN,
        MenuFont::BoxText => fonts::BOX_TEXT,
        MenuFont::Head => fonts::HEAD,
        MenuFont::CaptionA => fonts::CAPTION_A,
        MenuFont::ArcadeButton => fonts::ARCADE_BUTTON,
        MenuFont::StageTitle => fonts::STAGE_TITLE,
        MenuFont::ItemGrey => fonts::ITEM_GREY,
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

const _: () = assert!(ITEM_COUNT == 579 && MENUS.len() == MENU_COUNT as usize);

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
    pub player_axes: MenuHandle,
    pub player_axis: MenuHandle,
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
            player_axes: MenuHandle(96),
            player_axis: MenuHandle(97),
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
pub const MENU_COUNT: u16 = 98;

impl MenuId {
    fn table(self) -> &'static rc2k_core::menu::tables::Menu {
        &MENUS[self.0 as usize]
    }

    /// The offset of the menu's labels in the label lists; None for a menu without any.
    pub fn labels_place(self) -> Result<Option<u32>, crate::errors::BadInput> {
        match self.table().label_list {
            None => Ok(None),
            Some(list) if (list as usize) < LABEL_LIST_COUNT => {
                Ok(Some((list as usize * LABEL_LIST_LEN) as u32))
            }
            Some(_) => Err(crate::errors::BadInput(
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

    /// Item `number` (from 1). Refused: item 0 and one past the menu's end.
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

    /// The lists read and written as a run of dwords, five to a list: any alignment, over the
    /// lists' borders, refused past the last list.
    #[test]
    fn label_lists_read_and_write_as_dwords() {
        let _g = crate::test_support::test_lock();
        let labels = LabelTables::new();
        let lists = label_lists(&labels);
        let end = LABEL_LIST_COUNT * LABEL_LIST_LEN;
        let (x0, end0) = (lists[0].label.x.get(), lists[0].end.get());
        assert_eq!(labels.dword(0).unwrap(), x0);
        assert_eq!(labels.dword(0x10).unwrap(), end0);
        assert_eq!(labels.dword(0x14).unwrap(), lists[1].label.x.get());
        // Unaligned: the bytes of two dwords.
        let (a, b) = (lists[0].end.get(), lists[1].label.x.get());
        let want = u32::from_le_bytes([
            a.to_le_bytes()[2],
            a.to_le_bytes()[3],
            b.to_le_bytes()[0],
            b.to_le_bytes()[1],
        ]);
        assert_eq!(labels.dword(0x12).unwrap(), want);
        assert_eq!(labels.dword(2).unwrap(), {
            let (p, q) = (
                lists[0].label.x.get().to_le_bytes(),
                lists[0].label.y.get().to_le_bytes(),
            );
            u32::from_le_bytes([p[2], p[3], q[0], q[1]])
        });
        assert!(labels.dword(end - 4).is_ok() && labels.dword(end - 3).is_err());
        // A write to an end mark and back, a write past the end refused whole.
        labels.set_dword(0x10, 0x1234).unwrap();
        assert_eq!(lists[0].end.get(), 0x1234);
        labels.set_dword(0x10, end0).unwrap();
        assert!(labels.set_dword(end - 3, 1).is_err());
        assert_eq!(labels.dword(end - 4).unwrap(), lists[99].end.get());
        assert!(labels.set_dword(8, 99_999).is_err());
    }

    #[test]
    fn menu_labels_are_label_lists() {
        for n in 0..MENU_COUNT {
            let m = MenuId(n);
            assert_eq!(
                m.labels_place().unwrap().is_some(),
                m.table().label_list.is_some()
            );
            if let Some(at) = m.labels_place().unwrap() {
                assert_eq!(at as usize % LABEL_LIST_LEN, 0, "menu {n}");
            }
        }
    }

    #[test]
    fn menus_by_handle() {
        let tables = &crate::test_support::own::menus().tables.menus;
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
                assert_eq!(state.font.get(), font_of(fixed.font));
                assert_eq!(state.highlight_font.get(), font_of(fixed.highlight_font));
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
        let tables = &crate::test_support::own::menus().tables.menus;
        assert_eq!(tables.visual_preferences.id().0, 0);
        assert_eq!(tables.player_axis.id().0, MENU_COUNT - 1);
    }
}
