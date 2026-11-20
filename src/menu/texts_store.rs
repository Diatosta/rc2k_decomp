//! The menus' texts: two log texts of the map's polygon code, each menu screen's picture path
//! (one copy per screen, named for the screen that loads it; the others by their file), the stage maps', car
//! specifications' and slides' paths (which `menu::lists_store`'s tables name by
//! `MenuText::key`), and two tables of speech samples' names (`car_speech`, `stage_speech`: keys
//! of `audio::names_store`'s names, `speech_entry`). Each text's value is in `MenuTexts::new`
//! under its field's name. `Menus::texts` holds them, apart from the menus' state:
//! the only writes are the picture loaders', which may swap a path's extension in place
//! (`files::swap_missing_extension`).

use crate::audio::names_store::{NameList, SampleName};
use crate::memory::session::{PlainBytes, PlainDword, with_runs};

#[cfg_attr(test, repr(C, align(4)))]
#[derive(rc2k_derive::Cells)]
pub struct MenuTexts {
    /// Not in the image: two bytes so that each address keeps its alignment mod 4.
    pub _lead: PlainBytes<2>,
    pub no_point_text: PlainBytes<17>,
    pub no_second_point_text: PlainBytes<40>,
    pub loading_pcx: PlainBytes<37>,
    pub leave_question_picture: PlainBytes<37>,
    pub mob_welc_png: PlainBytes<38>,
    pub a8_welc_png: PlainBytes<37>,
    pub titlebar_png: PlainBytes<38>,
    pub main_menu_picture: PlainBytes<37>,
    pub start_rally_picture: PlainBytes<34>,
    pub network_rally_picture: PlainBytes<34>,
    pub player1_details_picture: PlainBytes<35>,
    pub player2_details_picture: PlainBytes<35>,
    pub player3_details_picture: PlainBytes<35>,
    pub player4_details_picture: PlainBytes<35>,
    pub player_details_picture: PlainBytes<35>,
    pub options_picture: PlainBytes<34>,
    pub information_picture: PlainBytes<34>,
    pub video_png: PlainBytes<35>,
    pub visual_preferences_picture: PlainBytes<36>,
    pub visual_options_picture: PlainBytes<36>,
    pub cspecs_png: PlainBytes<36>,
    pub cspecs_png_2: PlainBytes<36>,
    pub load_championship_picture: PlainBytes<36>,
    pub save_championship_picture: PlainBytes<36>,
    pub save_replay_picture: PlainBytes<36>,
    pub load_replay_picture: PlainBytes<36>,
    pub select_car_picture: PlainBytes<36>,
    pub car_specs_display_picture: PlainBytes<36>,
    pub cspecs_png_3: PlainBytes<36>,
    pub view_best_times_picture: PlainBytes<36>,
    pub view_best_scores_picture: PlainBytes<38>,
    pub best_times_picture: PlainBytes<36>,
    pub arcade_best_times_picture: PlainBytes<36>,
    pub time_trial_best_times_picture: PlainBytes<36>,
    pub arcade_best_scores_picture: PlainBytes<38>,
    pub best_class_scores_picture: PlainBytes<38>,
    pub best_overall_scores_picture: PlainBytes<38>,
    pub audio_setup_picture: PlainBytes<35>,
    pub advanced_audio_picture: PlainBytes<35>,
    pub vcourses_png: PlainBytes<38>,
    pub vcourses_png_2: PlainBytes<38>,
    pub vcourses_png_3: PlainBytes<38>,
    pub vcourses_png_4: PlainBytes<38>,
    pub ukmap_png: PlainBytes<35>,
    pub loading_png: PlainBytes<37>,
    pub player1_controller_picture: PlainBytes<37>,
    pub player1_buttons_picture: PlainBytes<37>,
    pub player2_controller_picture: PlainBytes<37>,
    pub player2_buttons_picture: PlainBytes<37>,
    pub player3_controller_picture: PlainBytes<37>,
    pub player3_buttons_picture: PlainBytes<37>,
    pub player4_controller_picture: PlainBytes<37>,
    pub player4_buttons_picture: PlainBytes<37>,
    pub redefine_keys_player1_picture: PlainBytes<37>,
    pub redefine_keys_player2_picture: PlainBytes<37>,
    pub redefine_keys_player3_picture: PlainBytes<37>,
    pub redefine_keys_player4_picture: PlainBytes<37>,
    pub sral_png: PlainBytes<34>,
    pub sral_png_2: PlainBytes<34>,
    pub championship_new_simulation_picture: PlainBytes<34>,
    pub new_simulation_picture: PlainBytes<34>,
    pub endday_png: PlainBytes<36>,
    pub endday_png_2: PlainBytes<36>,
    pub rally_stages_picture: PlainBytes<35>,
    pub serv00_png: PlainBytes<36>,
    pub serv00_png_2: PlainBytes<36>,
    pub serv00_png_3: PlainBytes<36>,
    pub class_stage_results_picture: PlainBytes<38>,
    pub overall_stage_results_picture: PlainBytes<38>,
    pub class_rally_results_picture: PlainBytes<38>,
    pub overall_rally_results_picture: PlainBytes<38>,
    pub results1_png: PlainBytes<38>,
    pub results1_png_2: PlainBytes<38>,
    /// The copy `menu::section_times_loop` loads as its background.
    pub section_times_png: PlainBytes<38>,
    pub inspection_area_picture: PlainBytes<37>,
    pub championship_service_review_picture: PlainBytes<38>,
    pub championship_standings_review_picture: PlainBytes<38>,
    pub championship_races_review_picture: PlainBytes<38>,
    pub championship_end_review_picture: PlainBytes<38>,
    pub in_game_options_picture: PlainBytes<34>,
    pub welcomer_png: PlainBytes<38>,
    pub welvx_png: PlainBytes<35>,
    pub welpir_png: PlainBytes<36>,
    pub welsct_png: PlainBytes<36>,
    pub weljc_png: PlainBytes<35>,
    pub welul_png: PlainBytes<35>,
    pub welmnx_png: PlainBytes<36>,
    pub a8_1_png: PlainBytes<34>,
    pub a8_2_png: PlainBytes<34>,
    pub a8_3_png: PlainBytes<34>,
    pub a8_4_png: PlainBytes<34>,
    pub a8_5_png: PlainBytes<34>,
    pub a8_6_png: PlainBytes<34>,
    pub setcar_png: PlainBytes<36>,
    pub class_championship_standings_picture: PlainBytes<37>,
    pub championship_standings_picture: PlainBytes<37>,
    pub simple_select_players_picture: PlainBytes<37>,
    pub single_rally_review_picture: PlainBytes<38>,
    pub arcade_review_options_picture: PlainBytes<38>,
    pub time_trial_select_players_picture: PlainBytes<37>,
    pub arcade_next_level_review_picture: PlainBytes<38>,
    pub arcade_retry_review_picture: PlainBytes<38>,
    pub roptions_png: PlainBytes<38>,
    pub arcade_level_end_results_picture: PlainBytes<38>,
    pub stage_result_picture: PlainBytes<38>,
    pub results1_png_4: PlainBytes<38>,
    pub single_rally_two_results_picture: PlainBytes<38>,
    pub single_rally_three_four_results_picture: PlainBytes<38>,
    pub time_trial_results_picture: PlainBytes<38>,
    pub single_rally_overall_results_picture: PlainBytes<38>,
    pub results1_png_5: PlainBytes<38>,
    pub selstage_png: PlainBytes<38>,
    pub selstage_png_2: PlainBytes<38>,
    pub arcstage_png: PlainBytes<38>,
    pub arcade1_png: PlainBytes<37>,
    pub arcade_select_level_picture: PlainBytes<36>,
    pub arcade_results_picture: PlainBytes<38>,
    pub roptions_png_2: PlainBytes<38>,
    pub racing_png: PlainBytes<36>,
    pub racing_png_2: PlainBytes<36>,
    pub mbuts00_pcx: PlainBytes<37>,
    pub serv00_png_4: PlainBytes<36>,
    pub serv01_png: PlainBytes<36>,
    pub serv02_png: PlainBytes<36>,
    pub serv03_png: PlainBytes<36>,
    pub serv04_png: PlainBytes<36>,
    pub serv05_png: PlainBytes<36>,
    pub serv06_png: PlainBytes<36>,
    pub serv07_png: PlainBytes<36>,
    pub serv08_png: PlainBytes<36>,
    pub serv09_png: PlainBytes<36>,
    pub serv10_png: PlainBytes<36>,
    pub serv11_png: PlainBytes<36>,
    pub serv12_png: PlainBytes<36>,
    pub serv13_png: PlainBytes<36>,
    pub serv14_png: PlainBytes<36>,
    pub serv15_png: PlainBytes<36>,
    pub serv16_png: PlainBytes<36>,
    pub serv17_png: PlainBytes<36>,
    pub serv18_png: PlainBytes<36>,
    pub serv19_png: PlainBytes<36>,
    pub serv20_png: PlainBytes<36>,
    pub serv21_png: PlainBytes<36>,
    pub serv22_png: PlainBytes<36>,
    pub serv33_png: PlainBytes<36>,
    pub vrw_001_png: PlainBytes<42>,
    pub vrw_002_png: PlainBytes<42>,
    pub vrw_003_png: PlainBytes<42>,
    pub vrw_004_png: PlainBytes<42>,
    pub vrw_005_png: PlainBytes<42>,
    pub vrw_006_png: PlainBytes<42>,
    pub pir_001_png: PlainBytes<42>,
    pub pir_002_png: PlainBytes<42>,
    pub pir_003_png: PlainBytes<42>,
    pub pir_004_png: PlainBytes<42>,
    pub pir_005_png: PlainBytes<42>,
    pub pir_006_png: PlainBytes<42>,
    pub sc_001_png: PlainBytes<41>,
    pub sc_002_png: PlainBytes<41>,
    pub sc_003_png: PlainBytes<41>,
    pub sc_004_png: PlainBytes<41>,
    pub sc_005_png: PlainBytes<41>,
    pub sc_006_png: PlainBytes<41>,
    pub jc_001_png: PlainBytes<41>,
    pub jc_002_png: PlainBytes<41>,
    pub jc_003_png: PlainBytes<41>,
    pub jc_004_png: PlainBytes<41>,
    pub jc_005_png: PlainBytes<41>,
    pub jc_006_png: PlainBytes<41>,
    pub ul_001_png: PlainBytes<41>,
    pub ul_002_png: PlainBytes<41>,
    pub ul_003_png: PlainBytes<41>,
    pub ul_004_png: PlainBytes<41>,
    pub ul_005_png: PlainBytes<41>,
    pub ul_006_png: PlainBytes<41>,
    pub mnx_001_png: PlainBytes<42>,
    pub mnx_002_png: PlainBytes<42>,
    pub mnx_003_png: PlainBytes<42>,
    pub mnx_004_png: PlainBytes<42>,
    pub mnx_005_png: PlainBytes<42>,
    pub mnx_006_png: PlainBytes<42>,
    pub mgnvc_png: PlainBytes<42>,
    pub vxavc_png: PlainBytes<42>,
    pub stcvc_png: PlainBytes<42>,
    pub hycvc_png: PlainBytes<42>,
    pub sbivc_png: PlainBytes<42>,
    pub pgovc_png: PlainBytes<42>,
    pub mxpvc_png: PlainBytes<42>,
    pub mtlvc_png: PlainBytes<42>,
    pub vwgvc_png: PlainBytes<42>,
    pub nsavc_png: PlainBytes<42>,
    pub ptwvc_png: PlainBytes<42>,
    pub ctsvc_png: PlainBytes<42>,
    pub hncvc_png: PlainBytes<42>,
    pub nsmvc_png: PlainBytes<42>,
    pub pgtvc_png: PlainBytes<42>,
    pub frtvc_png: PlainBytes<42>,
    pub sdovc_png: PlainBytes<42>,
    pub stivc_png: PlainBytes<42>,
    pub sdfvc_png: PlainBytes<42>,
    pub ptcvc_png: PlainBytes<42>,
    pub fmxvc_png: PlainBytes<42>,
    pub fpmvc_png: PlainBytes<42>,
    pub oscvc_png: PlainBytes<42>,
    pub rclvc_png: PlainBytes<42>,
    pub mgndm_png: PlainBytes<42>,
    pub vxadm_png: PlainBytes<42>,
    pub stcdm_png: PlainBytes<42>,
    pub hycdm_png: PlainBytes<42>,
    pub sbidm_png: PlainBytes<42>,
    pub pgodm_png: PlainBytes<42>,
    pub mxpdm_png: PlainBytes<42>,
    pub mtldm_png: PlainBytes<42>,
    pub vwgdm_png: PlainBytes<42>,
    pub nsadm_png: PlainBytes<42>,
    pub ptwdm_png: PlainBytes<42>,
    pub ctsdm_png: PlainBytes<42>,
    pub hncdm_png: PlainBytes<42>,
    pub nsmdm_png: PlainBytes<42>,
    pub pgtdm_png: PlainBytes<42>,
    pub frtdm_png: PlainBytes<42>,
    pub sdodm_png: PlainBytes<42>,
    pub stidm_png: PlainBytes<42>,
    pub sdfdm_png: PlainBytes<42>,
    pub ptcdm_png: PlainBytes<42>,
    pub fmxdm_png: PlainBytes<42>,
    pub fpmdm_png: PlainBytes<42>,
    pub oscdm_png: PlainBytes<42>,
    pub rcldm_png: PlainBytes<42>,
    pub mgnen_png: PlainBytes<42>,
    pub vxaen_png: PlainBytes<42>,
    pub stcen_png: PlainBytes<42>,
    pub hycen_png: PlainBytes<42>,
    pub sbien_png: PlainBytes<42>,
    pub pgoen_png: PlainBytes<42>,
    pub mxpen_png: PlainBytes<42>,
    pub mtlen_png: PlainBytes<42>,
    pub vwgen_png: PlainBytes<42>,
    pub nsaen_png: PlainBytes<42>,
    pub ptwen_png: PlainBytes<42>,
    pub ctsen_png: PlainBytes<42>,
    pub hncen_png: PlainBytes<42>,
    pub nsmen_png: PlainBytes<42>,
    pub pgten_png: PlainBytes<42>,
    pub frten_png: PlainBytes<42>,
    pub sdoen_png: PlainBytes<42>,
    pub stien_png: PlainBytes<42>,
    pub sdfen_png: PlainBytes<42>,
    pub ptcen_png: PlainBytes<42>,
    pub fmxen_png: PlainBytes<42>,
    pub fpmen_png: PlainBytes<42>,
    pub oscen_png: PlainBytes<42>,
    pub rclen_png: PlainBytes<42>,
    pub mgntc_png: PlainBytes<42>,
    pub vxatc_png: PlainBytes<42>,
    pub stctc_png: PlainBytes<42>,
    pub hyctc_png: PlainBytes<42>,
    pub sbitc_png: PlainBytes<42>,
    pub pgotc_png: PlainBytes<42>,
    pub mxptc_png: PlainBytes<42>,
    pub mtltc_png: PlainBytes<42>,
    pub vwgtc_png: PlainBytes<42>,
    pub nsatc_png: PlainBytes<42>,
    pub ptwtc_png: PlainBytes<42>,
    pub ctstc_png: PlainBytes<42>,
    pub hnctc_png: PlainBytes<42>,
    pub nsmtc_png: PlainBytes<42>,
    pub pgttc_png: PlainBytes<42>,
    pub frttc_png: PlainBytes<42>,
    pub sdotc_png: PlainBytes<42>,
    pub stitc_png: PlainBytes<42>,
    pub sdftc_png: PlainBytes<42>,
    pub ptctc_png: PlainBytes<42>,
    pub fmxtc_png: PlainBytes<42>,
    pub fpmtc_png: PlainBytes<42>,
    pub osctc_png: PlainBytes<42>,
    pub rcltc_png: PlainBytes<42>,
    pub mgnsl00_png: PlainBytes<29>,
    pub vxasl00_png: PlainBytes<29>,
    pub stcsl00_png: PlainBytes<29>,
    pub hycsl00_png: PlainBytes<29>,
    pub sbisl00_png: PlainBytes<29>,
    pub pgosl00_png: PlainBytes<29>,
    pub mxpsl00_png: PlainBytes<29>,
    pub mtlsl00_png: PlainBytes<29>,
    pub vwgsl00_png: PlainBytes<29>,
    pub nsasl00_png: PlainBytes<29>,
    pub ptwsl00_png: PlainBytes<29>,
    pub ctssl00_png: PlainBytes<29>,
    pub hncsl00_png: PlainBytes<29>,
    pub nsmsl00_png: PlainBytes<29>,
    pub pgtsl00_png: PlainBytes<29>,
    pub frtsl00_png: PlainBytes<29>,
    pub sdosl00_png: PlainBytes<29>,
    pub stisl00_png: PlainBytes<29>,
    pub sdfsl00_png: PlainBytes<29>,
    pub ptcsl00_png: PlainBytes<29>,
    pub fmxsl00_png: PlainBytes<29>,
    pub fpmsl00_png: PlainBytes<29>,
    pub oscsl00_png: PlainBytes<29>,
    pub rclsl00_png: PlainBytes<29>,
    /// Not in the image: two bytes that put the speech tables on a dword boundary.
    pub _speech_pad: PlainBytes<2>,
    /// Per car, its two speech samples' names and a 0.
    pub car_speech: [[PlainDword; 3]; 24],
    /// Per stage, three speech samples' names and a 0.
    pub stage_speech: [[PlainDword; 4]; 36],
}

impl MenuTexts {
    pub const fn new() -> Self {
        MenuTexts {
            _lead: PlainBytes::new([0; 2]),
            no_point_text: PlainBytes::new(with_runs::<17>(&[(0, b"No point in poly")])),
            no_second_point_text: PlainBytes::new(with_runs::<40>(&[(
                0,
                b"While scanning right didnt find point 2",
            )])),
            loading_pcx: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\loading.pcx",
            )])),
            leave_question_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\mainscr.png",
            )])),
            mob_welc_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\mob_welc.png",
            )])),
            a8_welc_png: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_welc.png",
            )])),
            titlebar_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\titlebar.png",
            )])),
            main_menu_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\mainscr.png",
            )])),
            start_rally_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\sral.png",
            )])),
            network_rally_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\nral.png",
            )])),
            player1_details_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\pdet1.png",
            )])),
            player2_details_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\pdet1.png",
            )])),
            player3_details_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\pdet1.png",
            )])),
            player4_details_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\pdet1.png",
            )])),
            player_details_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\pdetc.png",
            )])),
            options_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\opts.png",
            )])),
            information_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\info.png",
            )])),
            video_png: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\video.png",
            )])),
            visual_preferences_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\visual.png",
            )])),
            visual_options_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\visual.png",
            )])),
            cspecs_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs.png",
            )])),
            cspecs_png_2: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs.png",
            )])),
            load_championship_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\loadsv.png",
            )])),
            save_championship_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\loadsv.png",
            )])),
            save_replay_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\loadsv.png",
            )])),
            load_replay_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\loadsv.png",
            )])),
            select_car_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs.png",
            )])),
            car_specs_display_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs.png",
            )])),
            cspecs_png_3: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs.png",
            )])),
            view_best_times_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\btimes.png",
            )])),
            view_best_scores_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\bascores.png",
            )])),
            best_times_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\btimes.png",
            )])),
            arcade_best_times_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\btimes.png",
            )])),
            time_trial_best_times_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\btimes.png",
            )])),
            arcade_best_scores_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\bascores.png",
            )])),
            best_class_scores_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\bascores.png",
            )])),
            best_overall_scores_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\bascores.png",
            )])),
            audio_setup_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\audio.png",
            )])),
            advanced_audio_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\audio.png",
            )])),
            vcourses_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\vcourses.png",
            )])),
            vcourses_png_2: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\vcourses.png",
            )])),
            vcourses_png_3: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\vcourses.png",
            )])),
            vcourses_png_4: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\vcourses.png",
            )])),
            ukmap_png: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\ukmap.png",
            )])),
            loading_png: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\loading.png",
            )])),
            player1_controller_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player1_buttons_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player2_controller_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player2_buttons_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player3_controller_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player3_buttons_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player4_controller_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            player4_buttons_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\control.png",
            )])),
            redefine_keys_player1_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\defkeys.png",
            )])),
            redefine_keys_player2_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\defkeys.png",
            )])),
            redefine_keys_player3_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\defkeys.png",
            )])),
            redefine_keys_player4_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\defkeys.png",
            )])),
            sral_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\sral.png",
            )])),
            sral_png_2: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\sral.png",
            )])),
            championship_new_simulation_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\sral.png",
            )])),
            new_simulation_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\sral.png",
            )])),
            endday_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\endday.png",
            )])),
            endday_png_2: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\endday.png",
            )])),
            rally_stages_picture: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\ukmap.png",
            )])),
            serv00_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv00.png",
            )])),
            serv00_png_2: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv00.png",
            )])),
            serv00_png_3: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv00.png",
            )])),
            class_stage_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            overall_stage_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            class_rally_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            overall_rally_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            results1_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            results1_png_2: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            section_times_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            inspection_area_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\service.png",
            )])),
            championship_service_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            championship_standings_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            championship_races_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            championship_end_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            in_game_options_picture: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\opts.png",
            )])),
            welcomer_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\welcomer.png",
            )])),
            welvx_png: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\welvx.png",
            )])),
            welpir_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\welpir.png",
            )])),
            welsct_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\welsct.png",
            )])),
            weljc_png: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\weljc.png",
            )])),
            welul_png: PlainBytes::new(with_runs::<35>(&[(
                0,
                b"files\\g\\graphics\\screens\\welul.png",
            )])),
            welmnx_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\welmnx.png",
            )])),
            a8_1_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_1.png",
            )])),
            a8_2_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_2.png",
            )])),
            a8_3_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_3.png",
            )])),
            a8_4_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_4.png",
            )])),
            a8_5_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_5.png",
            )])),
            a8_6_png: PlainBytes::new(with_runs::<34>(&[(
                0,
                b"files\\g\\graphics\\screens\\a8_6.png",
            )])),
            setcar_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\setcar.png",
            )])),
            class_championship_standings_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\champoc.png",
            )])),
            championship_standings_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\champoc.png",
            )])),
            simple_select_players_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\arcade1.png",
            )])),
            single_rally_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            arcade_review_options_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            time_trial_select_players_picture: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\arcade1.png",
            )])),
            arcade_next_level_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            arcade_retry_review_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            roptions_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            arcade_level_end_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            stage_result_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            results1_png_4: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            single_rally_two_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            single_rally_three_four_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            time_trial_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            single_rally_overall_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            results1_png_5: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            selstage_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\selstage.png",
            )])),
            selstage_png_2: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\selstage.png",
            )])),
            arcstage_png: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\arcstage.png",
            )])),
            arcade1_png: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\screens\\arcade1.png",
            )])),
            arcade_select_level_picture: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\Selarc.png",
            )])),
            arcade_results_picture: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\results1.png",
            )])),
            roptions_png_2: PlainBytes::new(with_runs::<38>(&[(
                0,
                b"files\\g\\graphics\\screens\\roptions.png",
            )])),
            racing_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\racing.png",
            )])),
            racing_png_2: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\racing.png",
            )])),
            mbuts00_pcx: PlainBytes::new(with_runs::<37>(&[(
                0,
                b"files\\g\\graphics\\sprites\\mbuts00.pcx",
            )])),
            serv00_png_4: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv00.png",
            )])),
            serv01_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv01.png",
            )])),
            serv02_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv02.png",
            )])),
            serv03_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv03.png",
            )])),
            serv04_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv04.png",
            )])),
            serv05_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv05.png",
            )])),
            serv06_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv06.png",
            )])),
            serv07_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv07.png",
            )])),
            serv08_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv08.png",
            )])),
            serv09_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv09.png",
            )])),
            serv10_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv10.png",
            )])),
            serv11_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv11.png",
            )])),
            serv12_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv12.png",
            )])),
            serv13_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv13.png",
            )])),
            serv14_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv14.png",
            )])),
            serv15_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv15.png",
            )])),
            serv16_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv16.png",
            )])),
            serv17_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv17.png",
            )])),
            serv18_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv18.png",
            )])),
            serv19_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv19.png",
            )])),
            serv20_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv20.png",
            )])),
            serv21_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv21.png",
            )])),
            serv22_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv22.png",
            )])),
            serv33_png: PlainBytes::new(with_runs::<36>(&[(
                0,
                b"files\\g\\graphics\\screens\\serv33.png",
            )])),
            vrw_001_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_001.png",
            )])),
            vrw_002_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_002.png",
            )])),
            vrw_003_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_003.png",
            )])),
            vrw_004_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_004.png",
            )])),
            vrw_005_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_005.png",
            )])),
            vrw_006_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\vrw_006.png",
            )])),
            pir_001_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_001.png",
            )])),
            pir_002_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_002.png",
            )])),
            pir_003_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_003.png",
            )])),
            pir_004_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_004.png",
            )])),
            pir_005_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_005.png",
            )])),
            pir_006_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\pir_006.png",
            )])),
            sc_001_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_001.png",
            )])),
            sc_002_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_002.png",
            )])),
            sc_003_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_003.png",
            )])),
            sc_004_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_004.png",
            )])),
            sc_005_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_005.png",
            )])),
            sc_006_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\sc_006.png",
            )])),
            jc_001_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_001.png",
            )])),
            jc_002_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_002.png",
            )])),
            jc_003_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_003.png",
            )])),
            jc_004_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_004.png",
            )])),
            jc_005_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_005.png",
            )])),
            jc_006_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\jc_006.png",
            )])),
            ul_001_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_001.png",
            )])),
            ul_002_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_002.png",
            )])),
            ul_003_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_003.png",
            )])),
            ul_004_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_004.png",
            )])),
            ul_005_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_005.png",
            )])),
            ul_006_png: PlainBytes::new(with_runs::<41>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\ul_006.png",
            )])),
            mnx_001_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_001.png",
            )])),
            mnx_002_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_002.png",
            )])),
            mnx_003_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_003.png",
            )])),
            mnx_004_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_004.png",
            )])),
            mnx_005_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_005.png",
            )])),
            mnx_006_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\maps\\mnx_006.png",
            )])),
            mgnvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mgnvc.png",
            )])),
            vxavc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vxavc.png",
            )])),
            stcvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stcvc.png",
            )])),
            hycvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hycvc.png",
            )])),
            sbivc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sbivc.png",
            )])),
            pgovc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgovc.png",
            )])),
            mxpvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mxpvc.png",
            )])),
            mtlvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mtlvc.png",
            )])),
            vwgvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vwgvc.png",
            )])),
            nsavc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsavc.png",
            )])),
            ptwvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptwvc.png",
            )])),
            ctsvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ctsvc.png",
            )])),
            hncvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hncvc.png",
            )])),
            nsmvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsmvc.png",
            )])),
            pgtvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgtvc.png",
            )])),
            frtvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\frtvc.png",
            )])),
            sdovc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdovc.png",
            )])),
            stivc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stivc.png",
            )])),
            sdfvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdfvc.png",
            )])),
            ptcvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptcvc.png",
            )])),
            fmxvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fmxvc.png",
            )])),
            fpmvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fpmvc.png",
            )])),
            oscvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\oscvc.png",
            )])),
            rclvc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\rclvc.png",
            )])),
            mgndm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mgndm.png",
            )])),
            vxadm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vxadm.png",
            )])),
            stcdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stcdm.png",
            )])),
            hycdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hycdm.png",
            )])),
            sbidm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sbidm.png",
            )])),
            pgodm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgodm.png",
            )])),
            mxpdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mxpdm.png",
            )])),
            mtldm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mtldm.png",
            )])),
            vwgdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vwgdm.png",
            )])),
            nsadm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsadm.png",
            )])),
            ptwdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptwdm.png",
            )])),
            ctsdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ctsdm.png",
            )])),
            hncdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hncdm.png",
            )])),
            nsmdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsmdm.png",
            )])),
            pgtdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgtdm.png",
            )])),
            frtdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\frtdm.png",
            )])),
            sdodm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdodm.png",
            )])),
            stidm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stidm.png",
            )])),
            sdfdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdfdm.png",
            )])),
            ptcdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptcdm.png",
            )])),
            fmxdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fmxdm.png",
            )])),
            fpmdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fpmdm.png",
            )])),
            oscdm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\oscdm.png",
            )])),
            rcldm_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\rcldm.png",
            )])),
            mgnen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mgnen.png",
            )])),
            vxaen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vxaen.png",
            )])),
            stcen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stcen.png",
            )])),
            hycen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hycen.png",
            )])),
            sbien_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sbien.png",
            )])),
            pgoen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgoen.png",
            )])),
            mxpen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mxpen.png",
            )])),
            mtlen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mtlen.png",
            )])),
            vwgen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vwgen.png",
            )])),
            nsaen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsaen.png",
            )])),
            ptwen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptwen.png",
            )])),
            ctsen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ctsen.png",
            )])),
            hncen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hncen.png",
            )])),
            nsmen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsmen.png",
            )])),
            pgten_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgten.png",
            )])),
            frten_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\frten.png",
            )])),
            sdoen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdoen.png",
            )])),
            stien_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stien.png",
            )])),
            sdfen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdfen.png",
            )])),
            ptcen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptcen.png",
            )])),
            fmxen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fmxen.png",
            )])),
            fpmen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fpmen.png",
            )])),
            oscen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\oscen.png",
            )])),
            rclen_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\rclen.png",
            )])),
            mgntc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mgntc.png",
            )])),
            vxatc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vxatc.png",
            )])),
            stctc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stctc.png",
            )])),
            hyctc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hyctc.png",
            )])),
            sbitc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sbitc.png",
            )])),
            pgotc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgotc.png",
            )])),
            mxptc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mxptc.png",
            )])),
            mtltc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\mtltc.png",
            )])),
            vwgtc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\vwgtc.png",
            )])),
            nsatc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsatc.png",
            )])),
            ptwtc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptwtc.png",
            )])),
            ctstc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ctstc.png",
            )])),
            hnctc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\hnctc.png",
            )])),
            nsmtc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\nsmtc.png",
            )])),
            pgttc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\pgttc.png",
            )])),
            frttc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\frttc.png",
            )])),
            sdotc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdotc.png",
            )])),
            stitc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\stitc.png",
            )])),
            sdftc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\sdftc.png",
            )])),
            ptctc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\ptctc.png",
            )])),
            fmxtc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fmxtc.png",
            )])),
            fpmtc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\fpmtc.png",
            )])),
            osctc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\osctc.png",
            )])),
            rcltc_png: PlainBytes::new(with_runs::<42>(&[(
                0,
                b"files\\g\\graphics\\screens\\cspecs\\rcltc.png",
            )])),
            mgnsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\mgnsl00.png",
            )])),
            vxasl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\vxasl00.png",
            )])),
            stcsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\stcsl00.png",
            )])),
            hycsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\hycsl00.png",
            )])),
            sbisl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\sbisl00.png",
            )])),
            pgosl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\pgosl00.png",
            )])),
            mxpsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\mxpsl00.png",
            )])),
            mtlsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\mtlsl00.png",
            )])),
            vwgsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\vwgsl00.png",
            )])),
            nsasl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\nsasl00.png",
            )])),
            ptwsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\ptwsl00.png",
            )])),
            ctssl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\ctssl00.png",
            )])),
            hncsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\hncsl00.png",
            )])),
            nsmsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\nsmsl00.png",
            )])),
            pgtsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\pgtsl00.png",
            )])),
            frtsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\frtsl00.png",
            )])),
            sdosl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\sdosl00.png",
            )])),
            stisl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\stisl00.png",
            )])),
            sdfsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\sdfsl00.png",
            )])),
            ptcsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\ptcsl00.png",
            )])),
            fmxsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\fmxsl00.png",
            )])),
            fpmsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\fpmsl00.png",
            )])),
            oscsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\oscsl00.png",
            )])),
            rclsl00_png: PlainBytes::new(with_runs::<29>(&[(
                0,
                b";3files\\g\\slides\\rclsl00.png",
            )])),
            _speech_pad: PlainBytes::new([0; 2]),
            car_speech: [
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 84)),
                    PlainDword::new(speech(NameList::MenuSpeech, 85)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 86)),
                    PlainDword::new(speech(NameList::MenuSpeech, 87)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 88)),
                    PlainDword::new(speech(NameList::MenuSpeech, 89)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 90)),
                    PlainDword::new(speech(NameList::MenuSpeech, 91)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 92)),
                    PlainDword::new(speech(NameList::MenuSpeech, 93)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 94)),
                    PlainDword::new(speech(NameList::MenuSpeech, 95)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 104)),
                    PlainDword::new(speech(NameList::MenuSpeech, 105)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 96)),
                    PlainDword::new(speech(NameList::MenuSpeech, 97)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 98)),
                    PlainDword::new(speech(NameList::MenuSpeech, 99)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 100)),
                    PlainDword::new(speech(NameList::MenuSpeech, 101)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 102)),
                    PlainDword::new(speech(NameList::MenuSpeech, 103)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 104)),
                    PlainDword::new(speech(NameList::MenuSpeech, 105)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 106)),
                    PlainDword::new(speech(NameList::MenuSpeech, 107)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 108)),
                    PlainDword::new(speech(NameList::MenuSpeech, 109)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 110)),
                    PlainDword::new(speech(NameList::MenuSpeech, 111)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 112)),
                    PlainDword::new(speech(NameList::MenuSpeech, 113)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 114)),
                    PlainDword::new(speech(NameList::MenuSpeech, 115)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 116)),
                    PlainDword::new(speech(NameList::MenuSpeech, 117)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 118)),
                    PlainDword::new(speech(NameList::MenuSpeech, 119)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 120)),
                    PlainDword::new(speech(NameList::MenuSpeech, 121)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 122)),
                    PlainDword::new(speech(NameList::MenuSpeech, 123)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 124)),
                    PlainDword::new(speech(NameList::MenuSpeech, 125)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 126)),
                    PlainDword::new(speech(NameList::MenuSpeech, 127)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 128)),
                    PlainDword::new(speech(NameList::MenuSpeech, 129)),
                    PlainDword::new(SPEECH_END),
                ],
            ],
            stage_speech: [
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 48)),
                    PlainDword::new(speech(NameList::MenuSpeech, 12)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 49)),
                    PlainDword::new(speech(NameList::MenuSpeech, 13)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 50)),
                    PlainDword::new(speech(NameList::MenuSpeech, 14)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 51)),
                    PlainDword::new(speech(NameList::MenuSpeech, 15)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 52)),
                    PlainDword::new(speech(NameList::MenuSpeech, 16)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 53)),
                    PlainDword::new(speech(NameList::MenuSpeech, 17)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 54)),
                    PlainDword::new(speech(NameList::MenuSpeech, 18)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 55)),
                    PlainDword::new(speech(NameList::MenuSpeech, 19)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 56)),
                    PlainDword::new(speech(NameList::MenuSpeech, 20)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 57)),
                    PlainDword::new(speech(NameList::MenuSpeech, 21)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 58)),
                    PlainDword::new(speech(NameList::MenuSpeech, 22)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 59)),
                    PlainDword::new(speech(NameList::MenuSpeech, 23)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 60)),
                    PlainDword::new(speech(NameList::MenuSpeech, 24)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 61)),
                    PlainDword::new(speech(NameList::MenuSpeech, 25)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 62)),
                    PlainDword::new(speech(NameList::MenuSpeech, 26)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 63)),
                    PlainDword::new(speech(NameList::MenuSpeech, 27)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 64)),
                    PlainDword::new(speech(NameList::MenuSpeech, 28)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 65)),
                    PlainDword::new(speech(NameList::MenuSpeech, 29)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 66)),
                    PlainDword::new(speech(NameList::MenuSpeech, 30)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 67)),
                    PlainDword::new(speech(NameList::MenuSpeech, 31)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 68)),
                    PlainDword::new(speech(NameList::MenuSpeech, 32)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 69)),
                    PlainDword::new(speech(NameList::MenuSpeech, 33)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 70)),
                    PlainDword::new(speech(NameList::MenuSpeech, 34)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 71)),
                    PlainDword::new(speech(NameList::MenuSpeech, 35)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 72)),
                    PlainDword::new(speech(NameList::MenuSpeech, 36)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 73)),
                    PlainDword::new(speech(NameList::MenuSpeech, 37)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 74)),
                    PlainDword::new(speech(NameList::MenuSpeech, 38)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 75)),
                    PlainDword::new(speech(NameList::MenuSpeech, 39)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 76)),
                    PlainDword::new(speech(NameList::MenuSpeech, 40)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 77)),
                    PlainDword::new(speech(NameList::MenuSpeech, 41)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 0)),
                    PlainDword::new(speech(NameList::MenuSpeech, 78)),
                    PlainDword::new(speech(NameList::MenuSpeech, 42)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 1)),
                    PlainDword::new(speech(NameList::MenuSpeech, 79)),
                    PlainDword::new(speech(NameList::MenuSpeech, 43)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 2)),
                    PlainDword::new(speech(NameList::MenuSpeech, 80)),
                    PlainDword::new(speech(NameList::MenuSpeech, 44)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 3)),
                    PlainDword::new(speech(NameList::MenuSpeech, 81)),
                    PlainDword::new(speech(NameList::MenuSpeech, 45)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 4)),
                    PlainDword::new(speech(NameList::MenuSpeech, 82)),
                    PlainDword::new(speech(NameList::MenuSpeech, 46)),
                    PlainDword::new(SPEECH_END),
                ],
                [
                    PlainDword::new(speech(NameList::MenuSpeech, 5)),
                    PlainDword::new(speech(NameList::MenuSpeech, 83)),
                    PlainDword::new(speech(NameList::MenuSpeech, 47)),
                    PlainDword::new(SPEECH_END),
                ],
            ],
        }
    }
}

/// Declares `MenuText` over the text fields the menus' tables name, and how each is found.
macro_rules! menu_texts {
    ($($field:ident,)*) => {
        /// A text of `MenuTexts` the menus' tables name, by its field. A table cell holds
        /// `key()`, the text's number + 1 (0 names none).
        #[allow(
            non_camel_case_types,
            reason = "each variant is its field's own name"
        )]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum MenuText {
            $($field,)*
        }

        impl MenuText {
            /// Every text, in the order of the keys.
            const ALL: &[MenuText] = &[$(MenuText::$field,)*];

            /// The text's bytes: its field.
            fn place(self, texts: &'static MenuTexts) -> crate::memory::place::BytePlace {
                match self {
                    $(MenuText::$field => crate::memory::place::BytePlace::from(&texts.$field),)*
                }
            }
        }
    };
}

menu_texts! {
    welvx_png,
    welpir_png,
    welsct_png,
    weljc_png,
    welul_png,
    welmnx_png,
    a8_1_png,
    a8_2_png,
    a8_3_png,
    a8_4_png,
    a8_5_png,
    a8_6_png,
    serv00_png_4,
    serv01_png,
    serv02_png,
    serv03_png,
    serv04_png,
    serv05_png,
    serv06_png,
    serv07_png,
    serv08_png,
    serv09_png,
    serv10_png,
    serv11_png,
    serv12_png,
    serv13_png,
    serv14_png,
    serv15_png,
    serv16_png,
    serv17_png,
    serv18_png,
    serv19_png,
    serv20_png,
    serv21_png,
    serv22_png,
    serv33_png,
    vrw_001_png,
    vrw_002_png,
    vrw_003_png,
    vrw_004_png,
    vrw_005_png,
    vrw_006_png,
    pir_001_png,
    pir_002_png,
    pir_003_png,
    pir_004_png,
    pir_005_png,
    pir_006_png,
    sc_001_png,
    sc_002_png,
    sc_003_png,
    sc_004_png,
    sc_005_png,
    sc_006_png,
    jc_001_png,
    jc_002_png,
    jc_003_png,
    jc_004_png,
    jc_005_png,
    jc_006_png,
    ul_001_png,
    ul_002_png,
    ul_003_png,
    ul_004_png,
    ul_005_png,
    ul_006_png,
    mnx_001_png,
    mnx_002_png,
    mnx_003_png,
    mnx_004_png,
    mnx_005_png,
    mnx_006_png,
    mgnvc_png,
    vxavc_png,
    stcvc_png,
    hycvc_png,
    sbivc_png,
    pgovc_png,
    mxpvc_png,
    mtlvc_png,
    vwgvc_png,
    nsavc_png,
    ptwvc_png,
    ctsvc_png,
    hncvc_png,
    nsmvc_png,
    pgtvc_png,
    frtvc_png,
    sdovc_png,
    stivc_png,
    sdfvc_png,
    ptcvc_png,
    fmxvc_png,
    fpmvc_png,
    oscvc_png,
    rclvc_png,
    mgndm_png,
    vxadm_png,
    stcdm_png,
    hycdm_png,
    sbidm_png,
    pgodm_png,
    mxpdm_png,
    mtldm_png,
    vwgdm_png,
    nsadm_png,
    ptwdm_png,
    ctsdm_png,
    hncdm_png,
    nsmdm_png,
    pgtdm_png,
    frtdm_png,
    sdodm_png,
    stidm_png,
    sdfdm_png,
    ptcdm_png,
    fmxdm_png,
    fpmdm_png,
    oscdm_png,
    rcldm_png,
    mgnen_png,
    vxaen_png,
    stcen_png,
    hycen_png,
    sbien_png,
    pgoen_png,
    mxpen_png,
    mtlen_png,
    vwgen_png,
    nsaen_png,
    ptwen_png,
    ctsen_png,
    hncen_png,
    nsmen_png,
    pgten_png,
    frten_png,
    sdoen_png,
    stien_png,
    sdfen_png,
    ptcen_png,
    fmxen_png,
    fpmen_png,
    oscen_png,
    rclen_png,
    mgntc_png,
    vxatc_png,
    stctc_png,
    hyctc_png,
    sbitc_png,
    pgotc_png,
    mxptc_png,
    mtltc_png,
    vwgtc_png,
    nsatc_png,
    ptwtc_png,
    ctstc_png,
    hnctc_png,
    nsmtc_png,
    pgttc_png,
    frttc_png,
    sdotc_png,
    stitc_png,
    sdftc_png,
    ptctc_png,
    fmxtc_png,
    fpmtc_png,
    osctc_png,
    rcltc_png,
    mgnsl00_png,
    vxasl00_png,
    stcsl00_png,
    hycsl00_png,
    sbisl00_png,
    pgosl00_png,
    mxpsl00_png,
    mtlsl00_png,
    vwgsl00_png,
    nsasl00_png,
    ptwsl00_png,
    ctssl00_png,
    hncsl00_png,
    nsmsl00_png,
    pgtsl00_png,
    frtsl00_png,
    sdosl00_png,
    stisl00_png,
    sdfsl00_png,
    ptcsl00_png,
    fmxsl00_png,
    fpmsl00_png,
    oscsl00_png,
    rclsl00_png,
}

impl MenuText {
    /// The key a table cell holds for this text.
    pub const fn key(self) -> u32 {
        self as u32 + 1
    }
}

/// A speech table's entry: the key of the `index`-th name of `list`.
const fn speech(list: NameList, index: u16) -> u32 {
    SampleName { list, index }.key()
}

/// A speech row's end mark.
const SPEECH_END: u32 = 0xFFFF_FFFF;

/// A row of the speech tables: a car's (three entries) or a stage's (four).
#[derive(Clone, Copy, Debug)]
pub(super) enum SpeechRow {
    Car(usize),
    Stage(usize),
}

const CAR_ROWS: usize = 24;
const CAR_ENTRIES: usize = CAR_ROWS * 3;
const STAGE_ROWS: usize = 36;
const STAGE_ENTRIES: usize = STAGE_ROWS * 4;
/// The first speech row's key; the texts' keys are below it.
const SPEECH_KEY_BASE: u32 = 0x1000;
const _: () = assert!(MenuText::ALL.len() < SPEECH_KEY_BASE as usize);

impl SpeechRow {
    /// The key a table cell holds for this row: `SPEECH_KEY_BASE` + the number of its first
    /// entry among all the entries (the car rows, then the stage rows that follow them).
    pub const fn key(self) -> u32 {
        let first = match self {
            SpeechRow::Car(n) => {
                assert!(n < CAR_ROWS);
                3 * n
            }
            SpeechRow::Stage(n) => {
                assert!(n < STAGE_ROWS);
                CAR_ENTRIES + 4 * n
            }
        };
        SPEECH_KEY_BASE + first as u32
    }
}

/// Entry `n` of the speech tables read as one run (the car rows, then the stage rows), as
/// a row's reading on past its end goes on into the next row.
fn speech_dword(texts: &MenuTexts, n: usize) -> Option<u32> {
    if n < CAR_ENTRIES {
        Some(texts.car_speech[n / 3][n % 3].get())
    } else if n < CAR_ENTRIES + STAGE_ENTRIES {
        let n = n - CAR_ENTRIES;
        Some(texts.stage_speech[n / 4][n % 4].get())
    } else {
        None
    }
}

/// The entry at `place` of the speech row `row` (its key): the sample name's key
/// (`SampleName::key`), or `SPEECH_END`.
pub fn speech_entry(
    texts: &MenuTexts,
    names: &crate::audio::names_store::SampleNames,
    row: u32,
    place: u32,
) -> Result<u32, crate::errors::BadInput> {
    let first = row
        .checked_sub(SPEECH_KEY_BASE)
        .ok_or(crate::errors::BadInput(
            "a speech row's key off the speech rows",
        ))?;
    let entry = speech_dword(texts, first as usize + place as usize).ok_or(
        crate::errors::BadInput("a speech entry off the speech rows"),
    )?;
    // 0: none.
    if entry == SPEECH_END || entry == 0 {
        return Ok(entry);
    }
    crate::audio::store::name_in(names, entry)?;
    Ok(entry)
}

/// Where the text of `key` is (`MenuText::key`).
pub fn text_at(
    texts: &'static MenuTexts,
    key: u32,
) -> Result<crate::memory::place::BytePlace, crate::errors::BadInput> {
    let text = key
        .checked_sub(1)
        .and_then(|n| MenuText::ALL.get(n as usize))
        .ok_or(crate::errors::BadInput("a text's key off the menus' texts"))?;
    Ok(text.place(texts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speech_tables_sit_on_dword_boundaries() {
        assert_eq!(core::mem::offset_of!(MenuTexts, car_speech) % 4, 0);
        assert_eq!(size_of::<MenuTexts>() % 4, 0);
    }

    /// The speech tables' entries as the 1999 image addressed them (the first name of the
    /// speech names is at 0x506E6C), car rows then stage rows.
    const SPEECH_ADDRESSES: [u32; 216] = [
        0x50a15d, 0x50a17e, 0xffffffff, 0x50a19f, 0x50a1bf, 0xffffffff, 0x50a1df, 0x50a201,
        0xffffffff, 0x50a223, 0x50a243, 0xffffffff, 0x50a263, 0x50a285, 0xffffffff, 0x50a2a7,
        0x50a2c5, 0xffffffff, 0x50a3e3, 0x50a402, 0xffffffff, 0x50a2e3, 0x50a304, 0xffffffff,
        0x50a325, 0x50a344, 0xffffffff, 0x50a363, 0x50a384, 0xffffffff, 0x50a3a5, 0x50a3c4,
        0xffffffff, 0x50a3e3, 0x50a402, 0xffffffff, 0x50a421, 0x50a441, 0xffffffff, 0x50a461,
        0x50a481, 0xffffffff, 0x50a4a1, 0x50a4bf, 0xffffffff, 0x50a4dd, 0x50a4fe, 0xffffffff,
        0x50a51f, 0x50a541, 0xffffffff, 0x50a563, 0x50a583, 0xffffffff, 0x50a5a3, 0x50a5c5,
        0xffffffff, 0x50a5e7, 0x50a609, 0xffffffff, 0x50a62b, 0x50a64c, 0xffffffff, 0x50a66d,
        0x50a68c, 0xffffffff, 0x50a6ab, 0x50a6ca, 0xffffffff, 0x50a6e9, 0x50a706, 0xffffffff,
        0x50962f, 0x509c71, 0x5097a9, 0xffffffff, 0x50964d, 0x509c94, 0x5097cb, 0xffffffff,
        0x50966b, 0x509cb7, 0x5097ed, 0xffffffff, 0x509689, 0x509cda, 0x50980f, 0xffffffff,
        0x5096a7, 0x509cfd, 0x509831, 0xffffffff, 0x5096c5, 0x509d20, 0x509853, 0xffffffff,
        0x50962f, 0x509d43, 0x509875, 0xffffffff, 0x50964d, 0x509d66, 0x509897, 0xffffffff,
        0x50966b, 0x509d89, 0x5098b9, 0xffffffff, 0x509689, 0x509dac, 0x5098db, 0xffffffff,
        0x5096a7, 0x509dcf, 0x5098fd, 0xffffffff, 0x5096c5, 0x509df2, 0x50991f, 0xffffffff,
        0x50962f, 0x509e15, 0x509941, 0xffffffff, 0x50964d, 0x509e38, 0x509963, 0xffffffff,
        0x50966b, 0x509e5b, 0x509985, 0xffffffff, 0x509689, 0x509e7e, 0x5099a7, 0xffffffff,
        0x5096a7, 0x509ea1, 0x5099c9, 0xffffffff, 0x5096c5, 0x509ec4, 0x5099eb, 0xffffffff,
        0x50962f, 0x509ee7, 0x509a0d, 0xffffffff, 0x50964d, 0x509f0a, 0x509a2f, 0xffffffff,
        0x50966b, 0x509f2d, 0x509a51, 0xffffffff, 0x509689, 0x509f50, 0x509a73, 0xffffffff,
        0x5096a7, 0x509f73, 0x509a95, 0xffffffff, 0x5096c5, 0x509f96, 0x509ab7, 0xffffffff,
        0x50962f, 0x509fb9, 0x509ad9, 0xffffffff, 0x50964d, 0x509fdc, 0x509afb, 0xffffffff,
        0x50966b, 0x509fff, 0x509b1d, 0xffffffff, 0x509689, 0x50a022, 0x509b3f, 0xffffffff,
        0x5096a7, 0x50a045, 0x509b61, 0xffffffff, 0x5096c5, 0x50a068, 0x509b83, 0xffffffff,
        0x50962f, 0x50a08b, 0x509ba5, 0xffffffff, 0x50964d, 0x50a0ae, 0x509bc7, 0xffffffff,
        0x50966b, 0x50a0d1, 0x509be9, 0xffffffff, 0x509689, 0x50a0f4, 0x509c0b, 0xffffffff,
        0x5096a7, 0x50a117, 0x509c2d, 0xffffffff, 0x5096c5, 0x50a13a, 0x509c4f, 0xffffffff,
    ];

    #[test]
    fn the_speech_entries_keep_the_keys_of_their_old_addresses() {
        let texts = &crate::test_support::own::menus().texts;
        let mut now = Vec::new();
        for row in &texts.car_speech {
            now.extend(row.iter().map(crate::memory::session::PlainDword::get));
        }
        for row in &texts.stage_speech {
            now.extend(row.iter().map(crate::memory::session::PlainDword::get));
        }
        assert_eq!(now.len(), SPEECH_ADDRESSES.len());
        for (n, (key, address)) in now.iter().zip(SPEECH_ADDRESSES).enumerate() {
            let expected = if address == 0xffff_ffff {
                address
            } else {
                address - 0x506E6C + crate::audio::names_store::FIRST_KEY
            };
            assert_eq!(*key, expected, "entry {n}");
        }
    }

    #[test]
    fn every_key_names_its_text_and_every_speech_row_its_entries() {
        let texts = &crate::test_support::own::menus().texts;
        for (n, text) in MenuText::ALL.iter().enumerate() {
            assert_eq!(text.key() as usize, n + 1);
            let at = text_at(texts, text.key()).unwrap();
            assert!(!at.text("a test text").unwrap().is_empty());
        }
        assert!(text_at(texts, 0).is_err());
        assert!(text_at(texts, MenuText::ALL.len() as u32 + 1).is_err());
        assert_eq!(SpeechRow::Car(1).key(), SPEECH_KEY_BASE + 3);
        assert_eq!(SpeechRow::Stage(0).key(), SPEECH_KEY_BASE + 72);
        assert_eq!(speech_dword(texts, CAR_ENTRIES + STAGE_ENTRIES), None);
    }
}
