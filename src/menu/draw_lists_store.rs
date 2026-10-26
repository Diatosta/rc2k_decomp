//! The screens' draw lists, out of RAL.EXE's image (IDA 0x758B72 .. 0x7594AA; phase 7, typed):
//! the 392 lists of function addresses (code addresses in IDA's numbering, ended by -1) that
//! `menu::run_list` walks, in the image's order. A `#[repr(C)]` struct of atomic cells with the
//! image's layout, one static; `moved::MOVED` points the range at it. A list is named for
//! the screen and the pointer it is put in (`unread`, `draw`, `overlay`, `scene`, `setup`), or,
//! where several screens share it, for the pointers and its address. The last list of the
//! game's, `dword_7594AA`, is `menu::lists_store`'s. Generated from the image; the same data
//! is `rc2k_core::menu::tables::LISTS`.

use crate::session::Dword;

/// A list: `N` dwords, the last the -1 that ends it.
#[repr(C)]
pub struct DrawList<const N: usize> {
    pub entries: [Dword; N],
}

impl<const N: usize> DrawList<N> {
    pub const fn new(v: [u32; N]) -> Self {
        let mut entries = [const { Dword::new(0) }; N];
        let mut i = 0;
        while i < N {
            entries[i] = Dword::new(v[i]);
            i += 1;
        }
        Self { entries }
    }

    /// The list's address.
    pub fn addr(&self) -> usize {
        crate::pointers::host_number(self as *const Self)
    }
}

/// The screens' draw lists.
#[repr(C)]
pub struct DrawLists {
    /// List at IDA 0x758B72 (screens: arcade_best_times, arcade_results, audio_setup, best_class_scores ..): 0 entries and the -1 that ends it.
    pub unread_758b72: DrawList<1>,
    /// List at IDA 0x758B76 (the draw list of `main_menu`): 2 entries and the -1 that ends it.
    pub main_menu_draw: DrawList<3>,
    /// List at IDA 0x758B82 (the draw list of `start_rally`): 2 entries and the -1 that ends it.
    pub start_rally_draw: DrawList<3>,
    /// List at IDA 0x758B8E (the draw list of `simple_select_players`): 2 entries and the -1 that ends it.
    pub simple_select_players_draw: DrawList<3>,
    /// List at IDA 0x758B9A (named by no screen): 2 entries and the -1 that ends it.
    pub list_758b9a: DrawList<3>,
    /// List at IDA 0x758BA6 (screens: load_championship): 3 entries and the -1 that ends it.
    pub draw_758ba6: DrawList<4>,
    /// List at IDA 0x758BB6 (the draw list of `save_setup`): 0 entries and the -1 that ends it.
    pub save_setup_draw: DrawList<1>,
    /// List at IDA 0x758BBA (the draw list of `load_championship_46e899`): 0 entries and the -1 that ends it.
    pub load_championship_46e899_draw: DrawList<1>,
    /// List at IDA 0x758BBE (screens: save_championship): 3 entries and the -1 that ends it.
    pub draw_758bbe: DrawList<4>,
    /// List at IDA 0x758BCE (screens: save_replay): 1 entry and the -1 that ends it.
    pub draw_758bce: DrawList<2>,
    /// List at IDA 0x758BD6 (screens: load_replay): 1 entry and the -1 that ends it.
    pub draw_758bd6: DrawList<2>,
    /// List at IDA 0x758BDE (named by no screen): 2 entries and the -1 that ends it.
    pub list_758bde: DrawList<3>,
    /// List at IDA 0x758BEA (the draw list of `arcade_select_level`): 2 entries and the -1 that ends it.
    pub arcade_select_level_draw: DrawList<3>,
    /// List at IDA 0x758BF6 (named by no screen): 2 entries and the -1 that ends it.
    pub list_758bf6: DrawList<3>,
    /// List at IDA 0x758C02 (the draw list of `time_trial_select_players`): 2 entries and the -1 that ends it.
    pub time_trial_select_players_draw: DrawList<3>,
    /// List at IDA 0x758C0E (the draw list of `championship_new_simulation`): 2 entries and the -1 that ends it.
    pub championship_new_simulation_draw: DrawList<3>,
    /// List at IDA 0x758C1A (the draw list of `new_simulation`): 2 entries and the -1 that ends it.
    pub new_simulation_draw: DrawList<3>,
    /// List at IDA 0x758C26 (the draw list of `network_rally`): 2 entries and the -1 that ends it.
    pub network_rally_draw: DrawList<3>,
    /// List at IDA 0x758C32 (the draw list of `select_car`): 3 entries and the -1 that ends it.
    pub select_car_draw: DrawList<4>,
    /// List at IDA 0x758C42 (the draw list of `car_specs_47af4c`): 3 entries and the -1 that ends it.
    pub car_specs_47af4c_draw: DrawList<4>,
    /// List at IDA 0x758C52 (named by no screen): 3 entries and the -1 that ends it.
    pub list_758c52: DrawList<4>,
    /// List at IDA 0x758C62 (named by no screen): 1 entry and the -1 that ends it.
    pub list_758c62: DrawList<2>,
    /// List at IDA 0x758C6A (the draw list of `arcade_next_stage`): 1 entry and the -1 that ends it.
    pub arcade_next_stage_draw: DrawList<2>,
    /// List at IDA 0x758C72 (the draw list of `options`): 0 entries and the -1 that ends it.
    pub options_draw: DrawList<1>,
    /// List at IDA 0x758C76 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758c76: DrawList<1>,
    /// List at IDA 0x758C7A (the draw list of `player_details`): 0 entries and the -1 that ends it.
    pub player_details_draw: DrawList<1>,
    /// List at IDA 0x758C7E (named by no screen): 0 entries and the -1 that ends it.
    pub list_758c7e: DrawList<1>,
    /// List at IDA 0x758C82 (the draw list of `information`): 0 entries and the -1 that ends it.
    pub information_draw: DrawList<1>,
    /// List at IDA 0x758C86 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758c86: DrawList<1>,
    /// List at IDA 0x758C8A (named by no screen): 0 entries and the -1 that ends it.
    pub list_758c8a: DrawList<1>,
    /// List at IDA 0x758C8E (screens: audio_setup): 1 entry and the -1 that ends it.
    pub draw_758c8e: DrawList<2>,
    /// List at IDA 0x758C96 (the draw list of `advanced_audio`): 1 entry and the -1 that ends it.
    pub advanced_audio_draw: DrawList<2>,
    /// List at IDA 0x758C9E (screens: leave_question): 2 entries and the -1 that ends it.
    pub draw_758c9e: DrawList<3>,
    /// List at IDA 0x758CAA (screens: visual_preferences): 1 entry and the -1 that ends it.
    pub draw_758caa: DrawList<2>,
    /// List at IDA 0x758CB2 (the draw list of `visual_options`): 1 entry and the -1 that ends it.
    pub visual_options_draw: DrawList<2>,
    /// List at IDA 0x758CBA (screens: player1_details): 1 entry and the -1 that ends it.
    pub draw_758cba: DrawList<2>,
    /// List at IDA 0x758CC2 (screens: player2_details): 1 entry and the -1 that ends it.
    pub draw_758cc2: DrawList<2>,
    /// List at IDA 0x758CCA (screens: player3_details): 1 entry and the -1 that ends it.
    pub draw_758cca: DrawList<2>,
    /// List at IDA 0x758CD2 (screens: player4_details): 1 entry and the -1 that ends it.
    pub draw_758cd2: DrawList<2>,
    /// List at IDA 0x758CDA (the draw list of `car_specification`): 3 entries and the -1 that ends it.
    pub car_specification_draw: DrawList<4>,
    /// List at IDA 0x758CEA (the draw list of `slideshow`): 2 entries and the -1 that ends it.
    pub slideshow_draw: DrawList<3>,
    /// List at IDA 0x758CF6 (the draw list of `view_courses`): 1 entry and the -1 that ends it.
    pub view_courses_draw: DrawList<2>,
    /// List at IDA 0x758CFE (the draw list of `time_trial_stage_details`): 1 entry and the -1 that ends it.
    pub time_trial_stage_details_draw: DrawList<2>,
    /// List at IDA 0x758D06 (the draw list of `view_best_times`): 1 entry and the -1 that ends it.
    pub view_best_times_draw: DrawList<2>,
    /// List at IDA 0x758D0E (the draw list of `view_best_scores`): 2 entries and the -1 that ends it.
    pub view_best_scores_draw: DrawList<3>,
    /// List at IDA 0x758D1A (the draw list of `best_times`): 1 entry and the -1 that ends it.
    pub best_times_draw: DrawList<2>,
    /// List at IDA 0x758D22 (screens: arcade_best_times): 1 entry and the -1 that ends it.
    pub draw_758d22: DrawList<2>,
    /// List at IDA 0x758D2A (screens: time_trial_best_times): 1 entry and the -1 that ends it.
    pub draw_758d2a: DrawList<2>,
    /// List at IDA 0x758D32 (screens: best_scores_48a69e): 1 entry and the -1 that ends it.
    pub draw_758d32: DrawList<2>,
    /// List at IDA 0x758D3A (screens: best_class_scores): 1 entry and the -1 that ends it.
    pub draw_758d3a: DrawList<2>,
    /// List at IDA 0x758D42 (screens: best_overall_scores): 1 entry and the -1 that ends it.
    pub draw_758d42: DrawList<2>,
    /// List at IDA 0x758D4A (named by no screen): 0 entries and the -1 that ends it.
    pub list_758d4a: DrawList<1>,
    /// List at IDA 0x758D4E (the draw list of `welcome`): 0 entries and the -1 that ends it.
    pub welcome_draw: DrawList<1>,
    /// List at IDA 0x758D52 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758d52: DrawList<1>,
    /// List at IDA 0x758D56 (named by no screen): 1 entry and the -1 that ends it.
    pub list_758d56: DrawList<2>,
    /// List at IDA 0x758D5E (the draw list of `review_options_47d5a5`): 0 entries and the -1 that ends it.
    pub review_options_47d5a5_draw: DrawList<1>,
    /// List at IDA 0x758D62 (the draw list of `review_options_480d46`): 0 entries and the -1 that ends it.
    pub review_options_480d46_draw: DrawList<1>,
    /// List at IDA 0x758D66 (the draw list of `review_options_48108e`): 0 entries and the -1 that ends it.
    pub review_options_48108e_draw: DrawList<1>,
    /// List at IDA 0x758D6A (the draw list of `review_options_481429`): 0 entries and the -1 that ends it.
    pub review_options_481429_draw: DrawList<1>,
    /// List at IDA 0x758D6E (the draw list of `service_area`): 1 entry and the -1 that ends it.
    pub service_area_draw: DrawList<2>,
    /// List at IDA 0x758D76 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758d76: DrawList<1>,
    /// List at IDA 0x758D7A (named by no screen): 0 entries and the -1 that ends it.
    pub list_758d7a: DrawList<1>,
    /// List at IDA 0x758D7E (the draw list of `next_stage`): 1 entry and the -1 that ends it.
    pub next_stage_draw: DrawList<2>,
    /// List at IDA 0x758D86 (the draw list of `view_stage`): 1 entry and the -1 that ends it.
    pub view_stage_draw: DrawList<2>,
    /// List at IDA 0x758D8E (screens: player1_controller): 1 entry and the -1 that ends it.
    pub draw_758d8e: DrawList<2>,
    /// List at IDA 0x758D96 (screens: player1_buttons): 2 entries and the -1 that ends it.
    pub draw_758d96: DrawList<3>,
    /// List at IDA 0x758DA2 (screens: player2_controller): 1 entry and the -1 that ends it.
    pub draw_758da2: DrawList<2>,
    /// List at IDA 0x758DAA (screens: player2_buttons): 2 entries and the -1 that ends it.
    pub draw_758daa: DrawList<3>,
    /// List at IDA 0x758DB6 (screens: player3_controller): 1 entry and the -1 that ends it.
    pub draw_758db6: DrawList<2>,
    /// List at IDA 0x758DBE (screens: player3_buttons): 2 entries and the -1 that ends it.
    pub draw_758dbe: DrawList<3>,
    /// List at IDA 0x758DCA (screens: player4_controller): 1 entry and the -1 that ends it.
    pub draw_758dca: DrawList<2>,
    /// List at IDA 0x758DD2 (screens: player4_buttons): 2 entries and the -1 that ends it.
    pub draw_758dd2: DrawList<3>,
    /// List at IDA 0x758DDE (the draw list of `inspection_area`): 0 entries and the -1 that ends it.
    pub inspection_area_draw: DrawList<1>,
    /// List at IDA 0x758DE2 (screens: redefine_keys_player1): 2 entries and the -1 that ends it.
    pub draw_758de2: DrawList<3>,
    /// List at IDA 0x758DEE (screens: redefine_keys_player2): 2 entries and the -1 that ends it.
    pub draw_758dee: DrawList<3>,
    /// List at IDA 0x758DFA (screens: redefine_keys_player3): 2 entries and the -1 that ends it.
    pub draw_758dfa: DrawList<3>,
    /// List at IDA 0x758E06 (screens: redefine_keys_player4): 2 entries and the -1 that ends it.
    pub draw_758e06: DrawList<3>,
    /// List at IDA 0x758E12 (named by no screen): 2 entries and the -1 that ends it.
    pub list_758e12: DrawList<3>,
    /// List at IDA 0x758E1E (named by no screen): 2 entries and the -1 that ends it.
    pub list_758e1e: DrawList<3>,
    /// List at IDA 0x758E2A (named by no screen): 2 entries and the -1 that ends it.
    pub list_758e2a: DrawList<3>,
    /// List at IDA 0x758E36 (named by no screen): 2 entries and the -1 that ends it.
    pub list_758e36: DrawList<3>,
    /// List at IDA 0x758E42 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758e42: DrawList<1>,
    /// List at IDA 0x758E46 (named by no screen): 1 entry and the -1 that ends it.
    pub list_758e46: DrawList<2>,
    /// List at IDA 0x758E4E (named by no screen): 1 entry and the -1 that ends it.
    pub list_758e4e: DrawList<2>,
    /// List at IDA 0x758E56 (the draw list of `class_championship_standings`): 1 entry and the -1 that ends it.
    pub class_championship_standings_draw: DrawList<2>,
    /// List at IDA 0x758E5E (named by no screen): 1 entry and the -1 that ends it.
    pub list_758e5e: DrawList<2>,
    /// List at IDA 0x758E66 (the draw list of `screen_488f4c`): 1 entry and the -1 that ends it.
    pub screen_488f4c_draw: DrawList<2>,
    /// List at IDA 0x758E6E (named by no screen): 1 entry and the -1 that ends it.
    pub list_758e6e: DrawList<2>,
    /// List at IDA 0x758E76 (screens: arcade_results): 1 entry and the -1 that ends it.
    pub draw_758e76: DrawList<2>,
    /// List at IDA 0x758E7E (screens: stage_result): 1 entry and the -1 that ends it.
    pub draw_758e7e: DrawList<2>,
    /// List at IDA 0x758E86 (screens: results_48584b): 1 entry and the -1 that ends it.
    pub draw_758e86: DrawList<2>,
    /// List at IDA 0x758E8E (screens: results_482e9b): 1 entry and the -1 that ends it.
    pub draw_758e8e: DrawList<2>,
    /// List at IDA 0x758E96 (screens: time_trial_results): 1 entry and the -1 that ends it.
    pub draw_758e96: DrawList<2>,
    /// List at IDA 0x758E9E (the draw list of `select_stage`): 1 entry and the -1 that ends it.
    pub select_stage_draw: DrawList<2>,
    /// List at IDA 0x758EA6 (the draw list of `time_trial_select_stage`): 1 entry and the -1 that ends it.
    pub time_trial_select_stage_draw: DrawList<2>,
    /// List at IDA 0x758EAE (named by no screen): 2 entries and the -1 that ends it.
    pub list_758eae: DrawList<3>,
    /// List at IDA 0x758EBA (the draw list of `review_options_485f5e`): 0 entries and the -1 that ends it.
    pub review_options_485f5e_draw: DrawList<1>,
    /// List at IDA 0x758EBE (the draw list of `review_options_4882e2`): 0 entries and the -1 that ends it.
    pub review_options_4882e2_draw: DrawList<1>,
    /// List at IDA 0x758EC2 (the draw list of `review_options_48861d`): 0 entries and the -1 that ends it.
    pub review_options_48861d_draw: DrawList<1>,
    /// List at IDA 0x758EC6 (the draw list of `arcade_review_options`): 0 entries and the -1 that ends it.
    pub arcade_review_options_draw: DrawList<1>,
    /// List at IDA 0x758ECA (named by no screen): 0 entries and the -1 that ends it.
    pub list_758eca: DrawList<1>,
    /// List at IDA 0x758ECE (screens: screen_483163): 1 entry and the -1 that ends it.
    pub draw_758ece: DrawList<2>,
    /// List at IDA 0x758ED6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758ed6: DrawList<1>,
    /// List at IDA 0x758EDA (named by no screen): 2 entries and the -1 that ends it.
    pub list_758eda: DrawList<3>,
    /// List at IDA 0x758EE6 (the scene list of `main_menu`): 1 entry and the -1 that ends it.
    pub main_menu_scene: DrawList<2>,
    /// List at IDA 0x758EEE (the scene list of `start_rally`): 1 entry and the -1 that ends it.
    pub start_rally_scene: DrawList<2>,
    /// List at IDA 0x758EF6 (the scene list of `simple_select_players`): 1 entry and the -1 that ends it.
    pub simple_select_players_scene: DrawList<2>,
    /// List at IDA 0x758EFE (the scene list of `arcade_select_level`): 1 entry and the -1 that ends it.
    pub arcade_select_level_scene: DrawList<2>,
    /// List at IDA 0x758F06 (named by no screen): 1 entry and the -1 that ends it.
    pub list_758f06: DrawList<2>,
    /// List at IDA 0x758F0E (the scene list of `load_championship`): 0 entries and the -1 that ends it.
    pub load_championship_scene: DrawList<1>,
    /// List at IDA 0x758F12 (the scene list of `save_setup`): 1 entry and the -1 that ends it.
    pub save_setup_scene: DrawList<2>,
    /// List at IDA 0x758F1A (the scene list of `load_championship_46e899`): 1 entry and the -1 that ends it.
    pub load_championship_46e899_scene: DrawList<2>,
    /// List at IDA 0x758F22 (the scene list of `save_championship`): 0 entries and the -1 that ends it.
    pub save_championship_scene: DrawList<1>,
    /// List at IDA 0x758F26 (the scene list of `save_replay`): 0 entries and the -1 that ends it.
    pub save_replay_scene: DrawList<1>,
    /// List at IDA 0x758F2A (the scene list of `load_replay`): 0 entries and the -1 that ends it.
    pub load_replay_scene: DrawList<1>,
    /// List at IDA 0x758F2E (named by no screen): 1 entry and the -1 that ends it.
    pub list_758f2e: DrawList<2>,
    /// List at IDA 0x758F36 (named by no screen): 1 entry and the -1 that ends it.
    pub list_758f36: DrawList<2>,
    /// List at IDA 0x758F3E (the scene list of `time_trial_select_players`): 1 entry and the -1 that ends it.
    pub time_trial_select_players_scene: DrawList<2>,
    /// List at IDA 0x758F46 (the scene list of `championship_new_simulation`): 1 entry and the -1 that ends it.
    pub championship_new_simulation_scene: DrawList<2>,
    /// List at IDA 0x758F4E (the scene list of `new_simulation`): 1 entry and the -1 that ends it.
    pub new_simulation_scene: DrawList<2>,
    /// List at IDA 0x758F56 (the scene list of `network_rally`): 1 entry and the -1 that ends it.
    pub network_rally_scene: DrawList<2>,
    /// List at IDA 0x758F5E (the scene list of `select_car`): 1 entry and the -1 that ends it.
    pub select_car_scene: DrawList<2>,
    /// List at IDA 0x758F66 (the scene list of `car_specs_47af4c`): 1 entry and the -1 that ends it.
    pub car_specs_47af4c_scene: DrawList<2>,
    /// List at IDA 0x758F6E (named by no screen): 1 entry and the -1 that ends it.
    pub list_758f6e: DrawList<2>,
    /// List at IDA 0x758F76 (the scene list of `car_specification`): 1 entry and the -1 that ends it.
    pub car_specification_scene: DrawList<2>,
    /// List at IDA 0x758F7E (the scene list of `slideshow`): 0 entries and the -1 that ends it.
    pub slideshow_scene: DrawList<1>,
    /// List at IDA 0x758F82 (the scene list of `screen_483163`): 0 entries and the -1 that ends it.
    pub screen_483163_scene: DrawList<1>,
    /// List at IDA 0x758F86 (the scene list of `results_48584b`): 0 entries and the -1 that ends it.
    pub results_48584b_scene: DrawList<1>,
    /// List at IDA 0x758F8A (the scene list of `review_options_485f5e`): 1 entry and the -1 that ends it.
    pub review_options_485f5e_scene: DrawList<2>,
    /// List at IDA 0x758F92 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758f92: DrawList<1>,
    /// List at IDA 0x758F96 (the scene list of `review_options_4882e2`): 1 entry and the -1 that ends it.
    pub review_options_4882e2_scene: DrawList<2>,
    /// List at IDA 0x758F9E (the scene list of `review_options_48861d`): 1 entry and the -1 that ends it.
    pub review_options_48861d_scene: DrawList<2>,
    /// List at IDA 0x758FA6 (the scene list of `arcade_review_options`): 1 entry and the -1 that ends it.
    pub arcade_review_options_scene: DrawList<2>,
    /// List at IDA 0x758FAE (named by no screen): 0 entries and the -1 that ends it.
    pub list_758fae: DrawList<1>,
    /// List at IDA 0x758FB2 (the scene list of `best_scores_48a69e`): 0 entries and the -1 that ends it.
    pub best_scores_48a69e_scene: DrawList<1>,
    /// List at IDA 0x758FB6 (the scene list of `best_class_scores`): 0 entries and the -1 that ends it.
    pub best_class_scores_scene: DrawList<1>,
    /// List at IDA 0x758FBA (the scene list of `best_overall_scores`): 0 entries and the -1 that ends it.
    pub best_overall_scores_scene: DrawList<1>,
    /// List at IDA 0x758FBE (the scene list of `time_trial_select_stage`): 0 entries and the -1 that ends it.
    pub time_trial_select_stage_scene: DrawList<1>,
    /// List at IDA 0x758FC2 (the scene list of `time_trial_best_times`): 0 entries and the -1 that ends it.
    pub time_trial_best_times_scene: DrawList<1>,
    /// List at IDA 0x758FC6 (the scene list of `time_trial_results`): 0 entries and the -1 that ends it.
    pub time_trial_results_scene: DrawList<1>,
    /// List at IDA 0x758FCA (the scene list of `view_best_times`): 0 entries and the -1 that ends it.
    pub view_best_times_scene: DrawList<1>,
    /// List at IDA 0x758FCE (the scene list of `view_best_scores`): 0 entries and the -1 that ends it.
    pub view_best_scores_scene: DrawList<1>,
    /// List at IDA 0x758FD2 (the scene list of `best_times`): 0 entries and the -1 that ends it.
    pub best_times_scene: DrawList<1>,
    /// List at IDA 0x758FD6 (the scene list of `arcade_next_stage`): 0 entries and the -1 that ends it.
    pub arcade_next_stage_scene: DrawList<1>,
    /// List at IDA 0x758FDA (named by no screen): 0 entries and the -1 that ends it.
    pub list_758fda: DrawList<1>,
    /// List at IDA 0x758FDE (the scene list of `options`): 1 entry and the -1 that ends it.
    pub options_scene: DrawList<2>,
    /// List at IDA 0x758FE6 (the scene list of `player_details`): 1 entry and the -1 that ends it.
    pub player_details_scene: DrawList<2>,
    /// List at IDA 0x758FEE (the scene list of `information`): 1 entry and the -1 that ends it.
    pub information_scene: DrawList<2>,
    /// List at IDA 0x758FF6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_758ff6: DrawList<1>,
    /// List at IDA 0x758FFA (the scene list of `audio_setup`): 0 entries and the -1 that ends it.
    pub audio_setup_scene: DrawList<1>,
    /// List at IDA 0x758FFE (the scene list of `advanced_audio`): 0 entries and the -1 that ends it.
    pub advanced_audio_scene: DrawList<1>,
    /// List at IDA 0x759002 (the scene list of `leave_question`): 0 entries and the -1 that ends it.
    pub leave_question_scene: DrawList<1>,
    /// List at IDA 0x759006 (the scene list of `visual_preferences`): 0 entries and the -1 that ends it.
    pub visual_preferences_scene: DrawList<1>,
    /// List at IDA 0x75900A (the scene list of `visual_options`): 0 entries and the -1 that ends it.
    pub visual_options_scene: DrawList<1>,
    /// List at IDA 0x75900E (the scene list of `player1_details`): 0 entries and the -1 that ends it.
    pub player1_details_scene: DrawList<1>,
    /// List at IDA 0x759012 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759012: DrawList<1>,
    /// List at IDA 0x759016 (the scene list of `player2_details`): 0 entries and the -1 that ends it.
    pub player2_details_scene: DrawList<1>,
    /// List at IDA 0x75901A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75901a: DrawList<1>,
    /// List at IDA 0x75901E (the scene list of `player3_details`): 0 entries and the -1 that ends it.
    pub player3_details_scene: DrawList<1>,
    /// List at IDA 0x759022 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759022: DrawList<1>,
    /// List at IDA 0x759026 (the scene list of `player4_details`): 0 entries and the -1 that ends it.
    pub player4_details_scene: DrawList<1>,
    /// List at IDA 0x75902A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75902a: DrawList<1>,
    /// List at IDA 0x75902E (the scene list of `view_courses`): 0 entries and the -1 that ends it.
    pub view_courses_scene: DrawList<1>,
    /// List at IDA 0x759032 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759032: DrawList<1>,
    /// List at IDA 0x759036 (the scene list of `time_trial_stage_details`): 0 entries and the -1 that ends it.
    pub time_trial_stage_details_scene: DrawList<1>,
    /// List at IDA 0x75903A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75903a: DrawList<1>,
    /// List at IDA 0x75903E (the scene list of `arcade_best_times`): 0 entries and the -1 that ends it.
    pub arcade_best_times_scene: DrawList<1>,
    /// List at IDA 0x759042 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759042: DrawList<1>,
    /// List at IDA 0x759046 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759046: DrawList<1>,
    /// List at IDA 0x75904A (the scene list of `in_game_options`): 1 entry and the -1 that ends it.
    pub in_game_options_scene: DrawList<2>,
    /// List at IDA 0x759052 (the scene list of `select_stage`): 0 entries and the -1 that ends it.
    pub select_stage_scene: DrawList<1>,
    /// List at IDA 0x759056 (the scene list of `results_482e9b`): 0 entries and the -1 that ends it.
    pub results_482e9b_scene: DrawList<1>,
    /// List at IDA 0x75905A (the scene list of `welcome`): 0 entries and the -1 that ends it.
    pub welcome_scene: DrawList<1>,
    /// List at IDA 0x75905E (the scene list of `rally_stages`): 0 entries and the -1 that ends it.
    pub rally_stages_scene: DrawList<1>,
    /// List at IDA 0x759062 (the scene list of `review_options_47d5a5`): 1 entry and the -1 that ends it.
    pub review_options_47d5a5_scene: DrawList<2>,
    /// List at IDA 0x75906A (the scene list of `review_options_480d46`): 1 entry and the -1 that ends it.
    pub review_options_480d46_scene: DrawList<2>,
    /// List at IDA 0x759072 (the scene list of `review_options_48108e`): 1 entry and the -1 that ends it.
    pub review_options_48108e_scene: DrawList<2>,
    /// List at IDA 0x75907A (the scene list of `review_options_481429`): 1 entry and the -1 that ends it.
    pub review_options_481429_scene: DrawList<2>,
    /// List at IDA 0x759082 (the scene list of `service_area`): 1 entry and the -1 that ends it.
    pub service_area_scene: DrawList<2>,
    /// List at IDA 0x75908A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75908a: DrawList<1>,
    /// List at IDA 0x75908E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75908e: DrawList<1>,
    /// List at IDA 0x759092 (the scene list of `next_stage`): 0 entries and the -1 that ends it.
    pub next_stage_scene: DrawList<1>,
    /// List at IDA 0x759096 (the scene list of `view_stage`): 0 entries and the -1 that ends it.
    pub view_stage_scene: DrawList<1>,
    /// List at IDA 0x75909A (the scene list of `player1_controller`): 0 entries and the -1 that ends it.
    pub player1_controller_scene: DrawList<1>,
    /// List at IDA 0x75909E (the scene list of `player1_buttons`): 0 entries and the -1 that ends it.
    pub player1_buttons_scene: DrawList<1>,
    /// List at IDA 0x7590A2 (the scene list of `player2_controller`): 0 entries and the -1 that ends it.
    pub player2_controller_scene: DrawList<1>,
    /// List at IDA 0x7590A6 (the scene list of `player2_buttons`): 0 entries and the -1 that ends it.
    pub player2_buttons_scene: DrawList<1>,
    /// List at IDA 0x7590AA (the scene list of `player3_controller`): 0 entries and the -1 that ends it.
    pub player3_controller_scene: DrawList<1>,
    /// List at IDA 0x7590AE (the scene list of `player3_buttons`): 0 entries and the -1 that ends it.
    pub player3_buttons_scene: DrawList<1>,
    /// List at IDA 0x7590B2 (the scene list of `player4_controller`): 0 entries and the -1 that ends it.
    pub player4_controller_scene: DrawList<1>,
    /// List at IDA 0x7590B6 (the scene list of `player4_buttons`): 0 entries and the -1 that ends it.
    pub player4_buttons_scene: DrawList<1>,
    /// List at IDA 0x7590BA (the scene list of `inspection_area`): 1 entry and the -1 that ends it.
    pub inspection_area_scene: DrawList<2>,
    /// List at IDA 0x7590C2 (the scene list of `redefine_keys_player1`): 0 entries and the -1 that ends it.
    pub redefine_keys_player1_scene: DrawList<1>,
    /// List at IDA 0x7590C6 (the scene list of `redefine_keys_player2`): 0 entries and the -1 that ends it.
    pub redefine_keys_player2_scene: DrawList<1>,
    /// List at IDA 0x7590CA (the scene list of `redefine_keys_player3`): 0 entries and the -1 that ends it.
    pub redefine_keys_player3_scene: DrawList<1>,
    /// List at IDA 0x7590CE (the scene list of `redefine_keys_player4`): 0 entries and the -1 that ends it.
    pub redefine_keys_player4_scene: DrawList<1>,
    /// List at IDA 0x7590D2 (the scene list of `class_stage_results`): 0 entries and the -1 that ends it.
    pub class_stage_results_scene: DrawList<1>,
    /// List at IDA 0x7590D6 (the scene list of `overall_stage_results`): 0 entries and the -1 that ends it.
    pub overall_stage_results_scene: DrawList<1>,
    /// List at IDA 0x7590DA (the scene list of `class_rally_results`): 0 entries and the -1 that ends it.
    pub class_rally_results_scene: DrawList<1>,
    /// List at IDA 0x7590DE (the scene list of `overall_rally_results`): 0 entries and the -1 that ends it.
    pub overall_rally_results_scene: DrawList<1>,
    /// List at IDA 0x7590E2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7590e2: DrawList<1>,
    /// List at IDA 0x7590E6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7590e6: DrawList<1>,
    /// List at IDA 0x7590EA (the scene list of `screen_488f4c`): 0 entries and the -1 that ends it.
    pub screen_488f4c_scene: DrawList<1>,
    /// List at IDA 0x7590EE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7590ee: DrawList<1>,
    /// List at IDA 0x7590F2 (the scene list of `class_championship_standings`): 0 entries and the -1 that ends it.
    pub class_championship_standings_scene: DrawList<1>,
    /// List at IDA 0x7590F6 (the scene list of `championship_standings`): 0 entries and the -1 that ends it.
    pub championship_standings_scene: DrawList<1>,
    /// List at IDA 0x7590FA (the scene list of `arcade_results`): 0 entries and the -1 that ends it.
    pub arcade_results_scene: DrawList<1>,
    /// List at IDA 0x7590FE (the scene list of `stage_result`): 0 entries and the -1 that ends it.
    pub stage_result_scene: DrawList<1>,
    /// List at IDA 0x759102 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759102: DrawList<1>,
    /// List at IDA 0x759106 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759106: DrawList<1>,
    /// List at IDA 0x75910A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75910a: DrawList<1>,
    /// List at IDA 0x75910E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75910e: DrawList<1>,
    /// List at IDA 0x759112 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759112: DrawList<1>,
    /// List at IDA 0x759116 (named by no screen): 1 entry and the -1 that ends it.
    pub list_759116: DrawList<2>,
    /// List at IDA 0x75911E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75911e: DrawList<1>,
    /// List at IDA 0x759122 (the overlay list of `main_menu`): 1 entry and the -1 that ends it.
    pub main_menu_overlay: DrawList<2>,
    /// List at IDA 0x75912A (the overlay list of `start_rally`): 2 entries and the -1 that ends it.
    pub start_rally_overlay: DrawList<3>,
    /// List at IDA 0x759136 (the overlay list of `simple_select_players`): 1 entry and the -1 that ends it.
    pub simple_select_players_overlay: DrawList<2>,
    /// List at IDA 0x75913E (named by no screen): 1 entry and the -1 that ends it.
    pub list_75913e: DrawList<2>,
    /// List at IDA 0x759146 (screens: load_championship): 0 entries and the -1 that ends it.
    pub overlay_759146: DrawList<1>,
    /// List at IDA 0x75914A (screens: save_championship): 0 entries and the -1 that ends it.
    pub overlay_75914a: DrawList<1>,
    /// List at IDA 0x75914E (screens: save_replay): 0 entries and the -1 that ends it.
    pub overlay_75914e: DrawList<1>,
    /// List at IDA 0x759152 (screens: load_replay): 0 entries and the -1 that ends it.
    pub overlay_759152: DrawList<1>,
    /// List at IDA 0x759156 (the overlay list of `save_setup`): 1 entry and the -1 that ends it.
    pub save_setup_overlay: DrawList<2>,
    /// List at IDA 0x75915E (the overlay list of `load_championship_46e899`): 1 entry and the -1 that ends it.
    pub load_championship_46e899_overlay: DrawList<2>,
    /// List at IDA 0x759166 (named by no screen): 1 entry and the -1 that ends it.
    pub list_759166: DrawList<2>,
    /// List at IDA 0x75916E (the overlay list of `arcade_select_level`): 2 entries and the -1 that ends it.
    pub arcade_select_level_overlay: DrawList<3>,
    /// List at IDA 0x75917A (named by no screen): 1 entry and the -1 that ends it.
    pub list_75917a: DrawList<2>,
    /// List at IDA 0x759182 (the overlay list of `time_trial_select_players`): 1 entry and the -1 that ends it.
    pub time_trial_select_players_overlay: DrawList<2>,
    /// List at IDA 0x75918A (the overlay list of `championship_new_simulation`): 1 entry and the -1 that ends it.
    pub championship_new_simulation_overlay: DrawList<2>,
    /// List at IDA 0x759192 (the overlay list of `new_simulation`): 1 entry and the -1 that ends it.
    pub new_simulation_overlay: DrawList<2>,
    /// List at IDA 0x75919A (the overlay list of `network_rally`): 1 entry and the -1 that ends it.
    pub network_rally_overlay: DrawList<2>,
    /// List at IDA 0x7591A2 (the overlay list of `select_car`): 1 entry and the -1 that ends it.
    pub select_car_overlay: DrawList<2>,
    /// List at IDA 0x7591AA (the overlay list of `car_specs_47af4c`): 1 entry and the -1 that ends it.
    pub car_specs_47af4c_overlay: DrawList<2>,
    /// List at IDA 0x7591B2 (named by no screen): 1 entry and the -1 that ends it.
    pub list_7591b2: DrawList<2>,
    /// List at IDA 0x7591BA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7591ba: DrawList<1>,
    /// List at IDA 0x7591BE (the overlay list of `arcade_next_stage`): 0 entries and the -1 that ends it.
    pub arcade_next_stage_overlay: DrawList<1>,
    /// List at IDA 0x7591C2 (the overlay list of `options`): 0 entries and the -1 that ends it.
    pub options_overlay: DrawList<1>,
    /// List at IDA 0x7591C6 (the overlay list of `player_details`): 0 entries and the -1 that ends it.
    pub player_details_overlay: DrawList<1>,
    /// List at IDA 0x7591CA (the overlay list of `information`): 0 entries and the -1 that ends it.
    pub information_overlay: DrawList<1>,
    /// List at IDA 0x7591CE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7591ce: DrawList<1>,
    /// List at IDA 0x7591D2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7591d2: DrawList<1>,
    /// List at IDA 0x7591D6 (screens: audio_setup): 0 entries and the -1 that ends it.
    pub overlay_7591d6: DrawList<1>,
    /// List at IDA 0x7591DA (the overlay list of `advanced_audio`): 0 entries and the -1 that ends it.
    pub advanced_audio_overlay: DrawList<1>,
    /// List at IDA 0x7591DE (screens: leave_question): 0 entries and the -1 that ends it.
    pub overlay_7591de: DrawList<1>,
    /// List at IDA 0x7591E2 (screens: visual_preferences): 0 entries and the -1 that ends it.
    pub overlay_7591e2: DrawList<1>,
    /// List at IDA 0x7591E6 (the overlay list of `visual_options`): 0 entries and the -1 that ends it.
    pub visual_options_overlay: DrawList<1>,
    /// List at IDA 0x7591EA (screens: player1_details): 0 entries and the -1 that ends it.
    pub overlay_7591ea: DrawList<1>,
    /// List at IDA 0x7591EE (screens: player2_details): 0 entries and the -1 that ends it.
    pub overlay_7591ee: DrawList<1>,
    /// List at IDA 0x7591F2 (screens: player3_details): 0 entries and the -1 that ends it.
    pub overlay_7591f2: DrawList<1>,
    /// List at IDA 0x7591F6 (screens: player4_details): 0 entries and the -1 that ends it.
    pub overlay_7591f6: DrawList<1>,
    /// List at IDA 0x7591FA (the overlay list of `car_specification`): 1 entry and the -1 that ends it.
    pub car_specification_overlay: DrawList<2>,
    /// List at IDA 0x759202 (the overlay list of `slideshow`): 0 entries and the -1 that ends it.
    pub slideshow_overlay: DrawList<1>,
    /// List at IDA 0x759206 (the overlay list of `view_courses`): 0 entries and the -1 that ends it.
    pub view_courses_overlay: DrawList<1>,
    /// List at IDA 0x75920A (the overlay list of `time_trial_stage_details`): 0 entries and the -1 that ends it.
    pub time_trial_stage_details_overlay: DrawList<1>,
    /// List at IDA 0x75920E (the overlay list of `view_best_times`): 0 entries and the -1 that ends it.
    pub view_best_times_overlay: DrawList<1>,
    /// List at IDA 0x759212 (the overlay list of `view_best_scores`): 0 entries and the -1 that ends it.
    pub view_best_scores_overlay: DrawList<1>,
    /// List at IDA 0x759216 (the overlay list of `best_times`): 0 entries and the -1 that ends it.
    pub best_times_overlay: DrawList<1>,
    /// List at IDA 0x75921A (screens: arcade_best_times): 0 entries and the -1 that ends it.
    pub overlay_75921a: DrawList<1>,
    /// List at IDA 0x75921E (screens: time_trial_best_times): 0 entries and the -1 that ends it.
    pub overlay_75921e: DrawList<1>,
    /// List at IDA 0x759222 (screens: best_scores_48a69e): 0 entries and the -1 that ends it.
    pub overlay_759222: DrawList<1>,
    /// List at IDA 0x759226 (screens: best_class_scores): 0 entries and the -1 that ends it.
    pub overlay_759226: DrawList<1>,
    /// List at IDA 0x75922A (screens: best_overall_scores): 0 entries and the -1 that ends it.
    pub overlay_75922a: DrawList<1>,
    /// List at IDA 0x75922E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75922e: DrawList<1>,
    /// List at IDA 0x759232 (the overlay list of `welcome`): 0 entries and the -1 that ends it.
    pub welcome_overlay: DrawList<1>,
    /// List at IDA 0x759236 (the overlay list of `rally_stages`): 0 entries and the -1 that ends it.
    pub rally_stages_overlay: DrawList<1>,
    /// List at IDA 0x75923A (the overlay list of `review_options_47d5a5`): 0 entries and the -1 that ends it.
    pub review_options_47d5a5_overlay: DrawList<1>,
    /// List at IDA 0x75923E (the overlay list of `review_options_480d46`): 0 entries and the -1 that ends it.
    pub review_options_480d46_overlay: DrawList<1>,
    /// List at IDA 0x759242 (the overlay list of `review_options_48108e`): 0 entries and the -1 that ends it.
    pub review_options_48108e_overlay: DrawList<1>,
    /// List at IDA 0x759246 (the overlay list of `review_options_481429`): 0 entries and the -1 that ends it.
    pub review_options_481429_overlay: DrawList<1>,
    /// List at IDA 0x75924A (the overlay list of `service_area`): 1 entry and the -1 that ends it.
    pub service_area_overlay: DrawList<2>,
    /// List at IDA 0x759252 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759252: DrawList<1>,
    /// List at IDA 0x759256 (the overlay list of `next_stage`): 0 entries and the -1 that ends it.
    pub next_stage_overlay: DrawList<1>,
    /// List at IDA 0x75925A (the overlay list of `view_stage`): 0 entries and the -1 that ends it.
    pub view_stage_overlay: DrawList<1>,
    /// List at IDA 0x75925E (screens: player1_controller): 0 entries and the -1 that ends it.
    pub overlay_75925e: DrawList<1>,
    /// List at IDA 0x759262 (screens: player1_buttons): 0 entries and the -1 that ends it.
    pub overlay_759262: DrawList<1>,
    /// List at IDA 0x759266 (screens: player2_controller): 0 entries and the -1 that ends it.
    pub overlay_759266: DrawList<1>,
    /// List at IDA 0x75926A (screens: player2_buttons): 0 entries and the -1 that ends it.
    pub overlay_75926a: DrawList<1>,
    /// List at IDA 0x75926E (screens: player3_controller): 0 entries and the -1 that ends it.
    pub overlay_75926e: DrawList<1>,
    /// List at IDA 0x759272 (screens: player3_buttons): 0 entries and the -1 that ends it.
    pub overlay_759272: DrawList<1>,
    /// List at IDA 0x759276 (screens: player4_controller): 0 entries and the -1 that ends it.
    pub overlay_759276: DrawList<1>,
    /// List at IDA 0x75927A (screens: player4_buttons): 0 entries and the -1 that ends it.
    pub overlay_75927a: DrawList<1>,
    /// List at IDA 0x75927E (the overlay list of `inspection_area`): 1 entry and the -1 that ends it.
    pub inspection_area_overlay: DrawList<2>,
    /// List at IDA 0x759286 (screens: redefine_keys_player1): 0 entries and the -1 that ends it.
    pub overlay_759286: DrawList<1>,
    /// List at IDA 0x75928A (screens: redefine_keys_player2): 0 entries and the -1 that ends it.
    pub overlay_75928a: DrawList<1>,
    /// List at IDA 0x75928E (screens: redefine_keys_player3): 0 entries and the -1 that ends it.
    pub overlay_75928e: DrawList<1>,
    /// List at IDA 0x759292 (screens: redefine_keys_player4): 0 entries and the -1 that ends it.
    pub overlay_759292: DrawList<1>,
    /// List at IDA 0x759296 (the overlay list of `class_stage_results`): 0 entries and the -1 that ends it.
    pub class_stage_results_overlay: DrawList<1>,
    /// List at IDA 0x75929A (the overlay list of `overall_stage_results`): 0 entries and the -1 that ends it.
    pub overall_stage_results_overlay: DrawList<1>,
    /// List at IDA 0x75929E (the overlay list of `class_rally_results`): 0 entries and the -1 that ends it.
    pub class_rally_results_overlay: DrawList<1>,
    /// List at IDA 0x7592A2 (the overlay list of `overall_rally_results`): 0 entries and the -1 that ends it.
    pub overall_rally_results_overlay: DrawList<1>,
    /// List at IDA 0x7592A6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592a6: DrawList<1>,
    /// List at IDA 0x7592AA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592aa: DrawList<1>,
    /// List at IDA 0x7592AE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592ae: DrawList<1>,
    /// List at IDA 0x7592B2 (the overlay list of `class_championship_standings`): 0 entries and the -1 that ends it.
    pub class_championship_standings_overlay: DrawList<1>,
    /// List at IDA 0x7592B6 (screens: championship_standings): 0 entries and the -1 that ends it.
    pub overlay_7592b6: DrawList<1>,
    /// List at IDA 0x7592BA (the overlay list of `screen_488f4c`): 0 entries and the -1 that ends it.
    pub screen_488f4c_overlay: DrawList<1>,
    /// List at IDA 0x7592BE (screens: arcade_results): 0 entries and the -1 that ends it.
    pub overlay_7592be: DrawList<1>,
    /// List at IDA 0x7592C2 (screens: stage_result): 0 entries and the -1 that ends it.
    pub overlay_7592c2: DrawList<1>,
    /// List at IDA 0x7592C6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592c6: DrawList<1>,
    /// List at IDA 0x7592CA (screens: results_48584b): 0 entries and the -1 that ends it.
    pub overlay_7592ca: DrawList<1>,
    /// List at IDA 0x7592CE (screens: results_482e9b): 0 entries and the -1 that ends it.
    pub overlay_7592ce: DrawList<1>,
    /// List at IDA 0x7592D2 (screens: time_trial_results): 0 entries and the -1 that ends it.
    pub overlay_7592d2: DrawList<1>,
    /// List at IDA 0x7592D6 (the overlay list of `select_stage`): 0 entries and the -1 that ends it.
    pub select_stage_overlay: DrawList<1>,
    /// List at IDA 0x7592DA (the overlay list of `time_trial_select_stage`): 0 entries and the -1 that ends it.
    pub time_trial_select_stage_overlay: DrawList<1>,
    /// List at IDA 0x7592DE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592de: DrawList<1>,
    /// List at IDA 0x7592E2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592e2: DrawList<1>,
    /// List at IDA 0x7592E6 (the overlay list of `in_game_options`): 0 entries and the -1 that ends it.
    pub in_game_options_overlay: DrawList<1>,
    /// List at IDA 0x7592EA (the overlay list of `review_options_485f5e`): 0 entries and the -1 that ends it.
    pub review_options_485f5e_overlay: DrawList<1>,
    /// List at IDA 0x7592EE (the overlay list of `review_options_4882e2`): 0 entries and the -1 that ends it.
    pub review_options_4882e2_overlay: DrawList<1>,
    /// List at IDA 0x7592F2 (the overlay list of `review_options_48861d`): 0 entries and the -1 that ends it.
    pub review_options_48861d_overlay: DrawList<1>,
    /// List at IDA 0x7592F6 (the overlay list of `arcade_review_options`): 0 entries and the -1 that ends it.
    pub arcade_review_options_overlay: DrawList<1>,
    /// List at IDA 0x7592FA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7592fa: DrawList<1>,
    /// List at IDA 0x7592FE (screens: screen_483163): 0 entries and the -1 that ends it.
    pub overlay_7592fe: DrawList<1>,
    /// List at IDA 0x759302 (named by no screen): 1 entry and the -1 that ends it.
    pub list_759302: DrawList<2>,
    /// List at IDA 0x75930A (the setup list of `view_best_times`): 1 entry and the -1 that ends it.
    pub view_best_times_setup: DrawList<2>,
    /// List at IDA 0x759312 (the setup list of `view_best_scores`): 1 entry and the -1 that ends it.
    pub view_best_scores_setup: DrawList<2>,
    /// List at IDA 0x75931A (the setup list of `best_times`): 0 entries and the -1 that ends it.
    pub best_times_setup: DrawList<1>,
    /// List at IDA 0x75931E (the setup list of `select_stage`): 1 entry and the -1 that ends it.
    pub select_stage_setup: DrawList<2>,
    /// List at IDA 0x759326 (the setup list of `time_trial_select_stage`): 1 entry and the -1 that ends it.
    pub time_trial_select_stage_setup: DrawList<2>,
    /// List at IDA 0x75932E (the setup list of `view_courses`): 1 entry and the -1 that ends it.
    pub view_courses_setup: DrawList<2>,
    /// List at IDA 0x759336 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759336: DrawList<1>,
    /// List at IDA 0x75933A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75933a: DrawList<1>,
    /// List at IDA 0x75933E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75933e: DrawList<1>,
    /// List at IDA 0x759342 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759342: DrawList<1>,
    /// List at IDA 0x759346 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759346: DrawList<1>,
    /// List at IDA 0x75934A (the setup list of `main_menu`): 1 entry and the -1 that ends it.
    pub main_menu_setup: DrawList<2>,
    /// List at IDA 0x759352 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759352: DrawList<1>,
    /// List at IDA 0x759356 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759356: DrawList<1>,
    /// List at IDA 0x75935A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75935a: DrawList<1>,
    /// List at IDA 0x75935E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75935e: DrawList<1>,
    /// List at IDA 0x759362 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759362: DrawList<1>,
    /// List at IDA 0x759366 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759366: DrawList<1>,
    /// List at IDA 0x75936A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75936a: DrawList<1>,
    /// List at IDA 0x75936E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75936e: DrawList<1>,
    /// List at IDA 0x759372 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759372: DrawList<1>,
    /// List at IDA 0x759376 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759376: DrawList<1>,
    /// List at IDA 0x75937A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75937a: DrawList<1>,
    /// List at IDA 0x75937E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75937e: DrawList<1>,
    /// List at IDA 0x759382 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759382: DrawList<1>,
    /// List at IDA 0x759386 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759386: DrawList<1>,
    /// List at IDA 0x75938A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75938a: DrawList<1>,
    /// List at IDA 0x75938E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75938e: DrawList<1>,
    /// List at IDA 0x759392 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759392: DrawList<1>,
    /// List at IDA 0x759396 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759396: DrawList<1>,
    /// List at IDA 0x75939A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75939a: DrawList<1>,
    /// List at IDA 0x75939E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75939e: DrawList<1>,
    /// List at IDA 0x7593A2 (the setup list of `advanced_audio`): 1 entry and the -1 that ends it.
    pub advanced_audio_setup: DrawList<2>,
    /// List at IDA 0x7593AA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593aa: DrawList<1>,
    /// List at IDA 0x7593AE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593ae: DrawList<1>,
    /// List at IDA 0x7593B2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593b2: DrawList<1>,
    /// List at IDA 0x7593B6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593b6: DrawList<1>,
    /// List at IDA 0x7593BA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593ba: DrawList<1>,
    /// List at IDA 0x7593BE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593be: DrawList<1>,
    /// List at IDA 0x7593C2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593c2: DrawList<1>,
    /// List at IDA 0x7593C6 (the setup list of `car_specification`): 2 entries and the -1 that ends it.
    pub car_specification_setup: DrawList<3>,
    /// List at IDA 0x7593D2 (the setup list of `slideshow`): 2 entries and the -1 that ends it.
    pub slideshow_setup: DrawList<3>,
    /// List at IDA 0x7593DE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593de: DrawList<1>,
    /// List at IDA 0x7593E2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593e2: DrawList<1>,
    /// List at IDA 0x7593E6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593e6: DrawList<1>,
    /// List at IDA 0x7593EA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593ea: DrawList<1>,
    /// List at IDA 0x7593EE (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593ee: DrawList<1>,
    /// List at IDA 0x7593F2 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593f2: DrawList<1>,
    /// List at IDA 0x7593F6 (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593f6: DrawList<1>,
    /// List at IDA 0x7593FA (named by no screen): 0 entries and the -1 that ends it.
    pub list_7593fa: DrawList<1>,
    /// List at IDA 0x7593FE (the setup list of `select_car`): 3 entries and the -1 that ends it.
    pub select_car_setup: DrawList<4>,
    /// List at IDA 0x75940E (the setup list of `car_specs_47af4c`): 2 entries and the -1 that ends it.
    pub car_specs_47af4c_setup: DrawList<3>,
    /// List at IDA 0x75941A (named by no screen): 2 entries and the -1 that ends it.
    pub list_75941a: DrawList<3>,
    /// List at IDA 0x759426 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759426: DrawList<1>,
    /// List at IDA 0x75942A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75942a: DrawList<1>,
    /// List at IDA 0x75942E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75942e: DrawList<1>,
    /// List at IDA 0x759432 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759432: DrawList<1>,
    /// List at IDA 0x759436 (the setup list of `service_area`): 1 entry and the -1 that ends it.
    pub service_area_setup: DrawList<2>,
    /// List at IDA 0x75943E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75943e: DrawList<1>,
    /// List at IDA 0x759442 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759442: DrawList<1>,
    /// List at IDA 0x759446 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759446: DrawList<1>,
    /// List at IDA 0x75944A (the setup list of `load_championship`): 1 entry and the -1 that ends it.
    pub load_championship_setup: DrawList<2>,
    /// List at IDA 0x759452 (the setup list of `save_championship`): 1 entry and the -1 that ends it.
    pub save_championship_setup: DrawList<2>,
    /// List at IDA 0x75945A (the setup list of `save_replay`): 1 entry and the -1 that ends it.
    pub save_replay_setup: DrawList<2>,
    /// List at IDA 0x759462 (the setup list of `load_replay`): 1 entry and the -1 that ends it.
    pub load_replay_setup: DrawList<2>,
    /// List at IDA 0x75946A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75946a: DrawList<1>,
    /// List at IDA 0x75946E (named by no screen): 0 entries and the -1 that ends it.
    pub list_75946e: DrawList<1>,
    /// List at IDA 0x759472 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759472: DrawList<1>,
    /// List at IDA 0x759476 (named by no screen): 0 entries and the -1 that ends it.
    pub list_759476: DrawList<1>,
    /// List at IDA 0x75947A (named by no screen): 0 entries and the -1 that ends it.
    pub list_75947a: DrawList<1>,
    /// List at IDA 0x75947E (the setup list of `player1_controller`): 1 entry and the -1 that ends it.
    pub player1_controller_setup: DrawList<2>,
    /// List at IDA 0x759486 (the setup list of `player1_buttons`): 0 entries and the -1 that ends it.
    pub player1_buttons_setup: DrawList<1>,
    /// List at IDA 0x75948A (the setup list of `player2_controller`): 1 entry and the -1 that ends it.
    pub player2_controller_setup: DrawList<2>,
    /// List at IDA 0x759492 (the setup list of `player2_buttons`): 0 entries and the -1 that ends it.
    pub player2_buttons_setup: DrawList<1>,
    /// List at IDA 0x759496 (the setup list of `player3_controller`): 1 entry and the -1 that ends it.
    pub player3_controller_setup: DrawList<2>,
    /// List at IDA 0x75949E (the setup list of `player3_buttons`): 0 entries and the -1 that ends it.
    pub player3_buttons_setup: DrawList<1>,
    /// List at IDA 0x7594A2 (the setup list of `player4_controller`): 1 entry and the -1 that ends it.
    pub player4_controller_setup: DrawList<2>,
}

/// First values the image's (`tests::draw_lists_start_as_the_games`).
#[rustfmt::skip]
pub static DRAW_LISTS: DrawLists = DrawLists {
        unread_758b72: DrawList::new([0xFFFFFFFF]),
        main_menu_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        start_rally_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        simple_select_players_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        list_758b9a: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        draw_758ba6: DrawList::new([0x46D262, 0x46D45A, 0x46D88F, 0xFFFFFFFF]),
        save_setup_draw: DrawList::new([0xFFFFFFFF]),
        load_championship_46e899_draw: DrawList::new([0xFFFFFFFF]),
        draw_758bbe: DrawList::new([0x46D262, 0x46D45A, 0x46D88F, 0xFFFFFFFF]),
        draw_758bce: DrawList::new([0x46F41A, 0xFFFFFFFF]),
        draw_758bd6: DrawList::new([0x46F41A, 0xFFFFFFFF]),
        list_758bde: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        arcade_select_level_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        list_758bf6: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        time_trial_select_players_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        championship_new_simulation_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        new_simulation_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        network_rally_draw: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        select_car_draw: DrawList::new([0x46989B, 0x469873, 0x46A359, 0xFFFFFFFF]),
        car_specs_47af4c_draw: DrawList::new([0x46989B, 0x469873, 0x46A359, 0xFFFFFFFF]),
        list_758c52: DrawList::new([0x46989B, 0x469873, 0x46A359, 0xFFFFFFFF]),
        list_758c62: DrawList::new([0x4881F8, 0xFFFFFFFF]),
        arcade_next_stage_draw: DrawList::new([0x489AC8, 0xFFFFFFFF]),
        options_draw: DrawList::new([0xFFFFFFFF]),
        list_758c76: DrawList::new([0xFFFFFFFF]),
        player_details_draw: DrawList::new([0xFFFFFFFF]),
        list_758c7e: DrawList::new([0xFFFFFFFF]),
        information_draw: DrawList::new([0xFFFFFFFF]),
        list_758c86: DrawList::new([0xFFFFFFFF]),
        list_758c8a: DrawList::new([0xFFFFFFFF]),
        draw_758c8e: DrawList::new([0x476A92, 0xFFFFFFFF]),
        advanced_audio_draw: DrawList::new([0x47022A, 0xFFFFFFFF]),
        draw_758c9e: DrawList::new([0x46FA5A, 0x46FA75, 0xFFFFFFFF]),
        draw_758caa: DrawList::new([0x47A4C7, 0xFFFFFFFF]),
        visual_options_draw: DrawList::new([0x46FDF0, 0xFFFFFFFF]),
        draw_758cba: DrawList::new([0x46F894, 0xFFFFFFFF]),
        draw_758cc2: DrawList::new([0x4677B5, 0xFFFFFFFF]),
        draw_758cca: DrawList::new([0x467975, 0xFFFFFFFF]),
        draw_758cd2: DrawList::new([0x467975, 0xFFFFFFFF]),
        car_specification_draw: DrawList::new([0x46989B, 0x469A0E, 0x469A34, 0xFFFFFFFF]),
        slideshow_draw: DrawList::new([0x469A0E, 0x469A57, 0xFFFFFFFF]),
        view_courses_draw: DrawList::new([0x481A78, 0xFFFFFFFF]),
        time_trial_stage_details_draw: DrawList::new([0x487011, 0xFFFFFFFF]),
        view_best_times_draw: DrawList::new([0x48E740, 0xFFFFFFFF]),
        view_best_scores_draw: DrawList::new([0x46D128, 0x46D149, 0xFFFFFFFF]),
        best_times_draw: DrawList::new([0x4A41AE, 0xFFFFFFFF]),
        draw_758d22: DrawList::new([0x48A948, 0xFFFFFFFF]),
        draw_758d2a: DrawList::new([0x48D0F2, 0xFFFFFFFF]),
        draw_758d32: DrawList::new([0x48AB87, 0xFFFFFFFF]),
        draw_758d3a: DrawList::new([0x4A8DBF, 0xFFFFFFFF]),
        draw_758d42: DrawList::new([0x4A8FA2, 0xFFFFFFFF]),
        list_758d4a: DrawList::new([0xFFFFFFFF]),
        welcome_draw: DrawList::new([0xFFFFFFFF]),
        list_758d52: DrawList::new([0xFFFFFFFF]),
        list_758d56: DrawList::new([0x472C8C, 0xFFFFFFFF]),
        review_options_47d5a5_draw: DrawList::new([0xFFFFFFFF]),
        review_options_480d46_draw: DrawList::new([0xFFFFFFFF]),
        review_options_48108e_draw: DrawList::new([0xFFFFFFFF]),
        review_options_481429_draw: DrawList::new([0xFFFFFFFF]),
        service_area_draw: DrawList::new([0x46B890, 0xFFFFFFFF]),
        list_758d76: DrawList::new([0xFFFFFFFF]),
        list_758d7a: DrawList::new([0xFFFFFFFF]),
        next_stage_draw: DrawList::new([0x473DE4, 0xFFFFFFFF]),
        view_stage_draw: DrawList::new([0x480064, 0xFFFFFFFF]),
        draw_758d8e: DrawList::new([0x46A454, 0xFFFFFFFF]),
        draw_758d96: DrawList::new([0x46B131, 0x46A829, 0xFFFFFFFF]),
        draw_758da2: DrawList::new([0x46A454, 0xFFFFFFFF]),
        draw_758daa: DrawList::new([0x46B131, 0x46A829, 0xFFFFFFFF]),
        draw_758db6: DrawList::new([0x46A454, 0xFFFFFFFF]),
        draw_758dbe: DrawList::new([0x46B131, 0x46A829, 0xFFFFFFFF]),
        draw_758dca: DrawList::new([0x46A454, 0xFFFFFFFF]),
        draw_758dd2: DrawList::new([0x46B131, 0x46A829, 0xFFFFFFFF]),
        inspection_area_draw: DrawList::new([0xFFFFFFFF]),
        draw_758de2: DrawList::new([0x47D3D8, 0x47CFA7, 0xFFFFFFFF]),
        draw_758dee: DrawList::new([0x47D472, 0x47CFA7, 0xFFFFFFFF]),
        draw_758dfa: DrawList::new([0x47D472, 0x47CFA7, 0xFFFFFFFF]),
        draw_758e06: DrawList::new([0x47D472, 0x47CFA7, 0xFFFFFFFF]),
        list_758e12: DrawList::new([0x4A2111, 0x4A1FBB, 0xFFFFFFFF]),
        list_758e1e: DrawList::new([0x4A2F46, 0x4A1FBB, 0xFFFFFFFF]),
        list_758e2a: DrawList::new([0x4A3376, 0x4A1FBB, 0xFFFFFFFF]),
        list_758e36: DrawList::new([0x4A37E2, 0x4A1FBB, 0xFFFFFFFF]),
        list_758e42: DrawList::new([0xFFFFFFFF]),
        list_758e46: DrawList::new([0x47C16A, 0xFFFFFFFF]),
        list_758e4e: DrawList::new([0x46762B, 0xFFFFFFFF]),
        class_championship_standings_draw: DrawList::new([0x4A4E1E, 0xFFFFFFFF]),
        list_758e5e: DrawList::new([0x4A5410, 0xFFFFFFFF]),
        screen_488f4c_draw: DrawList::new([0x488F2F, 0xFFFFFFFF]),
        list_758e6e: DrawList::new([0x48558A, 0xFFFFFFFF]),
        draw_758e76: DrawList::new([0x4894AD, 0xFFFFFFFF]),
        draw_758e7e: DrawList::new([0x486299, 0xFFFFFFFF]),
        draw_758e86: DrawList::new([0x48647A, 0xFFFFFFFF]),
        draw_758e8e: DrawList::new([0x4836CF, 0xFFFFFFFF]),
        draw_758e96: DrawList::new([0x48D32E, 0xFFFFFFFF]),
        select_stage_draw: DrawList::new([0x481A78, 0xFFFFFFFF]),
        time_trial_select_stage_draw: DrawList::new([0x48C2A0, 0xFFFFFFFF]),
        list_758eae: DrawList::new([0x4A94E9, 0x4A94E9, 0xFFFFFFFF]),
        review_options_485f5e_draw: DrawList::new([0xFFFFFFFF]),
        review_options_4882e2_draw: DrawList::new([0xFFFFFFFF]),
        review_options_48861d_draw: DrawList::new([0xFFFFFFFF]),
        arcade_review_options_draw: DrawList::new([0xFFFFFFFF]),
        list_758eca: DrawList::new([0xFFFFFFFF]),
        draw_758ece: DrawList::new([0x483FDE, 0xFFFFFFFF]),
        list_758ed6: DrawList::new([0xFFFFFFFF]),
        list_758eda: DrawList::new([0x46989B, 0x4695C6, 0xFFFFFFFF]),
        main_menu_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        start_rally_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        simple_select_players_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        arcade_select_level_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        list_758f06: DrawList::new([0x490106, 0xFFFFFFFF]),
        load_championship_scene: DrawList::new([0xFFFFFFFF]),
        save_setup_scene: DrawList::new([0x49FE4B, 0xFFFFFFFF]),
        load_championship_46e899_scene: DrawList::new([0x49FE4B, 0xFFFFFFFF]),
        save_championship_scene: DrawList::new([0xFFFFFFFF]),
        save_replay_scene: DrawList::new([0xFFFFFFFF]),
        load_replay_scene: DrawList::new([0xFFFFFFFF]),
        list_758f2e: DrawList::new([0x490106, 0xFFFFFFFF]),
        list_758f36: DrawList::new([0x490106, 0xFFFFFFFF]),
        time_trial_select_players_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        championship_new_simulation_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        new_simulation_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        network_rally_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        select_car_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        car_specs_47af4c_scene: DrawList::new([0x490106, 0xFFFFFFFF]),
        list_758f6e: DrawList::new([0x490106, 0xFFFFFFFF]),
        car_specification_scene: DrawList::new([0x49017B, 0xFFFFFFFF]),
        slideshow_scene: DrawList::new([0xFFFFFFFF]),
        screen_483163_scene: DrawList::new([0xFFFFFFFF]),
        results_48584b_scene: DrawList::new([0xFFFFFFFF]),
        review_options_485f5e_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        list_758f92: DrawList::new([0xFFFFFFFF]),
        review_options_4882e2_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        review_options_48861d_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        arcade_review_options_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        list_758fae: DrawList::new([0xFFFFFFFF]),
        best_scores_48a69e_scene: DrawList::new([0xFFFFFFFF]),
        best_class_scores_scene: DrawList::new([0xFFFFFFFF]),
        best_overall_scores_scene: DrawList::new([0xFFFFFFFF]),
        time_trial_select_stage_scene: DrawList::new([0xFFFFFFFF]),
        time_trial_best_times_scene: DrawList::new([0xFFFFFFFF]),
        time_trial_results_scene: DrawList::new([0xFFFFFFFF]),
        view_best_times_scene: DrawList::new([0xFFFFFFFF]),
        view_best_scores_scene: DrawList::new([0xFFFFFFFF]),
        best_times_scene: DrawList::new([0xFFFFFFFF]),
        arcade_next_stage_scene: DrawList::new([0xFFFFFFFF]),
        list_758fda: DrawList::new([0xFFFFFFFF]),
        options_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        player_details_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        information_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        list_758ff6: DrawList::new([0xFFFFFFFF]),
        audio_setup_scene: DrawList::new([0xFFFFFFFF]),
        advanced_audio_scene: DrawList::new([0xFFFFFFFF]),
        leave_question_scene: DrawList::new([0xFFFFFFFF]),
        visual_preferences_scene: DrawList::new([0xFFFFFFFF]),
        visual_options_scene: DrawList::new([0xFFFFFFFF]),
        player1_details_scene: DrawList::new([0xFFFFFFFF]),
        list_759012: DrawList::new([0xFFFFFFFF]),
        player2_details_scene: DrawList::new([0xFFFFFFFF]),
        list_75901a: DrawList::new([0xFFFFFFFF]),
        player3_details_scene: DrawList::new([0xFFFFFFFF]),
        list_759022: DrawList::new([0xFFFFFFFF]),
        player4_details_scene: DrawList::new([0xFFFFFFFF]),
        list_75902a: DrawList::new([0xFFFFFFFF]),
        view_courses_scene: DrawList::new([0xFFFFFFFF]),
        list_759032: DrawList::new([0xFFFFFFFF]),
        time_trial_stage_details_scene: DrawList::new([0xFFFFFFFF]),
        list_75903a: DrawList::new([0xFFFFFFFF]),
        arcade_best_times_scene: DrawList::new([0xFFFFFFFF]),
        list_759042: DrawList::new([0xFFFFFFFF]),
        list_759046: DrawList::new([0xFFFFFFFF]),
        in_game_options_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        select_stage_scene: DrawList::new([0xFFFFFFFF]),
        results_482e9b_scene: DrawList::new([0xFFFFFFFF]),
        welcome_scene: DrawList::new([0xFFFFFFFF]),
        rally_stages_scene: DrawList::new([0xFFFFFFFF]),
        review_options_47d5a5_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        review_options_480d46_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        review_options_48108e_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        review_options_481429_scene: DrawList::new([0x49004E, 0xFFFFFFFF]),
        service_area_scene: DrawList::new([0x4940BC, 0xFFFFFFFF]),
        list_75908a: DrawList::new([0xFFFFFFFF]),
        list_75908e: DrawList::new([0xFFFFFFFF]),
        next_stage_scene: DrawList::new([0xFFFFFFFF]),
        view_stage_scene: DrawList::new([0xFFFFFFFF]),
        player1_controller_scene: DrawList::new([0xFFFFFFFF]),
        player1_buttons_scene: DrawList::new([0xFFFFFFFF]),
        player2_controller_scene: DrawList::new([0xFFFFFFFF]),
        player2_buttons_scene: DrawList::new([0xFFFFFFFF]),
        player3_controller_scene: DrawList::new([0xFFFFFFFF]),
        player3_buttons_scene: DrawList::new([0xFFFFFFFF]),
        player4_controller_scene: DrawList::new([0xFFFFFFFF]),
        player4_buttons_scene: DrawList::new([0xFFFFFFFF]),
        inspection_area_scene: DrawList::new([0x49FCCF, 0xFFFFFFFF]),
        redefine_keys_player1_scene: DrawList::new([0xFFFFFFFF]),
        redefine_keys_player2_scene: DrawList::new([0xFFFFFFFF]),
        redefine_keys_player3_scene: DrawList::new([0xFFFFFFFF]),
        redefine_keys_player4_scene: DrawList::new([0xFFFFFFFF]),
        class_stage_results_scene: DrawList::new([0xFFFFFFFF]),
        overall_stage_results_scene: DrawList::new([0xFFFFFFFF]),
        class_rally_results_scene: DrawList::new([0xFFFFFFFF]),
        overall_rally_results_scene: DrawList::new([0xFFFFFFFF]),
        list_7590e2: DrawList::new([0xFFFFFFFF]),
        list_7590e6: DrawList::new([0xFFFFFFFF]),
        screen_488f4c_scene: DrawList::new([0xFFFFFFFF]),
        list_7590ee: DrawList::new([0xFFFFFFFF]),
        class_championship_standings_scene: DrawList::new([0xFFFFFFFF]),
        championship_standings_scene: DrawList::new([0xFFFFFFFF]),
        arcade_results_scene: DrawList::new([0xFFFFFFFF]),
        stage_result_scene: DrawList::new([0xFFFFFFFF]),
        list_759102: DrawList::new([0xFFFFFFFF]),
        list_759106: DrawList::new([0xFFFFFFFF]),
        list_75910a: DrawList::new([0xFFFFFFFF]),
        list_75910e: DrawList::new([0xFFFFFFFF]),
        list_759112: DrawList::new([0xFFFFFFFF]),
        list_759116: DrawList::new([0x490106, 0xFFFFFFFF]),
        list_75911e: DrawList::new([0xFFFFFFFF]),
        main_menu_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        start_rally_overlay: DrawList::new([0x4698C4, 0x4AAAB8, 0xFFFFFFFF]),
        simple_select_players_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        list_75913e: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        overlay_759146: DrawList::new([0xFFFFFFFF]),
        overlay_75914a: DrawList::new([0xFFFFFFFF]),
        overlay_75914e: DrawList::new([0xFFFFFFFF]),
        overlay_759152: DrawList::new([0xFFFFFFFF]),
        save_setup_overlay: DrawList::new([0x46F031, 0xFFFFFFFF]),
        load_championship_46e899_overlay: DrawList::new([0x46F087, 0xFFFFFFFF]),
        list_759166: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        arcade_select_level_overlay: DrawList::new([0x4698C4, 0x48ADCE, 0xFFFFFFFF]),
        list_75917a: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        time_trial_select_players_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        championship_new_simulation_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        new_simulation_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        network_rally_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        select_car_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        car_specs_47af4c_overlay: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        list_7591b2: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        list_7591ba: DrawList::new([0xFFFFFFFF]),
        arcade_next_stage_overlay: DrawList::new([0xFFFFFFFF]),
        options_overlay: DrawList::new([0xFFFFFFFF]),
        player_details_overlay: DrawList::new([0xFFFFFFFF]),
        information_overlay: DrawList::new([0xFFFFFFFF]),
        list_7591ce: DrawList::new([0xFFFFFFFF]),
        list_7591d2: DrawList::new([0xFFFFFFFF]),
        overlay_7591d6: DrawList::new([0xFFFFFFFF]),
        advanced_audio_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_7591de: DrawList::new([0xFFFFFFFF]),
        overlay_7591e2: DrawList::new([0xFFFFFFFF]),
        visual_options_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_7591ea: DrawList::new([0xFFFFFFFF]),
        overlay_7591ee: DrawList::new([0xFFFFFFFF]),
        overlay_7591f2: DrawList::new([0xFFFFFFFF]),
        overlay_7591f6: DrawList::new([0xFFFFFFFF]),
        car_specification_overlay: DrawList::new([0x47CEB4, 0xFFFFFFFF]),
        slideshow_overlay: DrawList::new([0xFFFFFFFF]),
        view_courses_overlay: DrawList::new([0xFFFFFFFF]),
        time_trial_stage_details_overlay: DrawList::new([0xFFFFFFFF]),
        view_best_times_overlay: DrawList::new([0xFFFFFFFF]),
        view_best_scores_overlay: DrawList::new([0xFFFFFFFF]),
        best_times_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_75921a: DrawList::new([0xFFFFFFFF]),
        overlay_75921e: DrawList::new([0xFFFFFFFF]),
        overlay_759222: DrawList::new([0xFFFFFFFF]),
        overlay_759226: DrawList::new([0xFFFFFFFF]),
        overlay_75922a: DrawList::new([0xFFFFFFFF]),
        list_75922e: DrawList::new([0xFFFFFFFF]),
        welcome_overlay: DrawList::new([0xFFFFFFFF]),
        rally_stages_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_47d5a5_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_480d46_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_48108e_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_481429_overlay: DrawList::new([0xFFFFFFFF]),
        service_area_overlay: DrawList::new([0x4A5FA2, 0xFFFFFFFF]),
        list_759252: DrawList::new([0xFFFFFFFF]),
        next_stage_overlay: DrawList::new([0xFFFFFFFF]),
        view_stage_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_75925e: DrawList::new([0xFFFFFFFF]),
        overlay_759262: DrawList::new([0xFFFFFFFF]),
        overlay_759266: DrawList::new([0xFFFFFFFF]),
        overlay_75926a: DrawList::new([0xFFFFFFFF]),
        overlay_75926e: DrawList::new([0xFFFFFFFF]),
        overlay_759272: DrawList::new([0xFFFFFFFF]),
        overlay_759276: DrawList::new([0xFFFFFFFF]),
        overlay_75927a: DrawList::new([0xFFFFFFFF]),
        inspection_area_overlay: DrawList::new([0x47DEA2, 0xFFFFFFFF]),
        overlay_759286: DrawList::new([0xFFFFFFFF]),
        overlay_75928a: DrawList::new([0xFFFFFFFF]),
        overlay_75928e: DrawList::new([0xFFFFFFFF]),
        overlay_759292: DrawList::new([0xFFFFFFFF]),
        class_stage_results_overlay: DrawList::new([0xFFFFFFFF]),
        overall_stage_results_overlay: DrawList::new([0xFFFFFFFF]),
        class_rally_results_overlay: DrawList::new([0xFFFFFFFF]),
        overall_rally_results_overlay: DrawList::new([0xFFFFFFFF]),
        list_7592a6: DrawList::new([0xFFFFFFFF]),
        list_7592aa: DrawList::new([0xFFFFFFFF]),
        list_7592ae: DrawList::new([0xFFFFFFFF]),
        class_championship_standings_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_7592b6: DrawList::new([0xFFFFFFFF]),
        screen_488f4c_overlay: DrawList::new([0xFFFFFFFF]),
        overlay_7592be: DrawList::new([0xFFFFFFFF]),
        overlay_7592c2: DrawList::new([0xFFFFFFFF]),
        list_7592c6: DrawList::new([0xFFFFFFFF]),
        overlay_7592ca: DrawList::new([0xFFFFFFFF]),
        overlay_7592ce: DrawList::new([0xFFFFFFFF]),
        overlay_7592d2: DrawList::new([0xFFFFFFFF]),
        select_stage_overlay: DrawList::new([0xFFFFFFFF]),
        time_trial_select_stage_overlay: DrawList::new([0xFFFFFFFF]),
        list_7592de: DrawList::new([0xFFFFFFFF]),
        list_7592e2: DrawList::new([0xFFFFFFFF]),
        in_game_options_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_485f5e_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_4882e2_overlay: DrawList::new([0xFFFFFFFF]),
        review_options_48861d_overlay: DrawList::new([0xFFFFFFFF]),
        arcade_review_options_overlay: DrawList::new([0xFFFFFFFF]),
        list_7592fa: DrawList::new([0xFFFFFFFF]),
        overlay_7592fe: DrawList::new([0xFFFFFFFF]),
        list_759302: DrawList::new([0x4698C4, 0xFFFFFFFF]),
        view_best_times_setup: DrawList::new([0x4692A9, 0xFFFFFFFF]),
        view_best_scores_setup: DrawList::new([0x46D128, 0xFFFFFFFF]),
        best_times_setup: DrawList::new([0xFFFFFFFF]),
        select_stage_setup: DrawList::new([0x469393, 0xFFFFFFFF]),
        time_trial_select_stage_setup: DrawList::new([0x469393, 0xFFFFFFFF]),
        view_courses_setup: DrawList::new([0x469434, 0xFFFFFFFF]),
        list_759336: DrawList::new([0xFFFFFFFF]),
        list_75933a: DrawList::new([0xFFFFFFFF]),
        list_75933e: DrawList::new([0xFFFFFFFF]),
        list_759342: DrawList::new([0xFFFFFFFF]),
        list_759346: DrawList::new([0xFFFFFFFF]),
        main_menu_setup: DrawList::new([0x46989B, 0xFFFFFFFF]),
        list_759352: DrawList::new([0xFFFFFFFF]),
        list_759356: DrawList::new([0xFFFFFFFF]),
        list_75935a: DrawList::new([0xFFFFFFFF]),
        list_75935e: DrawList::new([0xFFFFFFFF]),
        list_759362: DrawList::new([0xFFFFFFFF]),
        list_759366: DrawList::new([0xFFFFFFFF]),
        list_75936a: DrawList::new([0xFFFFFFFF]),
        list_75936e: DrawList::new([0xFFFFFFFF]),
        list_759372: DrawList::new([0xFFFFFFFF]),
        list_759376: DrawList::new([0xFFFFFFFF]),
        list_75937a: DrawList::new([0xFFFFFFFF]),
        list_75937e: DrawList::new([0xFFFFFFFF]),
        list_759382: DrawList::new([0xFFFFFFFF]),
        list_759386: DrawList::new([0xFFFFFFFF]),
        list_75938a: DrawList::new([0xFFFFFFFF]),
        list_75938e: DrawList::new([0xFFFFFFFF]),
        list_759392: DrawList::new([0xFFFFFFFF]),
        list_759396: DrawList::new([0xFFFFFFFF]),
        list_75939a: DrawList::new([0xFFFFFFFF]),
        list_75939e: DrawList::new([0xFFFFFFFF]),
        advanced_audio_setup: DrawList::new([0x4707E6, 0xFFFFFFFF]),
        list_7593aa: DrawList::new([0xFFFFFFFF]),
        list_7593ae: DrawList::new([0xFFFFFFFF]),
        list_7593b2: DrawList::new([0xFFFFFFFF]),
        list_7593b6: DrawList::new([0xFFFFFFFF]),
        list_7593ba: DrawList::new([0xFFFFFFFF]),
        list_7593be: DrawList::new([0xFFFFFFFF]),
        list_7593c2: DrawList::new([0xFFFFFFFF]),
        car_specification_setup: DrawList::new([0x469A0E, 0x469A34, 0xFFFFFFFF]),
        slideshow_setup: DrawList::new([0x469A0E, 0x469A57, 0xFFFFFFFF]),
        list_7593de: DrawList::new([0xFFFFFFFF]),
        list_7593e2: DrawList::new([0xFFFFFFFF]),
        list_7593e6: DrawList::new([0xFFFFFFFF]),
        list_7593ea: DrawList::new([0xFFFFFFFF]),
        list_7593ee: DrawList::new([0xFFFFFFFF]),
        list_7593f2: DrawList::new([0xFFFFFFFF]),
        list_7593f6: DrawList::new([0xFFFFFFFF]),
        list_7593fa: DrawList::new([0xFFFFFFFF]),
        select_car_setup: DrawList::new([0x4699E0, 0x46A359, 0x490346, 0xFFFFFFFF]),
        car_specs_47af4c_setup: DrawList::new([0x46A359, 0x490346, 0xFFFFFFFF]),
        list_75941a: DrawList::new([0x4699E0, 0x46A359, 0xFFFFFFFF]),
        list_759426: DrawList::new([0xFFFFFFFF]),
        list_75942a: DrawList::new([0xFFFFFFFF]),
        list_75942e: DrawList::new([0xFFFFFFFF]),
        list_759432: DrawList::new([0xFFFFFFFF]),
        service_area_setup: DrawList::new([0x47070C, 0xFFFFFFFF]),
        list_75943e: DrawList::new([0xFFFFFFFF]),
        list_759442: DrawList::new([0xFFFFFFFF]),
        list_759446: DrawList::new([0xFFFFFFFF]),
        load_championship_setup: DrawList::new([0x46D262, 0xFFFFFFFF]),
        save_championship_setup: DrawList::new([0x46D262, 0xFFFFFFFF]),
        save_replay_setup: DrawList::new([0x470501, 0xFFFFFFFF]),
        load_replay_setup: DrawList::new([0x470501, 0xFFFFFFFF]),
        list_75946a: DrawList::new([0xFFFFFFFF]),
        list_75946e: DrawList::new([0xFFFFFFFF]),
        list_759472: DrawList::new([0xFFFFFFFF]),
        list_759476: DrawList::new([0xFFFFFFFF]),
        list_75947a: DrawList::new([0xFFFFFFFF]),
        player1_controller_setup: DrawList::new([0x46AEC5, 0xFFFFFFFF]),
        player1_buttons_setup: DrawList::new([0xFFFFFFFF]),
        player2_controller_setup: DrawList::new([0x46AF21, 0xFFFFFFFF]),
        player2_buttons_setup: DrawList::new([0xFFFFFFFF]),
        player3_controller_setup: DrawList::new([0x46AF58, 0xFFFFFFFF]),
        player3_buttons_setup: DrawList::new([0xFFFFFFFF]),
        player4_controller_setup: DrawList::new([0x46AF8F, 0xFFFFFFFF]),
};

/// IDA 0x758B72, the first byte of the piece.
pub const START: usize = 0x758B72;
/// IDA 0x7594AA, the end of the piece.
pub const END: usize = 0x7594AA;

/// Where IDA 0x758B72 is now (`moved::MOVED`'s home).
pub fn draw_lists_addr() -> usize {
    crate::pointers::host_number(&DRAW_LISTS as *const DrawLists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn draw_lists_start_as_the_games() {
        assert_eq!(size_of::<DrawLists>(), END - START);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(draw_lists_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(DrawLists, unread_758b72), 0x758B72);
        at(offset_of!(DrawLists, main_menu_draw), 0x758B76);
        at(offset_of!(DrawLists, start_rally_draw), 0x758B82);
        at(offset_of!(DrawLists, simple_select_players_draw), 0x758B8E);
        at(offset_of!(DrawLists, list_758b9a), 0x758B9A);
        at(offset_of!(DrawLists, draw_758ba6), 0x758BA6);
        at(offset_of!(DrawLists, save_setup_draw), 0x758BB6);
        at(
            offset_of!(DrawLists, load_championship_46e899_draw),
            0x758BBA,
        );
        at(offset_of!(DrawLists, draw_758bbe), 0x758BBE);
        at(offset_of!(DrawLists, draw_758bce), 0x758BCE);
        at(offset_of!(DrawLists, draw_758bd6), 0x758BD6);
        at(offset_of!(DrawLists, list_758bde), 0x758BDE);
        at(offset_of!(DrawLists, arcade_select_level_draw), 0x758BEA);
        at(offset_of!(DrawLists, list_758bf6), 0x758BF6);
        at(
            offset_of!(DrawLists, time_trial_select_players_draw),
            0x758C02,
        );
        at(
            offset_of!(DrawLists, championship_new_simulation_draw),
            0x758C0E,
        );
        at(offset_of!(DrawLists, new_simulation_draw), 0x758C1A);
        at(offset_of!(DrawLists, network_rally_draw), 0x758C26);
        at(offset_of!(DrawLists, select_car_draw), 0x758C32);
        at(offset_of!(DrawLists, car_specs_47af4c_draw), 0x758C42);
        at(offset_of!(DrawLists, list_758c52), 0x758C52);
        at(offset_of!(DrawLists, list_758c62), 0x758C62);
        at(offset_of!(DrawLists, arcade_next_stage_draw), 0x758C6A);
        at(offset_of!(DrawLists, options_draw), 0x758C72);
        at(offset_of!(DrawLists, list_758c76), 0x758C76);
        at(offset_of!(DrawLists, player_details_draw), 0x758C7A);
        at(offset_of!(DrawLists, list_758c7e), 0x758C7E);
        at(offset_of!(DrawLists, information_draw), 0x758C82);
        at(offset_of!(DrawLists, list_758c86), 0x758C86);
        at(offset_of!(DrawLists, list_758c8a), 0x758C8A);
        at(offset_of!(DrawLists, draw_758c8e), 0x758C8E);
        at(offset_of!(DrawLists, advanced_audio_draw), 0x758C96);
        at(offset_of!(DrawLists, draw_758c9e), 0x758C9E);
        at(offset_of!(DrawLists, draw_758caa), 0x758CAA);
        at(offset_of!(DrawLists, visual_options_draw), 0x758CB2);
        at(offset_of!(DrawLists, draw_758cba), 0x758CBA);
        at(offset_of!(DrawLists, draw_758cc2), 0x758CC2);
        at(offset_of!(DrawLists, draw_758cca), 0x758CCA);
        at(offset_of!(DrawLists, draw_758cd2), 0x758CD2);
        at(offset_of!(DrawLists, car_specification_draw), 0x758CDA);
        at(offset_of!(DrawLists, slideshow_draw), 0x758CEA);
        at(offset_of!(DrawLists, view_courses_draw), 0x758CF6);
        at(
            offset_of!(DrawLists, time_trial_stage_details_draw),
            0x758CFE,
        );
        at(offset_of!(DrawLists, view_best_times_draw), 0x758D06);
        at(offset_of!(DrawLists, view_best_scores_draw), 0x758D0E);
        at(offset_of!(DrawLists, best_times_draw), 0x758D1A);
        at(offset_of!(DrawLists, draw_758d22), 0x758D22);
        at(offset_of!(DrawLists, draw_758d2a), 0x758D2A);
        at(offset_of!(DrawLists, draw_758d32), 0x758D32);
        at(offset_of!(DrawLists, draw_758d3a), 0x758D3A);
        at(offset_of!(DrawLists, draw_758d42), 0x758D42);
        at(offset_of!(DrawLists, list_758d4a), 0x758D4A);
        at(offset_of!(DrawLists, welcome_draw), 0x758D4E);
        at(offset_of!(DrawLists, list_758d52), 0x758D52);
        at(offset_of!(DrawLists, list_758d56), 0x758D56);
        at(offset_of!(DrawLists, review_options_47d5a5_draw), 0x758D5E);
        at(offset_of!(DrawLists, review_options_480d46_draw), 0x758D62);
        at(offset_of!(DrawLists, review_options_48108e_draw), 0x758D66);
        at(offset_of!(DrawLists, review_options_481429_draw), 0x758D6A);
        at(offset_of!(DrawLists, service_area_draw), 0x758D6E);
        at(offset_of!(DrawLists, list_758d76), 0x758D76);
        at(offset_of!(DrawLists, list_758d7a), 0x758D7A);
        at(offset_of!(DrawLists, next_stage_draw), 0x758D7E);
        at(offset_of!(DrawLists, view_stage_draw), 0x758D86);
        at(offset_of!(DrawLists, draw_758d8e), 0x758D8E);
        at(offset_of!(DrawLists, draw_758d96), 0x758D96);
        at(offset_of!(DrawLists, draw_758da2), 0x758DA2);
        at(offset_of!(DrawLists, draw_758daa), 0x758DAA);
        at(offset_of!(DrawLists, draw_758db6), 0x758DB6);
        at(offset_of!(DrawLists, draw_758dbe), 0x758DBE);
        at(offset_of!(DrawLists, draw_758dca), 0x758DCA);
        at(offset_of!(DrawLists, draw_758dd2), 0x758DD2);
        at(offset_of!(DrawLists, inspection_area_draw), 0x758DDE);
        at(offset_of!(DrawLists, draw_758de2), 0x758DE2);
        at(offset_of!(DrawLists, draw_758dee), 0x758DEE);
        at(offset_of!(DrawLists, draw_758dfa), 0x758DFA);
        at(offset_of!(DrawLists, draw_758e06), 0x758E06);
        at(offset_of!(DrawLists, list_758e12), 0x758E12);
        at(offset_of!(DrawLists, list_758e1e), 0x758E1E);
        at(offset_of!(DrawLists, list_758e2a), 0x758E2A);
        at(offset_of!(DrawLists, list_758e36), 0x758E36);
        at(offset_of!(DrawLists, list_758e42), 0x758E42);
        at(offset_of!(DrawLists, list_758e46), 0x758E46);
        at(offset_of!(DrawLists, list_758e4e), 0x758E4E);
        at(
            offset_of!(DrawLists, class_championship_standings_draw),
            0x758E56,
        );
        at(offset_of!(DrawLists, list_758e5e), 0x758E5E);
        at(offset_of!(DrawLists, screen_488f4c_draw), 0x758E66);
        at(offset_of!(DrawLists, list_758e6e), 0x758E6E);
        at(offset_of!(DrawLists, draw_758e76), 0x758E76);
        at(offset_of!(DrawLists, draw_758e7e), 0x758E7E);
        at(offset_of!(DrawLists, draw_758e86), 0x758E86);
        at(offset_of!(DrawLists, draw_758e8e), 0x758E8E);
        at(offset_of!(DrawLists, draw_758e96), 0x758E96);
        at(offset_of!(DrawLists, select_stage_draw), 0x758E9E);
        at(
            offset_of!(DrawLists, time_trial_select_stage_draw),
            0x758EA6,
        );
        at(offset_of!(DrawLists, list_758eae), 0x758EAE);
        at(offset_of!(DrawLists, review_options_485f5e_draw), 0x758EBA);
        at(offset_of!(DrawLists, review_options_4882e2_draw), 0x758EBE);
        at(offset_of!(DrawLists, review_options_48861d_draw), 0x758EC2);
        at(offset_of!(DrawLists, arcade_review_options_draw), 0x758EC6);
        at(offset_of!(DrawLists, list_758eca), 0x758ECA);
        at(offset_of!(DrawLists, draw_758ece), 0x758ECE);
        at(offset_of!(DrawLists, list_758ed6), 0x758ED6);
        at(offset_of!(DrawLists, list_758eda), 0x758EDA);
        at(offset_of!(DrawLists, main_menu_scene), 0x758EE6);
        at(offset_of!(DrawLists, start_rally_scene), 0x758EEE);
        at(offset_of!(DrawLists, simple_select_players_scene), 0x758EF6);
        at(offset_of!(DrawLists, arcade_select_level_scene), 0x758EFE);
        at(offset_of!(DrawLists, list_758f06), 0x758F06);
        at(offset_of!(DrawLists, load_championship_scene), 0x758F0E);
        at(offset_of!(DrawLists, save_setup_scene), 0x758F12);
        at(
            offset_of!(DrawLists, load_championship_46e899_scene),
            0x758F1A,
        );
        at(offset_of!(DrawLists, save_championship_scene), 0x758F22);
        at(offset_of!(DrawLists, save_replay_scene), 0x758F26);
        at(offset_of!(DrawLists, load_replay_scene), 0x758F2A);
        at(offset_of!(DrawLists, list_758f2e), 0x758F2E);
        at(offset_of!(DrawLists, list_758f36), 0x758F36);
        at(
            offset_of!(DrawLists, time_trial_select_players_scene),
            0x758F3E,
        );
        at(
            offset_of!(DrawLists, championship_new_simulation_scene),
            0x758F46,
        );
        at(offset_of!(DrawLists, new_simulation_scene), 0x758F4E);
        at(offset_of!(DrawLists, network_rally_scene), 0x758F56);
        at(offset_of!(DrawLists, select_car_scene), 0x758F5E);
        at(offset_of!(DrawLists, car_specs_47af4c_scene), 0x758F66);
        at(offset_of!(DrawLists, list_758f6e), 0x758F6E);
        at(offset_of!(DrawLists, car_specification_scene), 0x758F76);
        at(offset_of!(DrawLists, slideshow_scene), 0x758F7E);
        at(offset_of!(DrawLists, screen_483163_scene), 0x758F82);
        at(offset_of!(DrawLists, results_48584b_scene), 0x758F86);
        at(offset_of!(DrawLists, review_options_485f5e_scene), 0x758F8A);
        at(offset_of!(DrawLists, list_758f92), 0x758F92);
        at(offset_of!(DrawLists, review_options_4882e2_scene), 0x758F96);
        at(offset_of!(DrawLists, review_options_48861d_scene), 0x758F9E);
        at(offset_of!(DrawLists, arcade_review_options_scene), 0x758FA6);
        at(offset_of!(DrawLists, list_758fae), 0x758FAE);
        at(offset_of!(DrawLists, best_scores_48a69e_scene), 0x758FB2);
        at(offset_of!(DrawLists, best_class_scores_scene), 0x758FB6);
        at(offset_of!(DrawLists, best_overall_scores_scene), 0x758FBA);
        at(
            offset_of!(DrawLists, time_trial_select_stage_scene),
            0x758FBE,
        );
        at(offset_of!(DrawLists, time_trial_best_times_scene), 0x758FC2);
        at(offset_of!(DrawLists, time_trial_results_scene), 0x758FC6);
        at(offset_of!(DrawLists, view_best_times_scene), 0x758FCA);
        at(offset_of!(DrawLists, view_best_scores_scene), 0x758FCE);
        at(offset_of!(DrawLists, best_times_scene), 0x758FD2);
        at(offset_of!(DrawLists, arcade_next_stage_scene), 0x758FD6);
        at(offset_of!(DrawLists, list_758fda), 0x758FDA);
        at(offset_of!(DrawLists, options_scene), 0x758FDE);
        at(offset_of!(DrawLists, player_details_scene), 0x758FE6);
        at(offset_of!(DrawLists, information_scene), 0x758FEE);
        at(offset_of!(DrawLists, list_758ff6), 0x758FF6);
        at(offset_of!(DrawLists, audio_setup_scene), 0x758FFA);
        at(offset_of!(DrawLists, advanced_audio_scene), 0x758FFE);
        at(offset_of!(DrawLists, leave_question_scene), 0x759002);
        at(offset_of!(DrawLists, visual_preferences_scene), 0x759006);
        at(offset_of!(DrawLists, visual_options_scene), 0x75900A);
        at(offset_of!(DrawLists, player1_details_scene), 0x75900E);
        at(offset_of!(DrawLists, list_759012), 0x759012);
        at(offset_of!(DrawLists, player2_details_scene), 0x759016);
        at(offset_of!(DrawLists, list_75901a), 0x75901A);
        at(offset_of!(DrawLists, player3_details_scene), 0x75901E);
        at(offset_of!(DrawLists, list_759022), 0x759022);
        at(offset_of!(DrawLists, player4_details_scene), 0x759026);
        at(offset_of!(DrawLists, list_75902a), 0x75902A);
        at(offset_of!(DrawLists, view_courses_scene), 0x75902E);
        at(offset_of!(DrawLists, list_759032), 0x759032);
        at(
            offset_of!(DrawLists, time_trial_stage_details_scene),
            0x759036,
        );
        at(offset_of!(DrawLists, list_75903a), 0x75903A);
        at(offset_of!(DrawLists, arcade_best_times_scene), 0x75903E);
        at(offset_of!(DrawLists, list_759042), 0x759042);
        at(offset_of!(DrawLists, list_759046), 0x759046);
        at(offset_of!(DrawLists, in_game_options_scene), 0x75904A);
        at(offset_of!(DrawLists, select_stage_scene), 0x759052);
        at(offset_of!(DrawLists, results_482e9b_scene), 0x759056);
        at(offset_of!(DrawLists, welcome_scene), 0x75905A);
        at(offset_of!(DrawLists, rally_stages_scene), 0x75905E);
        at(offset_of!(DrawLists, review_options_47d5a5_scene), 0x759062);
        at(offset_of!(DrawLists, review_options_480d46_scene), 0x75906A);
        at(offset_of!(DrawLists, review_options_48108e_scene), 0x759072);
        at(offset_of!(DrawLists, review_options_481429_scene), 0x75907A);
        at(offset_of!(DrawLists, service_area_scene), 0x759082);
        at(offset_of!(DrawLists, list_75908a), 0x75908A);
        at(offset_of!(DrawLists, list_75908e), 0x75908E);
        at(offset_of!(DrawLists, next_stage_scene), 0x759092);
        at(offset_of!(DrawLists, view_stage_scene), 0x759096);
        at(offset_of!(DrawLists, player1_controller_scene), 0x75909A);
        at(offset_of!(DrawLists, player1_buttons_scene), 0x75909E);
        at(offset_of!(DrawLists, player2_controller_scene), 0x7590A2);
        at(offset_of!(DrawLists, player2_buttons_scene), 0x7590A6);
        at(offset_of!(DrawLists, player3_controller_scene), 0x7590AA);
        at(offset_of!(DrawLists, player3_buttons_scene), 0x7590AE);
        at(offset_of!(DrawLists, player4_controller_scene), 0x7590B2);
        at(offset_of!(DrawLists, player4_buttons_scene), 0x7590B6);
        at(offset_of!(DrawLists, inspection_area_scene), 0x7590BA);
        at(offset_of!(DrawLists, redefine_keys_player1_scene), 0x7590C2);
        at(offset_of!(DrawLists, redefine_keys_player2_scene), 0x7590C6);
        at(offset_of!(DrawLists, redefine_keys_player3_scene), 0x7590CA);
        at(offset_of!(DrawLists, redefine_keys_player4_scene), 0x7590CE);
        at(offset_of!(DrawLists, class_stage_results_scene), 0x7590D2);
        at(offset_of!(DrawLists, overall_stage_results_scene), 0x7590D6);
        at(offset_of!(DrawLists, class_rally_results_scene), 0x7590DA);
        at(offset_of!(DrawLists, overall_rally_results_scene), 0x7590DE);
        at(offset_of!(DrawLists, list_7590e2), 0x7590E2);
        at(offset_of!(DrawLists, list_7590e6), 0x7590E6);
        at(offset_of!(DrawLists, screen_488f4c_scene), 0x7590EA);
        at(offset_of!(DrawLists, list_7590ee), 0x7590EE);
        at(
            offset_of!(DrawLists, class_championship_standings_scene),
            0x7590F2,
        );
        at(
            offset_of!(DrawLists, championship_standings_scene),
            0x7590F6,
        );
        at(offset_of!(DrawLists, arcade_results_scene), 0x7590FA);
        at(offset_of!(DrawLists, stage_result_scene), 0x7590FE);
        at(offset_of!(DrawLists, list_759102), 0x759102);
        at(offset_of!(DrawLists, list_759106), 0x759106);
        at(offset_of!(DrawLists, list_75910a), 0x75910A);
        at(offset_of!(DrawLists, list_75910e), 0x75910E);
        at(offset_of!(DrawLists, list_759112), 0x759112);
        at(offset_of!(DrawLists, list_759116), 0x759116);
        at(offset_of!(DrawLists, list_75911e), 0x75911E);
        at(offset_of!(DrawLists, main_menu_overlay), 0x759122);
        at(offset_of!(DrawLists, start_rally_overlay), 0x75912A);
        at(
            offset_of!(DrawLists, simple_select_players_overlay),
            0x759136,
        );
        at(offset_of!(DrawLists, list_75913e), 0x75913E);
        at(offset_of!(DrawLists, overlay_759146), 0x759146);
        at(offset_of!(DrawLists, overlay_75914a), 0x75914A);
        at(offset_of!(DrawLists, overlay_75914e), 0x75914E);
        at(offset_of!(DrawLists, overlay_759152), 0x759152);
        at(offset_of!(DrawLists, save_setup_overlay), 0x759156);
        at(
            offset_of!(DrawLists, load_championship_46e899_overlay),
            0x75915E,
        );
        at(offset_of!(DrawLists, list_759166), 0x759166);
        at(offset_of!(DrawLists, arcade_select_level_overlay), 0x75916E);
        at(offset_of!(DrawLists, list_75917a), 0x75917A);
        at(
            offset_of!(DrawLists, time_trial_select_players_overlay),
            0x759182,
        );
        at(
            offset_of!(DrawLists, championship_new_simulation_overlay),
            0x75918A,
        );
        at(offset_of!(DrawLists, new_simulation_overlay), 0x759192);
        at(offset_of!(DrawLists, network_rally_overlay), 0x75919A);
        at(offset_of!(DrawLists, select_car_overlay), 0x7591A2);
        at(offset_of!(DrawLists, car_specs_47af4c_overlay), 0x7591AA);
        at(offset_of!(DrawLists, list_7591b2), 0x7591B2);
        at(offset_of!(DrawLists, list_7591ba), 0x7591BA);
        at(offset_of!(DrawLists, arcade_next_stage_overlay), 0x7591BE);
        at(offset_of!(DrawLists, options_overlay), 0x7591C2);
        at(offset_of!(DrawLists, player_details_overlay), 0x7591C6);
        at(offset_of!(DrawLists, information_overlay), 0x7591CA);
        at(offset_of!(DrawLists, list_7591ce), 0x7591CE);
        at(offset_of!(DrawLists, list_7591d2), 0x7591D2);
        at(offset_of!(DrawLists, overlay_7591d6), 0x7591D6);
        at(offset_of!(DrawLists, advanced_audio_overlay), 0x7591DA);
        at(offset_of!(DrawLists, overlay_7591de), 0x7591DE);
        at(offset_of!(DrawLists, overlay_7591e2), 0x7591E2);
        at(offset_of!(DrawLists, visual_options_overlay), 0x7591E6);
        at(offset_of!(DrawLists, overlay_7591ea), 0x7591EA);
        at(offset_of!(DrawLists, overlay_7591ee), 0x7591EE);
        at(offset_of!(DrawLists, overlay_7591f2), 0x7591F2);
        at(offset_of!(DrawLists, overlay_7591f6), 0x7591F6);
        at(offset_of!(DrawLists, car_specification_overlay), 0x7591FA);
        at(offset_of!(DrawLists, slideshow_overlay), 0x759202);
        at(offset_of!(DrawLists, view_courses_overlay), 0x759206);
        at(
            offset_of!(DrawLists, time_trial_stage_details_overlay),
            0x75920A,
        );
        at(offset_of!(DrawLists, view_best_times_overlay), 0x75920E);
        at(offset_of!(DrawLists, view_best_scores_overlay), 0x759212);
        at(offset_of!(DrawLists, best_times_overlay), 0x759216);
        at(offset_of!(DrawLists, overlay_75921a), 0x75921A);
        at(offset_of!(DrawLists, overlay_75921e), 0x75921E);
        at(offset_of!(DrawLists, overlay_759222), 0x759222);
        at(offset_of!(DrawLists, overlay_759226), 0x759226);
        at(offset_of!(DrawLists, overlay_75922a), 0x75922A);
        at(offset_of!(DrawLists, list_75922e), 0x75922E);
        at(offset_of!(DrawLists, welcome_overlay), 0x759232);
        at(offset_of!(DrawLists, rally_stages_overlay), 0x759236);
        at(
            offset_of!(DrawLists, review_options_47d5a5_overlay),
            0x75923A,
        );
        at(
            offset_of!(DrawLists, review_options_480d46_overlay),
            0x75923E,
        );
        at(
            offset_of!(DrawLists, review_options_48108e_overlay),
            0x759242,
        );
        at(
            offset_of!(DrawLists, review_options_481429_overlay),
            0x759246,
        );
        at(offset_of!(DrawLists, service_area_overlay), 0x75924A);
        at(offset_of!(DrawLists, list_759252), 0x759252);
        at(offset_of!(DrawLists, next_stage_overlay), 0x759256);
        at(offset_of!(DrawLists, view_stage_overlay), 0x75925A);
        at(offset_of!(DrawLists, overlay_75925e), 0x75925E);
        at(offset_of!(DrawLists, overlay_759262), 0x759262);
        at(offset_of!(DrawLists, overlay_759266), 0x759266);
        at(offset_of!(DrawLists, overlay_75926a), 0x75926A);
        at(offset_of!(DrawLists, overlay_75926e), 0x75926E);
        at(offset_of!(DrawLists, overlay_759272), 0x759272);
        at(offset_of!(DrawLists, overlay_759276), 0x759276);
        at(offset_of!(DrawLists, overlay_75927a), 0x75927A);
        at(offset_of!(DrawLists, inspection_area_overlay), 0x75927E);
        at(offset_of!(DrawLists, overlay_759286), 0x759286);
        at(offset_of!(DrawLists, overlay_75928a), 0x75928A);
        at(offset_of!(DrawLists, overlay_75928e), 0x75928E);
        at(offset_of!(DrawLists, overlay_759292), 0x759292);
        at(offset_of!(DrawLists, class_stage_results_overlay), 0x759296);
        at(
            offset_of!(DrawLists, overall_stage_results_overlay),
            0x75929A,
        );
        at(offset_of!(DrawLists, class_rally_results_overlay), 0x75929E);
        at(
            offset_of!(DrawLists, overall_rally_results_overlay),
            0x7592A2,
        );
        at(offset_of!(DrawLists, list_7592a6), 0x7592A6);
        at(offset_of!(DrawLists, list_7592aa), 0x7592AA);
        at(offset_of!(DrawLists, list_7592ae), 0x7592AE);
        at(
            offset_of!(DrawLists, class_championship_standings_overlay),
            0x7592B2,
        );
        at(offset_of!(DrawLists, overlay_7592b6), 0x7592B6);
        at(offset_of!(DrawLists, screen_488f4c_overlay), 0x7592BA);
        at(offset_of!(DrawLists, overlay_7592be), 0x7592BE);
        at(offset_of!(DrawLists, overlay_7592c2), 0x7592C2);
        at(offset_of!(DrawLists, list_7592c6), 0x7592C6);
        at(offset_of!(DrawLists, overlay_7592ca), 0x7592CA);
        at(offset_of!(DrawLists, overlay_7592ce), 0x7592CE);
        at(offset_of!(DrawLists, overlay_7592d2), 0x7592D2);
        at(offset_of!(DrawLists, select_stage_overlay), 0x7592D6);
        at(
            offset_of!(DrawLists, time_trial_select_stage_overlay),
            0x7592DA,
        );
        at(offset_of!(DrawLists, list_7592de), 0x7592DE);
        at(offset_of!(DrawLists, list_7592e2), 0x7592E2);
        at(offset_of!(DrawLists, in_game_options_overlay), 0x7592E6);
        at(
            offset_of!(DrawLists, review_options_485f5e_overlay),
            0x7592EA,
        );
        at(
            offset_of!(DrawLists, review_options_4882e2_overlay),
            0x7592EE,
        );
        at(
            offset_of!(DrawLists, review_options_48861d_overlay),
            0x7592F2,
        );
        at(
            offset_of!(DrawLists, arcade_review_options_overlay),
            0x7592F6,
        );
        at(offset_of!(DrawLists, list_7592fa), 0x7592FA);
        at(offset_of!(DrawLists, overlay_7592fe), 0x7592FE);
        at(offset_of!(DrawLists, list_759302), 0x759302);
        at(offset_of!(DrawLists, view_best_times_setup), 0x75930A);
        at(offset_of!(DrawLists, view_best_scores_setup), 0x759312);
        at(offset_of!(DrawLists, best_times_setup), 0x75931A);
        at(offset_of!(DrawLists, select_stage_setup), 0x75931E);
        at(
            offset_of!(DrawLists, time_trial_select_stage_setup),
            0x759326,
        );
        at(offset_of!(DrawLists, view_courses_setup), 0x75932E);
        at(offset_of!(DrawLists, list_759336), 0x759336);
        at(offset_of!(DrawLists, list_75933a), 0x75933A);
        at(offset_of!(DrawLists, list_75933e), 0x75933E);
        at(offset_of!(DrawLists, list_759342), 0x759342);
        at(offset_of!(DrawLists, list_759346), 0x759346);
        at(offset_of!(DrawLists, main_menu_setup), 0x75934A);
        at(offset_of!(DrawLists, list_759352), 0x759352);
        at(offset_of!(DrawLists, list_759356), 0x759356);
        at(offset_of!(DrawLists, list_75935a), 0x75935A);
        at(offset_of!(DrawLists, list_75935e), 0x75935E);
        at(offset_of!(DrawLists, list_759362), 0x759362);
        at(offset_of!(DrawLists, list_759366), 0x759366);
        at(offset_of!(DrawLists, list_75936a), 0x75936A);
        at(offset_of!(DrawLists, list_75936e), 0x75936E);
        at(offset_of!(DrawLists, list_759372), 0x759372);
        at(offset_of!(DrawLists, list_759376), 0x759376);
        at(offset_of!(DrawLists, list_75937a), 0x75937A);
        at(offset_of!(DrawLists, list_75937e), 0x75937E);
        at(offset_of!(DrawLists, list_759382), 0x759382);
        at(offset_of!(DrawLists, list_759386), 0x759386);
        at(offset_of!(DrawLists, list_75938a), 0x75938A);
        at(offset_of!(DrawLists, list_75938e), 0x75938E);
        at(offset_of!(DrawLists, list_759392), 0x759392);
        at(offset_of!(DrawLists, list_759396), 0x759396);
        at(offset_of!(DrawLists, list_75939a), 0x75939A);
        at(offset_of!(DrawLists, list_75939e), 0x75939E);
        at(offset_of!(DrawLists, advanced_audio_setup), 0x7593A2);
        at(offset_of!(DrawLists, list_7593aa), 0x7593AA);
        at(offset_of!(DrawLists, list_7593ae), 0x7593AE);
        at(offset_of!(DrawLists, list_7593b2), 0x7593B2);
        at(offset_of!(DrawLists, list_7593b6), 0x7593B6);
        at(offset_of!(DrawLists, list_7593ba), 0x7593BA);
        at(offset_of!(DrawLists, list_7593be), 0x7593BE);
        at(offset_of!(DrawLists, list_7593c2), 0x7593C2);
        at(offset_of!(DrawLists, car_specification_setup), 0x7593C6);
        at(offset_of!(DrawLists, slideshow_setup), 0x7593D2);
        at(offset_of!(DrawLists, list_7593de), 0x7593DE);
        at(offset_of!(DrawLists, list_7593e2), 0x7593E2);
        at(offset_of!(DrawLists, list_7593e6), 0x7593E6);
        at(offset_of!(DrawLists, list_7593ea), 0x7593EA);
        at(offset_of!(DrawLists, list_7593ee), 0x7593EE);
        at(offset_of!(DrawLists, list_7593f2), 0x7593F2);
        at(offset_of!(DrawLists, list_7593f6), 0x7593F6);
        at(offset_of!(DrawLists, list_7593fa), 0x7593FA);
        at(offset_of!(DrawLists, select_car_setup), 0x7593FE);
        at(offset_of!(DrawLists, car_specs_47af4c_setup), 0x75940E);
        at(offset_of!(DrawLists, list_75941a), 0x75941A);
        at(offset_of!(DrawLists, list_759426), 0x759426);
        at(offset_of!(DrawLists, list_75942a), 0x75942A);
        at(offset_of!(DrawLists, list_75942e), 0x75942E);
        at(offset_of!(DrawLists, list_759432), 0x759432);
        at(offset_of!(DrawLists, service_area_setup), 0x759436);
        at(offset_of!(DrawLists, list_75943e), 0x75943E);
        at(offset_of!(DrawLists, list_759442), 0x759442);
        at(offset_of!(DrawLists, list_759446), 0x759446);
        at(offset_of!(DrawLists, load_championship_setup), 0x75944A);
        at(offset_of!(DrawLists, save_championship_setup), 0x759452);
        at(offset_of!(DrawLists, save_replay_setup), 0x75945A);
        at(offset_of!(DrawLists, load_replay_setup), 0x759462);
        at(offset_of!(DrawLists, list_75946a), 0x75946A);
        at(offset_of!(DrawLists, list_75946e), 0x75946E);
        at(offset_of!(DrawLists, list_759472), 0x759472);
        at(offset_of!(DrawLists, list_759476), 0x759476);
        at(offset_of!(DrawLists, list_75947a), 0x75947A);
        at(offset_of!(DrawLists, player1_controller_setup), 0x75947E);
        at(offset_of!(DrawLists, player1_buttons_setup), 0x759486);
        at(offset_of!(DrawLists, player2_controller_setup), 0x75948A);
        at(offset_of!(DrawLists, player2_buttons_setup), 0x759492);
        at(offset_of!(DrawLists, player3_controller_setup), 0x759496);
        at(offset_of!(DrawLists, player3_buttons_setup), 0x75949E);
        at(offset_of!(DrawLists, player4_controller_setup), 0x7594A2);
    }
}
