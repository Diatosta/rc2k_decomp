//! Which Rust function runs each function the menus' items and the draw lists name (typed
//! tables, `rc2k_core::menu::tables`); `table_calls.rs` calls these. Kept by hand. An item's
//! function gets an `ItemCall` (its value, item, menu and the carried values), a screen
//! function nothing.

use crate::errors::{BadInput, Refused};
use crate::menu::advanced_audio_loop;
use crate::menu::apply_screen_mode;
use crate::menu::arcade_levels;
use crate::menu::arcade_select_level_loop;
use crate::menu::ask_to_leave;
use crate::menu::audio_setup_loop;
use crate::menu::begin_player_name;
use crate::menu::begin_replay_name;
use crate::menu::begin_savegame_name;
use crate::menu::car_buttons::Which;
use crate::menu::car_choice;
use crate::menu::car_specification_loop;
use crate::menu::championship_new_simulation_loop;
use crate::menu::championship_standings;
use crate::menu::choose_key;
use crate::menu::clear_choice_marks;
use crate::menu::controller_items;
use crate::menu::flip_codriver;
use crate::menu::flip_gearing;
use crate::menu::flip_yes_no;
use crate::menu::go_back;
use crate::menu::in_game_options_loop;
use crate::menu::information_loop;
use crate::menu::item_actions;
use crate::menu::learn_bindings;
use crate::menu::load_championship_loop;
use crate::menu::load_championship_table_loop;
use crate::menu::lower_sensitivity;
use crate::menu::network_rally_loop;
use crate::menu::new_simulation_loop;
use crate::menu::next_stage_loop;
use crate::menu::option_items::{
    self, next_audio_quality, previous_audio_quality, previous_screen_mode,
};
use crate::menu::option_steps::{
    next_screen_mode, start_pick_a, start_pick_b, step_texture_setting,
};
use crate::menu::options_loop;
use crate::menu::player;
use crate::menu::player_actions;
use crate::menu::player_details_loop;
use crate::menu::player_settings;
use crate::menu::player1_buttons_loop;
use crate::menu::player1_controller_loop;
use crate::menu::player1_details_loop;
use crate::menu::player2_buttons_loop;
use crate::menu::player2_controller_loop;
use crate::menu::player2_details_loop;
use crate::menu::player3_buttons_loop;
use crate::menu::player3_controller_loop;
use crate::menu::player3_details_loop;
use crate::menu::player4_buttons_loop;
use crate::menu::player4_controller_loop;
use crate::menu::player4_details_loop;
use crate::menu::raise_sensitivity;
use crate::menu::rally_results;
use crate::menu::redefine_keys;
use crate::menu::redefine_keys_player1_loop;
use crate::menu::redefine_keys_player2_loop;
use crate::menu::redefine_keys_player3_loop;
use crate::menu::redefine_keys_player4_loop;
use crate::menu::replay_items;
use crate::menu::save_championship_loop;
use crate::menu::save_setup_loop;
use crate::menu::screen_loop::Entry;
use crate::menu::simple_select_players_loop;
use crate::menu::slideshow_loop;
use crate::menu::stage_choice;
use crate::menu::stage_range_caption;
use crate::menu::stage_rows;
use crate::menu::stage_view_steps;
use crate::menu::start_events;
use crate::menu::start_rally_loop;
use crate::menu::step_carousel;
use crate::menu::step_items;
use crate::menu::switch_stage_group;
use crate::menu::table_calls;
use crate::menu::time_trial_select_players_loop;
use crate::menu::view_best_scores_loop;
use crate::menu::view_best_times_loop;
use crate::menu::view_courses_loop;
use crate::menu::visual_options_loop;
use crate::menu::visual_preferences_loop;
use crate::net::settings_loop;
use crate::net::start_link;
use crate::owners::Owners;
use crate::saves::load_savegame;
use crate::scene::draw_car_object;
use crate::scene::draw_highlighted_object;
use crate::scene::draw_main_menu_objects;
use crate::screens::best_scores;
use crate::screens::car_caption;
use crate::screens::car_page_title;
use crate::screens::championship_table;
use crate::screens::controller;
use crate::screens::draw_arcade_stage_title;
use crate::screens::draw_background_band;
use crate::screens::draw_bottom_panel;
use crate::screens::draw_car_figures;
use crate::screens::draw_championship_standings;
use crate::screens::draw_class_ranking;
use crate::screens::draw_class_times_marked;
use crate::screens::draw_control_bindings;
use crate::screens::draw_duel_result;
use crate::screens::draw_event_best_scores;
use crate::screens::draw_graphics_settings;
use crate::screens::draw_highlighted_caption;
use crate::screens::draw_leave_question;
use crate::screens::draw_load_setup_list;
use crate::screens::draw_note_1fd;
use crate::screens::draw_options_checklist;
use crate::screens::draw_overall_best_scores;
use crate::screens::draw_player_details;
use crate::screens::draw_player_rally_result;
use crate::screens::draw_player_stage_result;
use crate::screens::draw_players_results;
use crate::screens::draw_players_results_points;
use crate::screens::draw_players_standings;
use crate::screens::draw_rally_stage_times;
use crate::screens::draw_redefined_keys;
use crate::screens::draw_replay_slot_names;
use crate::screens::draw_save_setup_list;
use crate::screens::draw_savegame_names;
use crate::screens::draw_savegame_preview;
use crate::screens::draw_section_best_times;
use crate::screens::draw_selected_stage_menu;
use crate::screens::draw_service_area;
use crate::screens::draw_service_boxes;
use crate::screens::draw_service_stage_info;
use crate::screens::draw_sound_settings;
use crate::screens::draw_stage_info;
use crate::screens::draw_stage_times;
use crate::screens::draw_time_trial_stage_info;
use crate::screens::draw_time_trial_stage_select;
use crate::screens::draw_two_column_panel;
use crate::screens::draw_visual_preferences;
use crate::screens::draw_volume_settings;
use crate::screens::gearing_text;
use crate::screens::itinerary;
use crate::screens::locked_notes;
use crate::screens::page_settings;
use crate::screens::player_number_text;
use crate::screens::results_pages::class_rally;
use crate::screens::results_pages::class_stage;
use crate::screens::results_pages::overall_rally;
use crate::screens::results_pages::overall_stage;
use crate::screens::savegame_page_numbers;
use crate::screens::set_stage_choice_titles;
use crate::screens::slideshow_title;
use crate::screens::stage_info;
use crate::screens::stage_results;
use crate::screens::yes_no_caption;
use crate::service::begin_edits;
use crate::service::begin_setup_name;
use crate::service::handle_edit_keys;
use crate::service::load_setup_slot;
use crate::service::load_setup_values;
use crate::service::next_setup_page;
use crate::service::toggle_repair_choice;
use crate::text::draw_text::GameCanvas;
use rc2k_core::menu::tables::{ItemFn, ScreenFn};

/// Runs item function `f` with what the item hands it (`c`).
pub fn item_fn(o: Owners<'static>, f: ItemFn, c: &table_calls::ItemCall) -> Result<(), Refused> {
    let menus = o.menus;
    let race = o.race;
    match f {
        ItemFn::StepSetting { setting, up } => step_items::player_setting(menus, setting, up),
        ItemFn::Toggle(setting) => step_items::player_setting(menus, setting, true),
        ItemFn::Volume { which, up } => option_items::volume(o, which, up),
        ItemFn::OptionSwitch(which) => option_items::switch(o, which),
        ItemFn::StageCounter { which, up } => option_items::stage_counter(o, which, up),
        ItemFn::ChooseKey(row) => {
            choose_key::choose_key(&menus.screens, u32::from(row));
            Ok(())
        }
        ItemFn::LearnBinding(button) => {
            learn_bindings::learn_binding(o.screens, u32::from(button));
            Ok(())
        }
        ItemFn::PlayerCarSpecs(n) => {
            player_actions::player_car_specs(menus, u32::from(n), &Entry::new(o))
        }
        ItemFn::ArcadeLevel(level) => arcade_levels::arcade_level(o, u32::from(level)),
        ItemFn::StartEvent(event, seats) => start_events::start_event_item(o, event, seats),
        ItemFn::FlipCodriver => flip_codriver::flip_codriver(menus).map(|_| ()),
        ItemFn::BeginPlayerName => begin_player_name::begin_player_name(menus).map(|_| ()),
        ItemFn::FlipGearing => Ok(flip_gearing::flip_gearing(&menus.screens, o.race)?),
        ItemFn::NextGearing => gearing_text::step_gearing(menus, o.race.text_buffers(), true),
        ItemFn::PreviousGearing => gearing_text::step_gearing(menus, o.race.text_buffers(), false),
        ItemFn::PreviousCar => step_items::car_button(o, Which::PreviousCar),
        ItemFn::NextCar => step_items::car_button(o, Which::NextCar),
        ItemFn::PreviousSlideshowCar => step_items::car_button(o, Which::PreviousSlideshowCar),
        ItemFn::NextSlideshowCar => step_items::car_button(o, Which::NextSlideshowCar),
        ItemFn::PreviousCarPage => step_items::car_button(o, Which::PreviousPage),
        ItemFn::NextCarPage => step_items::car_button(o, Which::NextPage),
        ItemFn::PreviousSlideshowPicture => step_items::car_button(o, Which::PreviousPicture),
        ItemFn::NextSlideshowPicture => step_items::car_button(o, Which::NextPicture),
        ItemFn::NextController => controller_items::next_controller(menus, o.input).map(|_| ()),
        ItemFn::PreviousController => {
            controller_items::previous_controller(menus, o.input).map(|_| ())
        }
        ItemFn::KeepPlayerKeys => player_actions::keep_player_keys(menus),
        ItemFn::LowerPlayer40 => player_actions::lower_end_zone(menus),
        ItemFn::RaiseDeadZoneA => controller_items::raise_dead_zone(menus, false),
        ItemFn::LowerPlayer44 => player_actions::lower_centre_dead_zone(menus),
        ItemFn::RaiseDeadZoneB => controller_items::raise_dead_zone(menus, true),
        ItemFn::LowerControllerSensitivity => {
            controller_items::lower_controller_sensitivity(menus, o.input).map(|_| ())
        }
        ItemFn::RaiseControllerSensitivity => controller_items::raise_controller_sensitivity(
            menus,
            o.input,
            c.menu
                .ok_or(BadInput("no menu (the original reads +0x2C of edi)"))?
                .items()[0]
                .fixed()
                .links[1],
        ),
        ItemFn::RaisePlayer48 => player_actions::raise_strength(menus),
        ItemFn::LowerPlayer48 => player_actions::lower_strength(menus),
        ItemFn::BeginGearRatioEdit => begin_edits::begin_edit(menus, begin_edits::GEAR_RATIO),
        ItemFn::BeginTyreEdit => begin_edits::begin_edit(menus, begin_edits::TYRE),
        ItemFn::OpenSteeringBox => {
            item_actions::open_service_box(&menus.screens, 3);
            Ok(())
        }
        ItemFn::OpenBrakeBox => {
            item_actions::open_service_box(&menus.screens, 4);
            Ok(())
        }
        ItemFn::BeginRideHeightEdit => begin_edits::begin_edit(menus, begin_edits::RIDE_HEIGHT),
        ItemFn::BeginSuspensionStiffnessEdit => {
            begin_edits::begin_edit(menus, begin_edits::SUSPENSION_STIFFNESS)
        }
        ItemFn::ToggleRepairChoice => toggle_repair_choice::toggle_repair_choice(menus).map(|_| ()),
        ItemFn::NextScoresPage => {
            player_actions::next_scores_page(&menus.tables);
            Ok(())
        }
        ItemFn::PreviousScoresPage => {
            player_actions::previous_scores_page(&menus.tables);
            Ok(())
        }
        ItemFn::PreviousSavePage => {
            item_actions::previous_save_page(&menus.tables);
            Ok(())
        }
        ItemFn::NextSavegamePage => {
            step_items::next_savegame_page(&menus.tables);
            Ok(())
        }
        ItemFn::PreviousReplayRange => {
            player_actions::previous_replay_range(menus, &menus.tables).map_err(Into::into)
        }
        ItemFn::NextReplayRange => {
            player_actions::next_replay_range(menus, &menus.tables).map_err(Into::into)
        }
        ItemFn::BeginSavegameName => Ok(begin_savegame_name::begin_savegame_name(
            menus, o.saves, true,
        )?),
        ItemFn::LoadSavegame => load_savegame::load_savegame(o),
        ItemFn::LoadChampionshipTableLoop => load_championship_table_loop::run(o),
        ItemFn::SaveSetupLoop => save_setup_loop::run(o),
        ItemFn::BeginSetupName => Ok(begin_setup_name::begin_setup_name(menus, o.service)?),
        ItemFn::LoadSetupSlot => {
            load_setup_slot::load_setup_slot(menus, o.service, o.system).map(|_| ())
        }
        ItemFn::NextSetupPage => {
            next_setup_page::next_setup_page(menus);
            Ok(())
        }
        ItemFn::BeginReplayName => Ok(begin_replay_name::begin_replay_name(menus, o.race, true)?),
        ItemFn::FlipYesNo => {
            flip_yes_no::flip_yes_no(&menus.tables);
            Ok(())
        }
        ItemFn::PreviousStagePage => stage_view_steps::step_stage_page(o, false),
        ItemFn::NextStagePage => stage_view_steps::step_stage_page(o, true),
        ItemFn::PreviousScreenMode => previous_screen_mode::run(o),
        ItemFn::NextScreenMode => {
            next_screen_mode(&menus.screens, o.gfx);
            Ok(())
        }
        ItemFn::PreviousTextureSetting => {
            step_texture_setting(&menus.screens, false, race);
            Ok(())
        }
        ItemFn::NextTextureSetting => {
            step_texture_setting(&menus.screens, true, race);
            Ok(())
        }
        ItemFn::NextAudioQuality => next_audio_quality::run(o),
        ItemFn::PreviousAudioQuality => previous_audio_quality::run(o),
        ItemFn::ViewStageDetails => {
            item_actions::view_stage_details(&menus.screens, &Entry::new(o))
        }
        ItemFn::ApplyScreenMode => apply_screen_mode::apply_screen_mode(o),
        ItemFn::TogglePlayer3c => player_actions::toggle_controller_mode(menus),
        ItemFn::ReplayItemC => replay_items::replay_item_c(race, o),
        ItemFn::ReplayItemB => replay_items::replay_item_b(race, o),
        ItemFn::ReplayItemA => replay_items::replay_item_a(race, o),
        ItemFn::ToggleCaptionChoice => {
            player_actions::toggle_caption_choice(&menus.screens, o.race).map_err(Into::into)
        }
        ItemFn::ShowIntro => item_actions::show_intro(o),
        ItemFn::RaiseSensitivity => raise_sensitivity::raise_sensitivity(menus).map(|_| ()),
        ItemFn::LowerSensitivity => lower_sensitivity::lower_sensitivity(menus).map(|_| ()),
        ItemFn::NextColourA => step_items::colour(&menus.tables, start_pick_a(menus), true),
        ItemFn::PreviousColourA => step_items::colour(&menus.tables, start_pick_a(menus), false),
        ItemFn::NextColourB => step_items::colour(&menus.tables, start_pick_b(menus), true),
        ItemFn::PreviousColourB => step_items::colour(&menus.tables, start_pick_b(menus), false),
        ItemFn::Nothing => Ok(()),
        ItemFn::GoBack => {
            go_back::go_back(menus);
            Ok(())
        }
        ItemFn::LeaveWithCarry => {
            player_actions::leave_with_carry(menus);
            Ok(())
        }
        ItemFn::NextStageLoop => next_stage_loop::run(o),
        ItemFn::StartRallyLoop => start_rally_loop::run(o),
        ItemFn::NetworkRallyLoop => network_rally_loop::run(o),
        ItemFn::ChampionshipNewSimulationLoop => championship_new_simulation_loop::run(o),
        ItemFn::AudioSetupLoop => audio_setup_loop::run(o),
        ItemFn::OptionsLoop => options_loop::run(o),
        ItemFn::PlayerDetailsLoop => player_details_loop::run(o),
        ItemFn::Player1DetailsLoop => player1_details_loop::run(o),
        ItemFn::Player2DetailsLoop => player2_details_loop::run(o),
        ItemFn::Player3DetailsLoop => player3_details_loop::run(o),
        ItemFn::Player4DetailsLoop => player4_details_loop::run(o),
        ItemFn::Player1ControllerLoop => player1_controller_loop::run(o),
        ItemFn::Player1ButtonsLoop => player1_buttons_loop::run(o),
        ItemFn::Player2ControllerLoop => player2_controller_loop::run(o),
        ItemFn::Player2ButtonsLoop => player2_buttons_loop::run(o),
        ItemFn::Player3ControllerLoop => player3_controller_loop::run(o),
        ItemFn::Player3ButtonsLoop => player3_buttons_loop::run(o),
        ItemFn::Player4ControllerLoop => player4_controller_loop::run(o),
        ItemFn::Player4ButtonsLoop => player4_buttons_loop::run(o),
        ItemFn::RedefineKeysPlayer1Loop => redefine_keys_player1_loop::run(o),
        ItemFn::RedefineKeysPlayer2Loop => redefine_keys_player2_loop::run(o),
        ItemFn::RedefineKeysPlayer3Loop => redefine_keys_player3_loop::run(o),
        ItemFn::RedefineKeysPlayer4Loop => redefine_keys_player4_loop::run(o),
        ItemFn::VisualPreferencesLoop => visual_preferences_loop::run(o),
        ItemFn::CarSpecificationLoop => car_specification_loop::run(o),
        ItemFn::SlideshowLoop => slideshow_loop::run(o),
        ItemFn::PreviousChoiceCar => {
            car_choice::previous_choice_car(menus);
            Ok(())
        }
        ItemFn::NextChoiceCar => {
            car_choice::next_choice_car(menus);
            Ok(())
        }
        ItemFn::InGameOptionsLoop => in_game_options_loop::run(o),
        ItemFn::AskToLeave => {
            ask_to_leave::ask_to_leave(menus);
            Ok(())
        }
        ItemFn::LoadChampionshipLoop => load_championship_loop::run(o),
        ItemFn::SaveChampionshipLoop => save_championship_loop::run(o),
        ItemFn::VisualOptionsLoop => visual_options_loop::run(o),
        ItemFn::AdvancedAudioLoop => advanced_audio_loop::run(o),
        ItemFn::SimpleSelectPlayersLoop => simple_select_players_loop::run(o),
        ItemFn::SwitchStageGroup => switch_stage_group::run(o),
        ItemFn::NextStageSub => {
            stage_choice::next_stage_sub(menus, &mut || stage_choice::draw_stage_map(o))
        }
        ItemFn::PreviousStageSub => {
            stage_choice::previous_stage_sub(menus, &mut || stage_choice::draw_stage_map(o))
        }
        ItemFn::NextStageNumber => {
            stage_choice::next_stage_number(menus, &mut || stage_choice::draw_stage_map(o))
        }
        ItemFn::PreviousStageNumber => {
            stage_choice::previous_stage_number(menus, &mut || stage_choice::draw_stage_map(o))
        }
        ItemFn::ArcadeSelectLevelLoop => arcade_select_level_loop::run(o),
        ItemFn::StartArcade => item_actions::start_arcade(o),
        ItemFn::TimeTrialSelectPlayersLoop => time_trial_select_players_loop::run(o),
        ItemFn::InformationLoop => information_loop::run(o),
        ItemFn::ViewBestTimesLoop => view_best_times_loop::run(o),
        ItemFn::ViewCoursesLoop => view_courses_loop::run(o),
        ItemFn::ViewBestScoresLoop => view_best_scores_loop::run(o),
        ItemFn::PreviousStageRows => stage_rows::step_rows(menus, o.race, false, false),
        ItemFn::NextStageRows => stage_rows::step_rows(menus, o.race, false, true),
        ItemFn::PreviousStageRowsOf90 => stage_rows::step_rows(menus, o.race, true, false),
        ItemFn::NextStageRowsOf90 => stage_rows::step_rows(menus, o.race, true, true),
        ItemFn::RallyResults => rally_results::rally_results(menus, &Entry::new(o)),
        ItemFn::ChampionshipStandings => {
            championship_standings::championship_standings(menus, &Entry::new(o))
        }
        ItemFn::NewSimulationIfUnlocked => item_actions::new_simulation_if_unlocked(o),
        ItemFn::NewSimulationLoop => new_simulation_loop::run(o),
        ItemFn::NetworkSettingsItem => settings_loop::settings_loop(o),
        // (The original leaves eax the mode and the carry set: no caller reads them.)
        ItemFn::JoinNetworkGame => start_link::start_link(o, 2),
        ItemFn::HostNetworkGame => start_link::start_link(o, 1),
    }
}

/// Runs screen function `f` (the draw lists' and the frames'; it takes nothing).
pub fn screen_fn(o: Owners<'static>, f: ScreenFn) -> Result<(), Refused> {
    let menus = o.menus;
    let race = o.race;
    match f {
        ScreenFn::DrawSectionBestTimes => {
            draw_section_best_times::draw_section_best_times(menus, o.saves, &mut GameCanvas)
                .map(|_| ())
        }
        ScreenFn::DrawPlayer2Details => draw_player_details::draw_player_page(
            menus,
            o.input,
            draw_player_details::FULL,
            &mut GameCanvas,
        ),
        ScreenFn::DrawPlayer34Details => draw_player_details::draw_player_page(
            menus,
            o.input,
            draw_player_details::SHORT,
            &mut GameCanvas,
        ),
        ScreenFn::StageOptionsTexts => page_settings::stage_options_texts(o).map(|_| ()),
        ScreenFn::SetStageChoiceTitles => {
            set_stage_choice_titles::set_stage_choice_titles(menus, o.system, o.race).map(|_| ())
        }
        ScreenFn::StageChoiceTexts => page_settings::stage_choice_texts(o).map(|_| ()),
        ScreenFn::DrawHighlightedCaption => {
            // The original's x is the ecx `StepCarousel` leaves, which only the measuring
            // gets, in a font in measure mode (the option font): it changes nothing, so 0.
            draw_highlighted_caption::draw_highlighted_caption(o, 0, &mut GameCanvas)?;
            Ok(())
        }
        ScreenFn::CarCaption2 => car_caption::car_caption(menus, o.race.text_buffers()).map(|_| ()),
        ScreenFn::StepCarousel => Ok(step_carousel::step_carousel(&menus.screens, o.race)?),
        ScreenFn::DrawBackgroundBand => {
            draw_background_band::draw_background_band(menus, &mut GameCanvas)
        }
        ScreenFn::GearingText => {
            gearing_text::gearing_text(menus, o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::CarCaption => car_caption::car_caption(menus, o.race.text_buffers()).map(|_| ()),
        ScreenFn::CarPageTitle => {
            car_page_title::car_page_title(menus, o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::SlideshowTitle => {
            slideshow_title::slideshow_title(o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::PlayerNumberText => {
            player_number_text::player_number_text(menus, o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::DrawControllerPage => {
            controller::draw_controller_page(menus, o.input, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawControlBindings => {
            draw_control_bindings::draw_control_bindings(menus, o.screens, &mut GameCanvas)
                .map(|_| ())
        }
        ScreenFn::PickPlayer1 => page_settings::pick_player(menus, 0).map(|_| ()),
        ScreenFn::PickPlayer2 => page_settings::pick_player(menus, 1).map(|_| ()),
        ScreenFn::PickPlayer3 => page_settings::pick_player(menus, 2).map(|_| ()),
        ScreenFn::PickPlayer4 => page_settings::pick_player(menus, 3).map(|_| ()),
        ScreenFn::LearnAxis => controller::learn_axis(menus, o.input),
        ScreenFn::HandleEditKeys => handle_edit_keys::handle_edit_keys(menus, o.system).map(|_| ()),
        ScreenFn::BestScoresCaption => {
            best_scores::best_scores_caption(menus, o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::DrawBestScoresPage => {
            best_scores::draw_best_scores_page(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::SavegamePageNumbers => {
            savegame_page_numbers::savegame_page_numbers(menus, o.race.text_buffers()).map(|_| ())
        }
        ScreenFn::DrawSavegameNames => {
            draw_savegame_names::draw_savegame_names(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawSavegamePreview => {
            draw_savegame_preview::draw_savegame_preview(menus, o.saves, o.system, &mut GameCanvas)
                .map(|_| ())
        }
        ScreenFn::DrawSaveSetupList => {
            draw_save_setup_list::draw_save_setup_list(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawLoadSetupList => {
            draw_load_setup_list::draw_load_setup_list(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawReplaySlotNames => {
            draw_replay_slot_names::draw_replay_slot_names(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawPlayerDetails => draw_player_details::draw_player_page(
            menus,
            o.input,
            draw_player_details::FULL,
            &mut GameCanvas,
        ),
        ScreenFn::YesNoCaption => yes_no_caption::run(menus, o.race.text_buffers()),
        ScreenFn::DrawLeaveQuestion => draw_leave_question::draw_leave_question(&mut GameCanvas),
        ScreenFn::DrawGraphicsSettings => {
            draw_graphics_settings::draw_graphics_settings(menus, o.gfx, &mut GameCanvas)
        }
        ScreenFn::DrawSoundSettings => {
            draw_sound_settings::draw_sound_settings(menus, o.audio, &mut GameCanvas)
        }
        ScreenFn::ReplayRangeCaption => Ok(best_scores::replay_range_caption_in(
            menus,
            race.text_buffers(),
        )?),
        ScreenFn::LoadSetupValues => Ok(load_setup_values::load_setup_values(menus)?),
        ScreenFn::LoadAudioSettings => {
            page_settings::load_audio_settings(menus, o.audio);
            Ok(())
        }
        ScreenFn::DrawItinerary => {
            itinerary::draw_itinerary(menus, o.system, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawStageInfo => {
            { draw_stage_info::draw_stage_info(menus, o.system, o.gfx, &mut GameCanvas) }
                .map(|_| ())
        }
        ScreenFn::DrawVolumeSettings => {
            draw_volume_settings::draw_volume_settings(o, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawVisualPreferences => {
            draw_visual_preferences::draw_visual_preferences(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawClassRanking => {
            draw_class_ranking::draw_class_ranking(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawCarFigures => {
            draw_car_figures::draw_car_figures(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawRedefinedKeys => {
            draw_redefined_keys::draw_redefined_keys(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::RedefinePlayer1Key => {
            redefine_keys::redefine_key(menus, player::player(menus, 0))
        }
        ScreenFn::RedefineKey => {
            redefine_keys::redefine_key(menus, player_settings::current(menus).ok())
        }
        ScreenFn::DrawOptionsChecklist => {
            draw_options_checklist::draw_options_checklist(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawServiceStageInfo => {
            draw_service_stage_info::draw_service_stage_info(
                menus,
                o.system,
                o.gfx,
                o.race,
                &mut GameCanvas,
            )
        }
        .map(|_| ()),
        ScreenFn::DrawSelectedStageMenu => {
            draw_selected_stage_menu::draw_selected_stage_menu(
                menus,
                o.system,
                o.gfx,
                o.race,
                &mut GameCanvas,
            )
        }
        .map(|_| ()),
        ScreenFn::DrawPlayersResultsPoints => {
            draw_players_results_points::draw(menus, o.system, &mut GameCanvas)
        }
        ScreenFn::DrawPlayersStandings => {
            draw_players_standings::draw_players_standings(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::EmptyDrawStep => Ok(()),
        ScreenFn::DrawPlayerStageResult => {
            draw_player_stage_result::draw_player_stage_result(menus, o.system, &mut GameCanvas)
                .map(|_| ())
        }
        ScreenFn::DrawDuelResult => {
            draw_duel_result::draw_duel_result(menus, o.system, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawTimeTrialStageInfo => {
            draw_time_trial_stage_info::draw_time_trial_stage_info(
                menus,
                o.system,
                o.gfx,
                o.race,
                &mut GameCanvas,
            )
        }
        .map(|_| ()),
        ScreenFn::DrawNote1fd => draw_note_1fd::draw_note_1fd(&mut GameCanvas),
        ScreenFn::DrawArcadeStageTitle => {
            draw_arcade_stage_title::draw_arcade_stage_title(&mut GameCanvas)
        }
        ScreenFn::DrawPlayerRallyResult => {
            draw_player_rally_result::draw_player_rally_result(menus, o.system, &mut GameCanvas)
                .map(|_| ())
        }
        ScreenFn::DrawRallyStageInfo => {
            stage_info::draw_rally_stage_info(menus, o.system, o.gfx, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawRallyStageTimes => {
            draw_rally_stage_times::draw(menus, o.system, &mut GameCanvas)
        }
        ScreenFn::DrawClassTimesMarked => draw_class_times_marked::draw(menus, &mut GameCanvas),
        ScreenFn::NoteLockedStage => locked_notes::note_locked_stage(menus, o.gfx),
        ScreenFn::DrawTimeTrialStageSelect => {
            draw_time_trial_stage_select::draw_time_trial_stage_select(
                menus,
                o.system,
                o.gfx,
                o.race,
                &mut GameCanvas,
            )
        }
        .map(|_| ()),
        ScreenFn::DrawStageTimes => draw_stage_times::draw(menus, o.system, &mut GameCanvas),
        ScreenFn::DrawPlayersResults => {
            draw_players_results::draw(menus, o.system, &mut GameCanvas)
        }
        ScreenFn::DrawStageResults => {
            stage_results::draw_stage_results(o, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawHighlightedObject => draw_highlighted_object::run(o),
        ScreenFn::DrawMainMenuObjects => draw_main_menu_objects::run(o),
        ScreenFn::DrawCarObject => draw_car_object::run(o),
        ScreenFn::ClearChoiceMarks => Ok(clear_choice_marks::clear_choice_marks(&menus.screens)?),
        ScreenFn::DrawServiceBoxes => draw_service_boxes::draw_service_boxes(o),
        ScreenFn::DrawBottomPanel => draw_bottom_panel::draw_bottom_panel(o),
        ScreenFn::DrawTwoColumnPanel => draw_two_column_panel::draw_two_column_panel(o),
        ScreenFn::StageRangeCaption => Ok(stage_range_caption::stage_range_caption(
            &menus.play,
            o.race,
        )?),
        ScreenFn::DrawClassStageResults => class_stage::draw(menus, o.system),
        ScreenFn::DrawOverallStageResults => overall_stage::draw(menus, o.system),
        ScreenFn::DrawClassRallyResults => class_rally::draw(menus, o.system),
        ScreenFn::DrawOverallRallyResults => overall_rally::draw(menus, o.system),
        ScreenFn::DrawRecordTimes => {
            best_scores::draw_record_times(menus, o.system, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawChampionshipTable => {
            championship_table::draw_championship_table(menus, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawChampionshipStandings => {
            draw_championship_standings::draw_championship_standings(
                menus,
                o.system,
                &mut GameCanvas,
            )
            .map(|_| ())
        }
        ScreenFn::DrawServiceArea => {
            draw_service_area::draw_service_area(menus, o.gfx, o.race, &mut GameCanvas).map(|_| ())
        }
        ScreenFn::DrawEventBestScores => draw_event_best_scores::draw(menus, &mut GameCanvas),
        ScreenFn::DrawOverallBestScores => draw_overall_best_scores::draw(menus, &mut GameCanvas),
        ScreenFn::Nothing => Ok(()),
        ScreenFn::NoteLockedItem3 => locked_notes::note_locked_item_3(menus, race, o.gfx),
    }
}
