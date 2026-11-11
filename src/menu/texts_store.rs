//! The menus' texts, out of RAL.EXE's image (IDA 0x731B82 .. 0x734BC6):
//! two log texts of the map's polygon code, the keys menu (`MenuTexts.keys_menu`), each menu
//! screen's picture path (one copy per screen, named for the screen that loads it; the
//! others by their file), the stage maps', car specifications' and slides' paths (which
//! `menu::lists_store`'s tables name by handle, `text_key`), and two tables of speech samples' names
//! (`car_speech`, `stage_speech`: handles of `audio::names_store`'s names, `speech_entry`). Nothing writes
//! the texts. `moved::MOVED` points the range at it.
//!
//! Every address of the texts is 2 past a multiple of 4: a two-byte lead and `align(4)` keep
//! each image address's alignment; the keys menu's dword is `Loose`. The speech tables
//! after the texts sit on dword boundaries (`SPEECH_PAD` bytes before them, which the
//! handles of `text_key` count).

use crate::menu::owner::menus;
use crate::session::{PlainBytes, PlainDword, PlainLoose, with_runs};

#[repr(C, align(4))]
#[derive(rc2k_derive::Cells)]
pub struct MenuTexts {
    /// Not in the image: two bytes so that each address keeps its alignment mod 4.
    pub _lead: PlainBytes<2>,
    /// 0x731B82: "No point in poly".
    pub no_point_text: PlainBytes<17>,
    /// 0x731B93: "While scanning right didnt find point 2".
    pub no_second_point_text: PlainBytes<40>,
    /// 0x731BBB: `dword_731BBB`: the keys menu (`menu::handle_keys`, `screen_loop`).
    pub keys_menu: PlainLoose,
    /// 0x731BBF: "files\g\graphics\screens\loading.pcx".
    pub loading_pcx: PlainBytes<37>,
    /// 0x731BE4: "files\g\graphics\screens\mainscr.png".
    pub leave_question_picture: PlainBytes<37>,
    /// 0x731C09: "files\g\graphics\screens\mob_welc.png".
    pub mob_welc_png: PlainBytes<38>,
    /// 0x731C2F: "files\g\graphics\screens\a8_welc.png".
    pub a8_welc_png: PlainBytes<37>,
    /// 0x731C54: "files\g\graphics\screens\titlebar.png".
    pub titlebar_png: PlainBytes<38>,
    /// 0x731C7A: "files\g\graphics\screens\mainscr.png".
    pub main_menu_picture: PlainBytes<37>,
    /// 0x731C9F: "files\g\graphics\screens\sral.png".
    pub start_rally_picture: PlainBytes<34>,
    /// 0x731CC1: "files\g\graphics\screens\nral.png".
    pub network_rally_picture: PlainBytes<34>,
    /// 0x731CE3: "files\g\graphics\screens\pdet1.png".
    pub player1_details_picture: PlainBytes<35>,
    /// 0x731D06: "files\g\graphics\screens\pdet1.png".
    pub player2_details_picture: PlainBytes<35>,
    /// 0x731D29: "files\g\graphics\screens\pdet1.png".
    pub player3_details_picture: PlainBytes<35>,
    /// 0x731D4C: "files\g\graphics\screens\pdet1.png".
    pub player4_details_picture: PlainBytes<35>,
    /// 0x731D6F: "files\g\graphics\screens\pdetc.png".
    pub player_details_picture: PlainBytes<35>,
    /// 0x731D92: "files\g\graphics\screens\opts.png".
    pub options_picture: PlainBytes<34>,
    /// 0x731DB4: "files\g\graphics\screens\info.png".
    pub information_picture: PlainBytes<34>,
    /// 0x731DD6: "files\g\graphics\screens\video.png".
    pub video_png: PlainBytes<35>,
    /// 0x731DF9: "files\g\graphics\screens\visual.png".
    pub visual_preferences_picture: PlainBytes<36>,
    /// 0x731E1D: "files\g\graphics\screens\visual.png".
    pub visual_options_picture: PlainBytes<36>,
    /// 0x731E41: "files\g\graphics\screens\cspecs.png".
    pub cspecs_png: PlainBytes<36>,
    /// 0x731E65: "files\g\graphics\screens\cspecs.png".
    pub cspecs_png_2: PlainBytes<36>,
    /// 0x731E89: "files\g\graphics\screens\loadsv.png".
    pub load_championship_picture: PlainBytes<36>,
    /// 0x731EAD: "files\g\graphics\screens\loadsv.png".
    pub save_championship_picture: PlainBytes<36>,
    /// 0x731ED1: "files\g\graphics\screens\loadsv.png".
    pub save_replay_picture: PlainBytes<36>,
    /// 0x731EF5: "files\g\graphics\screens\loadsv.png".
    pub load_replay_picture: PlainBytes<36>,
    /// 0x731F19: "files\g\graphics\screens\cspecs.png".
    pub select_car_picture: PlainBytes<36>,
    /// 0x731F3D: "files\g\graphics\screens\cspecs.png".
    pub car_specs_display_picture: PlainBytes<36>,
    /// 0x731F61: "files\g\graphics\screens\cspecs.png".
    pub cspecs_png_3: PlainBytes<36>,
    /// 0x731F85: "files\g\graphics\screens\btimes.png".
    pub view_best_times_picture: PlainBytes<36>,
    /// 0x731FA9: "files\g\graphics\screens\bascores.png".
    pub view_best_scores_picture: PlainBytes<38>,
    /// 0x731FCF: "files\g\graphics\screens\btimes.png".
    pub best_times_picture: PlainBytes<36>,
    /// 0x731FF3: "files\g\graphics\screens\btimes.png".
    pub arcade_best_times_picture: PlainBytes<36>,
    /// 0x732017: "files\g\graphics\screens\btimes.png".
    pub time_trial_best_times_picture: PlainBytes<36>,
    /// 0x73203B: "files\g\graphics\screens\bascores.png".
    pub arcade_best_scores_picture: PlainBytes<38>,
    /// 0x732061: "files\g\graphics\screens\bascores.png".
    pub best_class_scores_picture: PlainBytes<38>,
    /// 0x732087: "files\g\graphics\screens\bascores.png".
    pub best_overall_scores_picture: PlainBytes<38>,
    /// 0x7320AD: "files\g\graphics\screens\audio.png".
    pub audio_setup_picture: PlainBytes<35>,
    /// 0x7320D0: "files\g\graphics\screens\audio.png".
    pub advanced_audio_picture: PlainBytes<35>,
    /// 0x7320F3: "files\g\graphics\screens\vcourses.png".
    pub vcourses_png: PlainBytes<38>,
    /// 0x732119: "files\g\graphics\screens\vcourses.png".
    pub vcourses_png_2: PlainBytes<38>,
    /// 0x73213F: "files\g\graphics\screens\vcourses.png".
    pub vcourses_png_3: PlainBytes<38>,
    /// 0x732165: "files\g\graphics\screens\vcourses.png".
    pub vcourses_png_4: PlainBytes<38>,
    /// 0x73218B: "files\g\graphics\screens\ukmap.png".
    pub ukmap_png: PlainBytes<35>,
    /// 0x7321AE: "files\g\graphics\screens\loading.png".
    pub loading_png: PlainBytes<37>,
    /// 0x7321D3: "files\g\graphics\screens\control.png".
    pub player1_controller_picture: PlainBytes<37>,
    /// 0x7321F8: "files\g\graphics\screens\control.png".
    pub player1_buttons_picture: PlainBytes<37>,
    /// 0x73221D: "files\g\graphics\screens\control.png".
    pub player2_controller_picture: PlainBytes<37>,
    /// 0x732242: "files\g\graphics\screens\control.png".
    pub player2_buttons_picture: PlainBytes<37>,
    /// 0x732267: "files\g\graphics\screens\control.png".
    pub player3_controller_picture: PlainBytes<37>,
    /// 0x73228C: "files\g\graphics\screens\control.png".
    pub player3_buttons_picture: PlainBytes<37>,
    /// 0x7322B1: "files\g\graphics\screens\control.png".
    pub player4_controller_picture: PlainBytes<37>,
    /// 0x7322D6: "files\g\graphics\screens\control.png".
    pub player4_buttons_picture: PlainBytes<37>,
    /// 0x7322FB: "files\g\graphics\screens\defkeys.png".
    pub redefine_keys_player1_picture: PlainBytes<37>,
    /// 0x732320: "files\g\graphics\screens\defkeys.png".
    pub redefine_keys_player2_picture: PlainBytes<37>,
    /// 0x732345: "files\g\graphics\screens\defkeys.png".
    pub redefine_keys_player3_picture: PlainBytes<37>,
    /// 0x73236A: "files\g\graphics\screens\defkeys.png".
    pub redefine_keys_player4_picture: PlainBytes<37>,
    /// 0x73238F: "files\g\graphics\screens\sral.png".
    pub sral_png: PlainBytes<34>,
    /// 0x7323B1: "files\g\graphics\screens\sral.png".
    pub sral_png_2: PlainBytes<34>,
    /// 0x7323D3: "files\g\graphics\screens\sral.png".
    pub championship_new_simulation_picture: PlainBytes<34>,
    /// 0x7323F5: "files\g\graphics\screens\sral.png".
    pub new_simulation_picture: PlainBytes<34>,
    /// 0x732417: "files\g\graphics\screens\endday.png".
    pub endday_png: PlainBytes<36>,
    /// 0x73243B: "files\g\graphics\screens\endday.png".
    pub endday_png_2: PlainBytes<36>,
    /// 0x73245F: "files\g\graphics\screens\ukmap.png".
    pub rally_stages_picture: PlainBytes<35>,
    /// 0x732482: "files\g\graphics\screens\serv00.png".
    pub serv00_png: PlainBytes<36>,
    /// 0x7324A6: "files\g\graphics\screens\serv00.png".
    pub serv00_png_2: PlainBytes<36>,
    /// 0x7324CA: "files\g\graphics\screens\serv00.png".
    pub serv00_png_3: PlainBytes<36>,
    /// 0x7324EE: "files\g\graphics\screens\results1.png".
    pub class_stage_results_picture: PlainBytes<38>,
    /// 0x732514: "files\g\graphics\screens\results1.png".
    pub overall_stage_results_picture: PlainBytes<38>,
    /// 0x73253A: "files\g\graphics\screens\results1.png".
    pub class_rally_results_picture: PlainBytes<38>,
    /// 0x732560: "files\g\graphics\screens\results1.png".
    pub overall_rally_results_picture: PlainBytes<38>,
    /// 0x732586: "files\g\graphics\screens\results1.png".
    pub results1_png: PlainBytes<38>,
    /// 0x7325AC: "files\g\graphics\screens\results1.png".
    pub results1_png_2: PlainBytes<38>,
    /// 0x7325D2: the copy `menu::section_times_loop` loads as its background: "files\g\graphics\screens\results1.png".
    pub section_times_png: PlainBytes<38>,
    /// 0x7325F8: "files\g\graphics\screens\service.png".
    pub inspection_area_picture: PlainBytes<37>,
    /// 0x73261D: "files\g\graphics\screens\roptions.png".
    pub championship_service_review_picture: PlainBytes<38>,
    /// 0x732643: "files\g\graphics\screens\roptions.png".
    pub championship_standings_review_picture: PlainBytes<38>,
    /// 0x732669: "files\g\graphics\screens\roptions.png".
    pub championship_races_review_picture: PlainBytes<38>,
    /// 0x73268F: "files\g\graphics\screens\roptions.png".
    pub championship_end_review_picture: PlainBytes<38>,
    /// 0x7326B5: "files\g\graphics\screens\opts.png".
    pub in_game_options_picture: PlainBytes<34>,
    /// 0x7326D7: "files\g\graphics\screens\welcomer.png".
    pub welcomer_png: PlainBytes<38>,
    /// 0x7326FD: "files\g\graphics\screens\welvx.png".
    pub welvx_png: PlainBytes<35>,
    /// 0x732720: "files\g\graphics\screens\welpir.png".
    pub welpir_png: PlainBytes<36>,
    /// 0x732744: "files\g\graphics\screens\welsct.png".
    pub welsct_png: PlainBytes<36>,
    /// 0x732768: "files\g\graphics\screens\weljc.png".
    pub weljc_png: PlainBytes<35>,
    /// 0x73278B: "files\g\graphics\screens\welul.png".
    pub welul_png: PlainBytes<35>,
    /// 0x7327AE: "files\g\graphics\screens\welmnx.png".
    pub welmnx_png: PlainBytes<36>,
    /// 0x7327D2: "files\g\graphics\screens\a8_1.png".
    pub a8_1_png: PlainBytes<34>,
    /// 0x7327F4: "files\g\graphics\screens\a8_2.png".
    pub a8_2_png: PlainBytes<34>,
    /// 0x732816: "files\g\graphics\screens\a8_3.png".
    pub a8_3_png: PlainBytes<34>,
    /// 0x732838: "files\g\graphics\screens\a8_4.png".
    pub a8_4_png: PlainBytes<34>,
    /// 0x73285A: "files\g\graphics\screens\a8_5.png".
    pub a8_5_png: PlainBytes<34>,
    /// 0x73287C: "files\g\graphics\screens\a8_6.png".
    pub a8_6_png: PlainBytes<34>,
    /// 0x73289E: "files\g\graphics\screens\setcar.png".
    pub setcar_png: PlainBytes<36>,
    /// 0x7328C2: "files\g\graphics\screens\champoc.png".
    pub class_championship_standings_picture: PlainBytes<37>,
    /// 0x7328E7: "files\g\graphics\screens\champoc.png".
    pub championship_standings_picture: PlainBytes<37>,
    /// 0x73290C: "files\g\graphics\screens\arcade1.png".
    pub simple_select_players_picture: PlainBytes<37>,
    /// 0x732931: "files\g\graphics\screens\roptions.png".
    pub single_rally_review_picture: PlainBytes<38>,
    /// 0x732957: "files\g\graphics\screens\roptions.png".
    pub arcade_review_options_picture: PlainBytes<38>,
    /// 0x73297D: "files\g\graphics\screens\arcade1.png".
    pub time_trial_select_players_picture: PlainBytes<37>,
    /// 0x7329A2: "files\g\graphics\screens\roptions.png".
    pub arcade_next_level_review_picture: PlainBytes<38>,
    /// 0x7329C8: "files\g\graphics\screens\roptions.png".
    pub arcade_retry_review_picture: PlainBytes<38>,
    /// 0x7329EE: "files\g\graphics\screens\roptions.png".
    pub roptions_png: PlainBytes<38>,
    /// 0x732A14: "files\g\graphics\screens\results1.png".
    pub arcade_level_end_results_picture: PlainBytes<38>,
    /// 0x732A3A: "files\g\graphics\screens\results1.png".
    pub stage_result_picture: PlainBytes<38>,
    /// 0x732A60: "files\g\graphics\screens\results1.png".
    pub results1_png_4: PlainBytes<38>,
    /// 0x732A86: "files\g\graphics\screens\results1.png".
    pub single_rally_two_results_picture: PlainBytes<38>,
    /// 0x732AAC: "files\g\graphics\screens\results1.png".
    pub single_rally_three_four_results_picture: PlainBytes<38>,
    /// 0x732AD2: "files\g\graphics\screens\results1.png".
    pub time_trial_results_picture: PlainBytes<38>,
    /// 0x732AF8: "files\g\graphics\screens\results1.png".
    pub single_rally_overall_results_picture: PlainBytes<38>,
    /// 0x732B1E: "files\g\graphics\screens\results1.png".
    pub results1_png_5: PlainBytes<38>,
    /// 0x732B44: "files\g\graphics\screens\selstage.png".
    pub selstage_png: PlainBytes<38>,
    /// 0x732B6A: "files\g\graphics\screens\selstage.png".
    pub selstage_png_2: PlainBytes<38>,
    /// 0x732B90: "files\g\graphics\screens\arcstage.png".
    pub arcstage_png: PlainBytes<38>,
    /// 0x732BB6: "files\g\graphics\screens\arcade1.png".
    pub arcade1_png: PlainBytes<37>,
    /// 0x732BDB: "files\g\graphics\screens\Selarc.png".
    pub arcade_select_level_picture: PlainBytes<36>,
    /// 0x732BFF: "files\g\graphics\screens\results1.png".
    pub arcade_results_picture: PlainBytes<38>,
    /// 0x732C25: "files\g\graphics\screens\roptions.png".
    pub roptions_png_2: PlainBytes<38>,
    /// 0x732C4B: "files\g\graphics\screens\racing.png".
    pub racing_png: PlainBytes<36>,
    /// 0x732C6F: "files\g\graphics\screens\racing.png".
    pub racing_png_2: PlainBytes<36>,
    /// 0x732C93: "files\g\graphics\sprites\mbuts00.pcx".
    pub mbuts00_pcx: PlainBytes<37>,
    /// 0x732CB8: "files\g\graphics\screens\serv00.png".
    pub serv00_png_4: PlainBytes<36>,
    /// 0x732CDC: "files\g\graphics\screens\serv01.png".
    pub serv01_png: PlainBytes<36>,
    /// 0x732D00: "files\g\graphics\screens\serv02.png".
    pub serv02_png: PlainBytes<36>,
    /// 0x732D24: "files\g\graphics\screens\serv03.png".
    pub serv03_png: PlainBytes<36>,
    /// 0x732D48: "files\g\graphics\screens\serv04.png".
    pub serv04_png: PlainBytes<36>,
    /// 0x732D6C: "files\g\graphics\screens\serv05.png".
    pub serv05_png: PlainBytes<36>,
    /// 0x732D90: "files\g\graphics\screens\serv06.png".
    pub serv06_png: PlainBytes<36>,
    /// 0x732DB4: "files\g\graphics\screens\serv07.png".
    pub serv07_png: PlainBytes<36>,
    /// 0x732DD8: "files\g\graphics\screens\serv08.png".
    pub serv08_png: PlainBytes<36>,
    /// 0x732DFC: "files\g\graphics\screens\serv09.png".
    pub serv09_png: PlainBytes<36>,
    /// 0x732E20: "files\g\graphics\screens\serv10.png".
    pub serv10_png: PlainBytes<36>,
    /// 0x732E44: "files\g\graphics\screens\serv11.png".
    pub serv11_png: PlainBytes<36>,
    /// 0x732E68: "files\g\graphics\screens\serv12.png".
    pub serv12_png: PlainBytes<36>,
    /// 0x732E8C: "files\g\graphics\screens\serv13.png".
    pub serv13_png: PlainBytes<36>,
    /// 0x732EB0: "files\g\graphics\screens\serv14.png".
    pub serv14_png: PlainBytes<36>,
    /// 0x732ED4: "files\g\graphics\screens\serv15.png".
    pub serv15_png: PlainBytes<36>,
    /// 0x732EF8: "files\g\graphics\screens\serv16.png".
    pub serv16_png: PlainBytes<36>,
    /// 0x732F1C: "files\g\graphics\screens\serv17.png".
    pub serv17_png: PlainBytes<36>,
    /// 0x732F40: "files\g\graphics\screens\serv18.png".
    pub serv18_png: PlainBytes<36>,
    /// 0x732F64: "files\g\graphics\screens\serv19.png".
    pub serv19_png: PlainBytes<36>,
    /// 0x732F88: "files\g\graphics\screens\serv20.png".
    pub serv20_png: PlainBytes<36>,
    /// 0x732FAC: "files\g\graphics\screens\serv21.png".
    pub serv21_png: PlainBytes<36>,
    /// 0x732FD0: "files\g\graphics\screens\serv22.png".
    pub serv22_png: PlainBytes<36>,
    /// 0x732FF4: "files\g\graphics\screens\serv33.png".
    pub serv33_png: PlainBytes<36>,
    /// 0x733018: "files\g\graphics\screens\maps\vrw_001.png".
    pub vrw_001_png: PlainBytes<42>,
    /// 0x733042: "files\g\graphics\screens\maps\vrw_002.png".
    pub vrw_002_png: PlainBytes<42>,
    /// 0x73306C: "files\g\graphics\screens\maps\vrw_003.png".
    pub vrw_003_png: PlainBytes<42>,
    /// 0x733096: "files\g\graphics\screens\maps\vrw_004.png".
    pub vrw_004_png: PlainBytes<42>,
    /// 0x7330C0: "files\g\graphics\screens\maps\vrw_005.png".
    pub vrw_005_png: PlainBytes<42>,
    /// 0x7330EA: "files\g\graphics\screens\maps\vrw_006.png".
    pub vrw_006_png: PlainBytes<42>,
    /// 0x733114: "files\g\graphics\screens\maps\pir_001.png".
    pub pir_001_png: PlainBytes<42>,
    /// 0x73313E: "files\g\graphics\screens\maps\pir_002.png".
    pub pir_002_png: PlainBytes<42>,
    /// 0x733168: "files\g\graphics\screens\maps\pir_003.png".
    pub pir_003_png: PlainBytes<42>,
    /// 0x733192: "files\g\graphics\screens\maps\pir_004.png".
    pub pir_004_png: PlainBytes<42>,
    /// 0x7331BC: "files\g\graphics\screens\maps\pir_005.png".
    pub pir_005_png: PlainBytes<42>,
    /// 0x7331E6: "files\g\graphics\screens\maps\pir_006.png".
    pub pir_006_png: PlainBytes<42>,
    /// 0x733210: "files\g\graphics\screens\maps\sc_001.png".
    pub sc_001_png: PlainBytes<41>,
    /// 0x733239: "files\g\graphics\screens\maps\sc_002.png".
    pub sc_002_png: PlainBytes<41>,
    /// 0x733262: "files\g\graphics\screens\maps\sc_003.png".
    pub sc_003_png: PlainBytes<41>,
    /// 0x73328B: "files\g\graphics\screens\maps\sc_004.png".
    pub sc_004_png: PlainBytes<41>,
    /// 0x7332B4: "files\g\graphics\screens\maps\sc_005.png".
    pub sc_005_png: PlainBytes<41>,
    /// 0x7332DD: "files\g\graphics\screens\maps\sc_006.png".
    pub sc_006_png: PlainBytes<41>,
    /// 0x733306: "files\g\graphics\screens\maps\jc_001.png".
    pub jc_001_png: PlainBytes<41>,
    /// 0x73332F: "files\g\graphics\screens\maps\jc_002.png".
    pub jc_002_png: PlainBytes<41>,
    /// 0x733358: "files\g\graphics\screens\maps\jc_003.png".
    pub jc_003_png: PlainBytes<41>,
    /// 0x733381: "files\g\graphics\screens\maps\jc_004.png".
    pub jc_004_png: PlainBytes<41>,
    /// 0x7333AA: "files\g\graphics\screens\maps\jc_005.png".
    pub jc_005_png: PlainBytes<41>,
    /// 0x7333D3: "files\g\graphics\screens\maps\jc_006.png".
    pub jc_006_png: PlainBytes<41>,
    /// 0x7333FC: "files\g\graphics\screens\maps\ul_001.png".
    pub ul_001_png: PlainBytes<41>,
    /// 0x733425: "files\g\graphics\screens\maps\ul_002.png".
    pub ul_002_png: PlainBytes<41>,
    /// 0x73344E: "files\g\graphics\screens\maps\ul_003.png".
    pub ul_003_png: PlainBytes<41>,
    /// 0x733477: "files\g\graphics\screens\maps\ul_004.png".
    pub ul_004_png: PlainBytes<41>,
    /// 0x7334A0: "files\g\graphics\screens\maps\ul_005.png".
    pub ul_005_png: PlainBytes<41>,
    /// 0x7334C9: "files\g\graphics\screens\maps\ul_006.png".
    pub ul_006_png: PlainBytes<41>,
    /// 0x7334F2: "files\g\graphics\screens\maps\mnx_001.png".
    pub mnx_001_png: PlainBytes<42>,
    /// 0x73351C: "files\g\graphics\screens\maps\mnx_002.png".
    pub mnx_002_png: PlainBytes<42>,
    /// 0x733546: "files\g\graphics\screens\maps\mnx_003.png".
    pub mnx_003_png: PlainBytes<42>,
    /// 0x733570: "files\g\graphics\screens\maps\mnx_004.png".
    pub mnx_004_png: PlainBytes<42>,
    /// 0x73359A: "files\g\graphics\screens\maps\mnx_005.png".
    pub mnx_005_png: PlainBytes<42>,
    /// 0x7335C4: "files\g\graphics\screens\maps\mnx_006.png".
    pub mnx_006_png: PlainBytes<42>,
    /// 0x7335EE: "files\g\graphics\screens\cspecs\mgnvc.png".
    pub mgnvc_png: PlainBytes<42>,
    /// 0x733618: "files\g\graphics\screens\cspecs\vxavc.png".
    pub vxavc_png: PlainBytes<42>,
    /// 0x733642: "files\g\graphics\screens\cspecs\stcvc.png".
    pub stcvc_png: PlainBytes<42>,
    /// 0x73366C: "files\g\graphics\screens\cspecs\hycvc.png".
    pub hycvc_png: PlainBytes<42>,
    /// 0x733696: "files\g\graphics\screens\cspecs\sbivc.png".
    pub sbivc_png: PlainBytes<42>,
    /// 0x7336C0: "files\g\graphics\screens\cspecs\pgovc.png".
    pub pgovc_png: PlainBytes<42>,
    /// 0x7336EA: "files\g\graphics\screens\cspecs\mxpvc.png".
    pub mxpvc_png: PlainBytes<42>,
    /// 0x733714: "files\g\graphics\screens\cspecs\mtlvc.png".
    pub mtlvc_png: PlainBytes<42>,
    /// 0x73373E: "files\g\graphics\screens\cspecs\vwgvc.png".
    pub vwgvc_png: PlainBytes<42>,
    /// 0x733768: "files\g\graphics\screens\cspecs\nsavc.png".
    pub nsavc_png: PlainBytes<42>,
    /// 0x733792: "files\g\graphics\screens\cspecs\ptwvc.png".
    pub ptwvc_png: PlainBytes<42>,
    /// 0x7337BC: "files\g\graphics\screens\cspecs\ctsvc.png".
    pub ctsvc_png: PlainBytes<42>,
    /// 0x7337E6: "files\g\graphics\screens\cspecs\hncvc.png".
    pub hncvc_png: PlainBytes<42>,
    /// 0x733810: "files\g\graphics\screens\cspecs\nsmvc.png".
    pub nsmvc_png: PlainBytes<42>,
    /// 0x73383A: "files\g\graphics\screens\cspecs\pgtvc.png".
    pub pgtvc_png: PlainBytes<42>,
    /// 0x733864: "files\g\graphics\screens\cspecs\frtvc.png".
    pub frtvc_png: PlainBytes<42>,
    /// 0x73388E: "files\g\graphics\screens\cspecs\sdovc.png".
    pub sdovc_png: PlainBytes<42>,
    /// 0x7338B8: "files\g\graphics\screens\cspecs\stivc.png".
    pub stivc_png: PlainBytes<42>,
    /// 0x7338E2: "files\g\graphics\screens\cspecs\sdfvc.png".
    pub sdfvc_png: PlainBytes<42>,
    /// 0x73390C: "files\g\graphics\screens\cspecs\ptcvc.png".
    pub ptcvc_png: PlainBytes<42>,
    /// 0x733936: "files\g\graphics\screens\cspecs\fmxvc.png".
    pub fmxvc_png: PlainBytes<42>,
    /// 0x733960: "files\g\graphics\screens\cspecs\fpmvc.png".
    pub fpmvc_png: PlainBytes<42>,
    /// 0x73398A: "files\g\graphics\screens\cspecs\oscvc.png".
    pub oscvc_png: PlainBytes<42>,
    /// 0x7339B4: "files\g\graphics\screens\cspecs\rclvc.png".
    pub rclvc_png: PlainBytes<42>,
    /// 0x7339DE: "files\g\graphics\screens\cspecs\mgndm.png".
    pub mgndm_png: PlainBytes<42>,
    /// 0x733A08: "files\g\graphics\screens\cspecs\vxadm.png".
    pub vxadm_png: PlainBytes<42>,
    /// 0x733A32: "files\g\graphics\screens\cspecs\stcdm.png".
    pub stcdm_png: PlainBytes<42>,
    /// 0x733A5C: "files\g\graphics\screens\cspecs\hycdm.png".
    pub hycdm_png: PlainBytes<42>,
    /// 0x733A86: "files\g\graphics\screens\cspecs\sbidm.png".
    pub sbidm_png: PlainBytes<42>,
    /// 0x733AB0: "files\g\graphics\screens\cspecs\pgodm.png".
    pub pgodm_png: PlainBytes<42>,
    /// 0x733ADA: "files\g\graphics\screens\cspecs\mxpdm.png".
    pub mxpdm_png: PlainBytes<42>,
    /// 0x733B04: "files\g\graphics\screens\cspecs\mtldm.png".
    pub mtldm_png: PlainBytes<42>,
    /// 0x733B2E: "files\g\graphics\screens\cspecs\vwgdm.png".
    pub vwgdm_png: PlainBytes<42>,
    /// 0x733B58: "files\g\graphics\screens\cspecs\nsadm.png".
    pub nsadm_png: PlainBytes<42>,
    /// 0x733B82: "files\g\graphics\screens\cspecs\ptwdm.png".
    pub ptwdm_png: PlainBytes<42>,
    /// 0x733BAC: "files\g\graphics\screens\cspecs\ctsdm.png".
    pub ctsdm_png: PlainBytes<42>,
    /// 0x733BD6: "files\g\graphics\screens\cspecs\hncdm.png".
    pub hncdm_png: PlainBytes<42>,
    /// 0x733C00: "files\g\graphics\screens\cspecs\nsmdm.png".
    pub nsmdm_png: PlainBytes<42>,
    /// 0x733C2A: "files\g\graphics\screens\cspecs\pgtdm.png".
    pub pgtdm_png: PlainBytes<42>,
    /// 0x733C54: "files\g\graphics\screens\cspecs\frtdm.png".
    pub frtdm_png: PlainBytes<42>,
    /// 0x733C7E: "files\g\graphics\screens\cspecs\sdodm.png".
    pub sdodm_png: PlainBytes<42>,
    /// 0x733CA8: "files\g\graphics\screens\cspecs\stidm.png".
    pub stidm_png: PlainBytes<42>,
    /// 0x733CD2: "files\g\graphics\screens\cspecs\sdfdm.png".
    pub sdfdm_png: PlainBytes<42>,
    /// 0x733CFC: "files\g\graphics\screens\cspecs\ptcdm.png".
    pub ptcdm_png: PlainBytes<42>,
    /// 0x733D26: "files\g\graphics\screens\cspecs\fmxdm.png".
    pub fmxdm_png: PlainBytes<42>,
    /// 0x733D50: "files\g\graphics\screens\cspecs\fpmdm.png".
    pub fpmdm_png: PlainBytes<42>,
    /// 0x733D7A: "files\g\graphics\screens\cspecs\oscdm.png".
    pub oscdm_png: PlainBytes<42>,
    /// 0x733DA4: "files\g\graphics\screens\cspecs\rcldm.png".
    pub rcldm_png: PlainBytes<42>,
    /// 0x733DCE: "files\g\graphics\screens\cspecs\mgnen.png".
    pub mgnen_png: PlainBytes<42>,
    /// 0x733DF8: "files\g\graphics\screens\cspecs\vxaen.png".
    pub vxaen_png: PlainBytes<42>,
    /// 0x733E22: "files\g\graphics\screens\cspecs\stcen.png".
    pub stcen_png: PlainBytes<42>,
    /// 0x733E4C: "files\g\graphics\screens\cspecs\hycen.png".
    pub hycen_png: PlainBytes<42>,
    /// 0x733E76: "files\g\graphics\screens\cspecs\sbien.png".
    pub sbien_png: PlainBytes<42>,
    /// 0x733EA0: "files\g\graphics\screens\cspecs\pgoen.png".
    pub pgoen_png: PlainBytes<42>,
    /// 0x733ECA: "files\g\graphics\screens\cspecs\mxpen.png".
    pub mxpen_png: PlainBytes<42>,
    /// 0x733EF4: "files\g\graphics\screens\cspecs\mtlen.png".
    pub mtlen_png: PlainBytes<42>,
    /// 0x733F1E: "files\g\graphics\screens\cspecs\vwgen.png".
    pub vwgen_png: PlainBytes<42>,
    /// 0x733F48: "files\g\graphics\screens\cspecs\nsaen.png".
    pub nsaen_png: PlainBytes<42>,
    /// 0x733F72: "files\g\graphics\screens\cspecs\ptwen.png".
    pub ptwen_png: PlainBytes<42>,
    /// 0x733F9C: "files\g\graphics\screens\cspecs\ctsen.png".
    pub ctsen_png: PlainBytes<42>,
    /// 0x733FC6: "files\g\graphics\screens\cspecs\hncen.png".
    pub hncen_png: PlainBytes<42>,
    /// 0x733FF0: "files\g\graphics\screens\cspecs\nsmen.png".
    pub nsmen_png: PlainBytes<42>,
    /// 0x73401A: "files\g\graphics\screens\cspecs\pgten.png".
    pub pgten_png: PlainBytes<42>,
    /// 0x734044: "files\g\graphics\screens\cspecs\frten.png".
    pub frten_png: PlainBytes<42>,
    /// 0x73406E: "files\g\graphics\screens\cspecs\sdoen.png".
    pub sdoen_png: PlainBytes<42>,
    /// 0x734098: "files\g\graphics\screens\cspecs\stien.png".
    pub stien_png: PlainBytes<42>,
    /// 0x7340C2: "files\g\graphics\screens\cspecs\sdfen.png".
    pub sdfen_png: PlainBytes<42>,
    /// 0x7340EC: "files\g\graphics\screens\cspecs\ptcen.png".
    pub ptcen_png: PlainBytes<42>,
    /// 0x734116: "files\g\graphics\screens\cspecs\fmxen.png".
    pub fmxen_png: PlainBytes<42>,
    /// 0x734140: "files\g\graphics\screens\cspecs\fpmen.png".
    pub fpmen_png: PlainBytes<42>,
    /// 0x73416A: "files\g\graphics\screens\cspecs\oscen.png".
    pub oscen_png: PlainBytes<42>,
    /// 0x734194: "files\g\graphics\screens\cspecs\rclen.png".
    pub rclen_png: PlainBytes<42>,
    /// 0x7341BE: "files\g\graphics\screens\cspecs\mgntc.png".
    pub mgntc_png: PlainBytes<42>,
    /// 0x7341E8: "files\g\graphics\screens\cspecs\vxatc.png".
    pub vxatc_png: PlainBytes<42>,
    /// 0x734212: "files\g\graphics\screens\cspecs\stctc.png".
    pub stctc_png: PlainBytes<42>,
    /// 0x73423C: "files\g\graphics\screens\cspecs\hyctc.png".
    pub hyctc_png: PlainBytes<42>,
    /// 0x734266: "files\g\graphics\screens\cspecs\sbitc.png".
    pub sbitc_png: PlainBytes<42>,
    /// 0x734290: "files\g\graphics\screens\cspecs\pgotc.png".
    pub pgotc_png: PlainBytes<42>,
    /// 0x7342BA: "files\g\graphics\screens\cspecs\mxptc.png".
    pub mxptc_png: PlainBytes<42>,
    /// 0x7342E4: "files\g\graphics\screens\cspecs\mtltc.png".
    pub mtltc_png: PlainBytes<42>,
    /// 0x73430E: "files\g\graphics\screens\cspecs\vwgtc.png".
    pub vwgtc_png: PlainBytes<42>,
    /// 0x734338: "files\g\graphics\screens\cspecs\nsatc.png".
    pub nsatc_png: PlainBytes<42>,
    /// 0x734362: "files\g\graphics\screens\cspecs\ptwtc.png".
    pub ptwtc_png: PlainBytes<42>,
    /// 0x73438C: "files\g\graphics\screens\cspecs\ctstc.png".
    pub ctstc_png: PlainBytes<42>,
    /// 0x7343B6: "files\g\graphics\screens\cspecs\hnctc.png".
    pub hnctc_png: PlainBytes<42>,
    /// 0x7343E0: "files\g\graphics\screens\cspecs\nsmtc.png".
    pub nsmtc_png: PlainBytes<42>,
    /// 0x73440A: "files\g\graphics\screens\cspecs\pgttc.png".
    pub pgttc_png: PlainBytes<42>,
    /// 0x734434: "files\g\graphics\screens\cspecs\frttc.png".
    pub frttc_png: PlainBytes<42>,
    /// 0x73445E: "files\g\graphics\screens\cspecs\sdotc.png".
    pub sdotc_png: PlainBytes<42>,
    /// 0x734488: "files\g\graphics\screens\cspecs\stitc.png".
    pub stitc_png: PlainBytes<42>,
    /// 0x7344B2: "files\g\graphics\screens\cspecs\sdftc.png".
    pub sdftc_png: PlainBytes<42>,
    /// 0x7344DC: "files\g\graphics\screens\cspecs\ptctc.png".
    pub ptctc_png: PlainBytes<42>,
    /// 0x734506: "files\g\graphics\screens\cspecs\fmxtc.png".
    pub fmxtc_png: PlainBytes<42>,
    /// 0x734530: "files\g\graphics\screens\cspecs\fpmtc.png".
    pub fpmtc_png: PlainBytes<42>,
    /// 0x73455A: "files\g\graphics\screens\cspecs\osctc.png".
    pub osctc_png: PlainBytes<42>,
    /// 0x734584: "files\g\graphics\screens\cspecs\rcltc.png".
    pub rcltc_png: PlainBytes<42>,
    /// 0x7345AE: ";3files\g\slides\mgnsl00.png".
    pub mgnsl00_png: PlainBytes<29>,
    /// 0x7345CB: ";3files\g\slides\vxasl00.png".
    pub vxasl00_png: PlainBytes<29>,
    /// 0x7345E8: ";3files\g\slides\stcsl00.png".
    pub stcsl00_png: PlainBytes<29>,
    /// 0x734605: ";3files\g\slides\hycsl00.png".
    pub hycsl00_png: PlainBytes<29>,
    /// 0x734622: ";3files\g\slides\sbisl00.png".
    pub sbisl00_png: PlainBytes<29>,
    /// 0x73463F: ";3files\g\slides\pgosl00.png".
    pub pgosl00_png: PlainBytes<29>,
    /// 0x73465C: ";3files\g\slides\mxpsl00.png".
    pub mxpsl00_png: PlainBytes<29>,
    /// 0x734679: ";3files\g\slides\mtlsl00.png".
    pub mtlsl00_png: PlainBytes<29>,
    /// 0x734696: ";3files\g\slides\vwgsl00.png".
    pub vwgsl00_png: PlainBytes<29>,
    /// 0x7346B3: ";3files\g\slides\nsasl00.png".
    pub nsasl00_png: PlainBytes<29>,
    /// 0x7346D0: ";3files\g\slides\ptwsl00.png".
    pub ptwsl00_png: PlainBytes<29>,
    /// 0x7346ED: ";3files\g\slides\ctssl00.png".
    pub ctssl00_png: PlainBytes<29>,
    /// 0x73470A: ";3files\g\slides\hncsl00.png".
    pub hncsl00_png: PlainBytes<29>,
    /// 0x734727: ";3files\g\slides\nsmsl00.png".
    pub nsmsl00_png: PlainBytes<29>,
    /// 0x734744: ";3files\g\slides\pgtsl00.png".
    pub pgtsl00_png: PlainBytes<29>,
    /// 0x734761: ";3files\g\slides\frtsl00.png".
    pub frtsl00_png: PlainBytes<29>,
    /// 0x73477E: ";3files\g\slides\sdosl00.png".
    pub sdosl00_png: PlainBytes<29>,
    /// 0x73479B: ";3files\g\slides\stisl00.png".
    pub stisl00_png: PlainBytes<29>,
    /// 0x7347B8: ";3files\g\slides\sdfsl00.png".
    pub sdfsl00_png: PlainBytes<29>,
    /// 0x7347D5: ";3files\g\slides\ptcsl00.png".
    pub ptcsl00_png: PlainBytes<29>,
    /// 0x7347F2: ";3files\g\slides\fmxsl00.png".
    pub fmxsl00_png: PlainBytes<29>,
    /// 0x73480F: ";3files\g\slides\fpmsl00.png".
    pub fpmsl00_png: PlainBytes<29>,
    /// 0x73482C: ";3files\g\slides\oscsl00.png".
    pub oscsl00_png: PlainBytes<29>,
    /// 0x734849: ";3files\g\slides\rclsl00.png".
    pub rclsl00_png: PlainBytes<29>,
    /// Not in the image: two bytes that put the speech tables on a dword boundary.
    pub _speech_pad: PlainBytes<2>,
    /// 0x734866: per car, its two speech samples' names and a 0.
    pub car_speech: [[PlainDword; 3]; 24],
    /// 0x734986: per stage, three speech samples' names and a 0.
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
            keys_menu: PlainLoose::new(0x0),
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
                    PlainDword::new(speech_key(0x50a15d)),
                    PlainDword::new(speech_key(0x50a17e)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a19f)),
                    PlainDword::new(speech_key(0x50a1bf)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a1df)),
                    PlainDword::new(speech_key(0x50a201)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a223)),
                    PlainDword::new(speech_key(0x50a243)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a263)),
                    PlainDword::new(speech_key(0x50a285)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a2a7)),
                    PlainDword::new(speech_key(0x50a2c5)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a3e3)),
                    PlainDword::new(speech_key(0x50a402)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a2e3)),
                    PlainDword::new(speech_key(0x50a304)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a325)),
                    PlainDword::new(speech_key(0x50a344)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a363)),
                    PlainDword::new(speech_key(0x50a384)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a3a5)),
                    PlainDword::new(speech_key(0x50a3c4)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a3e3)),
                    PlainDword::new(speech_key(0x50a402)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a421)),
                    PlainDword::new(speech_key(0x50a441)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a461)),
                    PlainDword::new(speech_key(0x50a481)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a4a1)),
                    PlainDword::new(speech_key(0x50a4bf)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a4dd)),
                    PlainDword::new(speech_key(0x50a4fe)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a51f)),
                    PlainDword::new(speech_key(0x50a541)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a563)),
                    PlainDword::new(speech_key(0x50a583)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a5a3)),
                    PlainDword::new(speech_key(0x50a5c5)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a5e7)),
                    PlainDword::new(speech_key(0x50a609)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a62b)),
                    PlainDword::new(speech_key(0x50a64c)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a66d)),
                    PlainDword::new(speech_key(0x50a68c)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a6ab)),
                    PlainDword::new(speech_key(0x50a6ca)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50a6e9)),
                    PlainDword::new(speech_key(0x50a706)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
            ],
            stage_speech: [
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x509c71)),
                    PlainDword::new(speech_key(0x5097a9)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x509c94)),
                    PlainDword::new(speech_key(0x5097cb)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x509cb7)),
                    PlainDword::new(speech_key(0x5097ed)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x509cda)),
                    PlainDword::new(speech_key(0x50980f)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x509cfd)),
                    PlainDword::new(speech_key(0x509831)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x509d20)),
                    PlainDword::new(speech_key(0x509853)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x509d43)),
                    PlainDword::new(speech_key(0x509875)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x509d66)),
                    PlainDword::new(speech_key(0x509897)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x509d89)),
                    PlainDword::new(speech_key(0x5098b9)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x509dac)),
                    PlainDword::new(speech_key(0x5098db)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x509dcf)),
                    PlainDword::new(speech_key(0x5098fd)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x509df2)),
                    PlainDword::new(speech_key(0x50991f)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x509e15)),
                    PlainDword::new(speech_key(0x509941)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x509e38)),
                    PlainDword::new(speech_key(0x509963)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x509e5b)),
                    PlainDword::new(speech_key(0x509985)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x509e7e)),
                    PlainDword::new(speech_key(0x5099a7)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x509ea1)),
                    PlainDword::new(speech_key(0x5099c9)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x509ec4)),
                    PlainDword::new(speech_key(0x5099eb)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x509ee7)),
                    PlainDword::new(speech_key(0x509a0d)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x509f0a)),
                    PlainDword::new(speech_key(0x509a2f)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x509f2d)),
                    PlainDword::new(speech_key(0x509a51)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x509f50)),
                    PlainDword::new(speech_key(0x509a73)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x509f73)),
                    PlainDword::new(speech_key(0x509a95)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x509f96)),
                    PlainDword::new(speech_key(0x509ab7)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x509fb9)),
                    PlainDword::new(speech_key(0x509ad9)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x509fdc)),
                    PlainDword::new(speech_key(0x509afb)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x509fff)),
                    PlainDword::new(speech_key(0x509b1d)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x50a022)),
                    PlainDword::new(speech_key(0x509b3f)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x50a045)),
                    PlainDword::new(speech_key(0x509b61)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x50a068)),
                    PlainDword::new(speech_key(0x509b83)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50962f)),
                    PlainDword::new(speech_key(0x50a08b)),
                    PlainDword::new(speech_key(0x509ba5)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50964d)),
                    PlainDword::new(speech_key(0x50a0ae)),
                    PlainDword::new(speech_key(0x509bc7)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x50966b)),
                    PlainDword::new(speech_key(0x50a0d1)),
                    PlainDword::new(speech_key(0x509be9)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x509689)),
                    PlainDword::new(speech_key(0x50a0f4)),
                    PlainDword::new(speech_key(0x509c0b)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096a7)),
                    PlainDword::new(speech_key(0x50a117)),
                    PlainDword::new(speech_key(0x509c2d)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
                [
                    PlainDword::new(speech_key(0x5096c5)),
                    PlainDword::new(speech_key(0x50a13a)),
                    PlainDword::new(speech_key(0x509c4f)),
                    PlainDword::new(speech_key(0xffffffff)),
                ],
            ],
        }
    }
}

/// IDA 0x731B82, the first byte of the piece.
pub const START: usize = 0x731B82;
/// The bytes `MenuTexts` has before IDA's `START` (its `_lead`).
const LEAD: usize = 2;
/// The bytes `MenuTexts` has before the speech tables that the image does not.
const SPEECH_PAD: usize = 2;
/// IDA 0x734866, the first speech table.
pub const CAR_SPEECH: usize = 0x734866;
/// Where the first speech table is in `MenuTexts`.
pub const CAR_SPEECH_OFFSET: usize = CAR_SPEECH - START + LEAD + SPEECH_PAD;

/// A speech table's entry: a sample name's handle (`audio::store::name_key`; the
/// original kept its address), or the row's end mark, -1.
const fn speech_key(ida: u32) -> u32 {
    if ida == SPEECH_END {
        return ida;
    }
    crate::audio::store::name_key(ida)
}

/// A speech row's end mark.
pub const SPEECH_END: u32 = 0xFFFF_FFFF;

/// The entry at `place` of the speech row `row` (its `text_key`): the sample name's key
/// (`audio::store::name_key`), or `SPEECH_END`.
pub fn speech_entry(row: u32, place: u32) -> Result<u32, crate::errors::BadInput> {
    let entry = text_at(row)?
        .offset(place.wrapping_mul(4))
        .read::<u32>("a speech row")?;
    // 0: none, as the original's null.
    if entry == SPEECH_END || entry == 0 {
        return Ok(entry);
    }
    crate::audio::store::name_at(entry)?;
    Ok(entry)
}

/// A text's handle: its place in `MENU_TEXTS` (the original kept its address). 0
/// (none) and -1 (an end mark) stay; another address outside fails to compile.
pub const fn text_key(ida: u32) -> u32 {
    if ida == 0 || ida == 0xFFFF_FFFF {
        return ida;
    }
    let at = ida as usize;
    assert!(START <= at && at < END);
    let pad = if at >= CAR_SPEECH { SPEECH_PAD } else { 0 };
    (at - START + LEAD + pad) as u32
}

/// Where the text of `key` is (`text_key`).
pub fn text_at(key: u32) -> Result<crate::pointers::GamePtr, crate::errors::BadInput> {
    let at = key as usize;
    if !(LEAD..LEAD + SPEECH_PAD + END - START).contains(&at) {
        return Err(crate::errors::BadInput(
            "a text's handle off the menus' texts",
        ));
    }
    Ok(crate::pointers::GamePtr::to(&menus().tables.texts).offset(key))
}

/// IDA 0x734BC6, the end of the piece.
pub const END: usize = 0x734BC6;

#[cfg(test)]
const SPEECH_TABLES_END: usize = 0x734986 + 36 * 4 * 4;

#[cfg(test)]
pub fn menu_texts_addr() -> u32 {
    crate::pointers::game_address(&raw const menus().tables.texts) + 2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn each_speech_entry_names_the_samples_it_named_in_the_image() {
        let mut names = 0;
        for at in (CAR_SPEECH..SPEECH_TABLES_END).step_by(4) {
            let image = image_bytes(at, 4);
            let wanted = u32::from_le_bytes(image[..4].try_into().unwrap());
            let found = speech_entry(text_key(at as u32), 0).unwrap();
            if wanted == SPEECH_END || wanted == 0 {
                assert_eq!(found, wanted);
                continue;
            }
            names += 1;
            let text = crate::audio::store::name_at(found)
                .unwrap()
                .read_bytes(16, "a name")
                .unwrap();
            assert_eq!(text, image_bytes(wanted as usize, 16), "{at:#x}");
        }
        assert_eq!(names, 156);
    }

    #[test]
    fn menu_texts_start_as_the_games() {
        assert_eq!(size_of::<MenuTexts>(), END - START + LEAD + SPEECH_PAD);
        assert_eq!(offset_of!(MenuTexts, keys_menu), 0x731BBB - START + 2);
        assert_eq!(offset_of!(MenuTexts, car_speech), 0x734866 - START + 4);
        assert_eq!(offset_of!(MenuTexts, stage_speech), 0x734986 - START + 4);
        assert_eq!(menu_texts_addr() as usize % 4, START % 4);
        assert_eq!(offset_of!(MenuTexts, car_speech) % 4, 0);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(menu_texts_addr()).cast_const(),
                END - START + SPEECH_PAD,
            )
        };
        let mut image = image_bytes(START, END - START);
        for at in (CAR_SPEECH..SPEECH_TABLES_END).step_by(4) {
            let o = at - START;
            let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
            image[o..o + 4].copy_from_slice(&speech_key(v).to_le_bytes());
        }
        let split = CAR_SPEECH - START;
        assert_eq!(&mine[..split], &image[..split]);
        assert_eq!(&mine[split..split + SPEECH_PAD], &[0, 0]);
        assert_eq!(&mine[split + SPEECH_PAD..], &image[split..]);
    }
}
