//! The menus' texts: two log texts of the map's polygon code, each menu screen's picture path
//! (one copy per screen, named for the screen that loads it; the others by their file), the stage maps', car
//! specifications' and slides' paths (which `menu::lists_store`'s tables name by
//! `MenuText::key`), and two tables of speech samples' names (`CAR_SPEECH_ENTRIES`, `STAGE_SPEECH_ENTRIES`:
//! keys of `audio::names_store`'s names, `speech_entry`; read only). Each text's value is in the
//! `menu_texts_struct!` list under its field's name. `Menus::texts` holds them, apart from the menus' state:
//! the only writes are the picture loaders', which may swap a path's extension in place
//! (`files::swap_missing_extension`).

use crate::audio::names_store::{NameList, SampleName};
use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::with_runs;

/// Declares `MenuTexts` over its texts, each a field named for the text and holding its bytes
/// and a NUL; `MenuTexts::new` fills each one from its literal.
macro_rules! menu_texts_struct {
    ($($(#[$doc:meta])* $field:ident: $text:literal,)*) => {
        pub struct MenuTexts {
            $($(#[$doc])* pub $field: PlainBuffer<{ $text.len() + 1 }>,)*
        }

        impl MenuTexts {
            pub const fn new() -> Self {
                MenuTexts {
                    $($field: PlainBuffer::new(with_runs::<{ $text.len() + 1 }>(&[(0, $text)])),)*
                }
            }
        }
    };
}

menu_texts_struct! {
    no_point_text: b"No point in poly",
    no_second_point_text: b"While scanning right didnt find point 2",
    loading_pcx: b"files\\g\\graphics\\screens\\loading.pcx",
    leave_question_picture: b"files\\g\\graphics\\screens\\mainscr.png",
    mob_welc_png: b"files\\g\\graphics\\screens\\mob_welc.png",
    a8_welc_png: b"files\\g\\graphics\\screens\\a8_welc.png",
    titlebar_png: b"files\\g\\graphics\\screens\\titlebar.png",
    main_menu_picture: b"files\\g\\graphics\\screens\\mainscr.png",
    start_rally_picture: b"files\\g\\graphics\\screens\\sral.png",
    network_rally_picture: b"files\\g\\graphics\\screens\\nral.png",
    player1_details_picture: b"files\\g\\graphics\\screens\\pdet1.png",
    player2_details_picture: b"files\\g\\graphics\\screens\\pdet1.png",
    player3_details_picture: b"files\\g\\graphics\\screens\\pdet1.png",
    player4_details_picture: b"files\\g\\graphics\\screens\\pdet1.png",
    player_details_picture: b"files\\g\\graphics\\screens\\pdetc.png",
    options_picture: b"files\\g\\graphics\\screens\\opts.png",
    information_picture: b"files\\g\\graphics\\screens\\info.png",
    video_png: b"files\\g\\graphics\\screens\\video.png",
    visual_preferences_picture: b"files\\g\\graphics\\screens\\visual.png",
    visual_options_picture: b"files\\g\\graphics\\screens\\visual.png",
    cspecs_png: b"files\\g\\graphics\\screens\\cspecs.png",
    cspecs_png_2: b"files\\g\\graphics\\screens\\cspecs.png",
    load_championship_picture: b"files\\g\\graphics\\screens\\loadsv.png",
    save_championship_picture: b"files\\g\\graphics\\screens\\loadsv.png",
    save_replay_picture: b"files\\g\\graphics\\screens\\loadsv.png",
    load_replay_picture: b"files\\g\\graphics\\screens\\loadsv.png",
    select_car_picture: b"files\\g\\graphics\\screens\\cspecs.png",
    car_specs_display_picture: b"files\\g\\graphics\\screens\\cspecs.png",
    cspecs_png_3: b"files\\g\\graphics\\screens\\cspecs.png",
    view_best_times_picture: b"files\\g\\graphics\\screens\\btimes.png",
    view_best_scores_picture: b"files\\g\\graphics\\screens\\bascores.png",
    best_times_picture: b"files\\g\\graphics\\screens\\btimes.png",
    arcade_best_times_picture: b"files\\g\\graphics\\screens\\btimes.png",
    time_trial_best_times_picture: b"files\\g\\graphics\\screens\\btimes.png",
    arcade_best_scores_picture: b"files\\g\\graphics\\screens\\bascores.png",
    best_class_scores_picture: b"files\\g\\graphics\\screens\\bascores.png",
    best_overall_scores_picture: b"files\\g\\graphics\\screens\\bascores.png",
    audio_setup_picture: b"files\\g\\graphics\\screens\\audio.png",
    advanced_audio_picture: b"files\\g\\graphics\\screens\\audio.png",
    vcourses_png: b"files\\g\\graphics\\screens\\vcourses.png",
    vcourses_png_2: b"files\\g\\graphics\\screens\\vcourses.png",
    vcourses_png_3: b"files\\g\\graphics\\screens\\vcourses.png",
    vcourses_png_4: b"files\\g\\graphics\\screens\\vcourses.png",
    ukmap_png: b"files\\g\\graphics\\screens\\ukmap.png",
    loading_png: b"files\\g\\graphics\\screens\\loading.png",
    player1_controller_picture: b"files\\g\\graphics\\screens\\control.png",
    player1_buttons_picture: b"files\\g\\graphics\\screens\\control.png",
    player2_controller_picture: b"files\\g\\graphics\\screens\\control.png",
    player2_buttons_picture: b"files\\g\\graphics\\screens\\control.png",
    player3_controller_picture: b"files\\g\\graphics\\screens\\control.png",
    player3_buttons_picture: b"files\\g\\graphics\\screens\\control.png",
    player4_controller_picture: b"files\\g\\graphics\\screens\\control.png",
    player4_buttons_picture: b"files\\g\\graphics\\screens\\control.png",
    redefine_keys_player1_picture: b"files\\g\\graphics\\screens\\defkeys.png",
    redefine_keys_player2_picture: b"files\\g\\graphics\\screens\\defkeys.png",
    redefine_keys_player3_picture: b"files\\g\\graphics\\screens\\defkeys.png",
    redefine_keys_player4_picture: b"files\\g\\graphics\\screens\\defkeys.png",
    sral_png: b"files\\g\\graphics\\screens\\sral.png",
    sral_png_2: b"files\\g\\graphics\\screens\\sral.png",
    championship_new_simulation_picture: b"files\\g\\graphics\\screens\\sral.png",
    new_simulation_picture: b"files\\g\\graphics\\screens\\sral.png",
    endday_png: b"files\\g\\graphics\\screens\\endday.png",
    endday_png_2: b"files\\g\\graphics\\screens\\endday.png",
    rally_stages_picture: b"files\\g\\graphics\\screens\\ukmap.png",
    serv00_png: b"files\\g\\graphics\\screens\\serv00.png",
    serv00_png_2: b"files\\g\\graphics\\screens\\serv00.png",
    serv00_png_3: b"files\\g\\graphics\\screens\\serv00.png",
    class_stage_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    overall_stage_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    class_rally_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    overall_rally_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    results1_png: b"files\\g\\graphics\\screens\\results1.png",
    results1_png_2: b"files\\g\\graphics\\screens\\results1.png",
    /// The copy `menu::section_times_loop` loads as its background.
    section_times_png: b"files\\g\\graphics\\screens\\results1.png",
    inspection_area_picture: b"files\\g\\graphics\\screens\\service.png",
    championship_service_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    championship_standings_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    championship_races_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    championship_end_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    in_game_options_picture: b"files\\g\\graphics\\screens\\opts.png",
    welcomer_png: b"files\\g\\graphics\\screens\\welcomer.png",
    welvx_png: b"files\\g\\graphics\\screens\\welvx.png",
    welpir_png: b"files\\g\\graphics\\screens\\welpir.png",
    welsct_png: b"files\\g\\graphics\\screens\\welsct.png",
    weljc_png: b"files\\g\\graphics\\screens\\weljc.png",
    welul_png: b"files\\g\\graphics\\screens\\welul.png",
    welmnx_png: b"files\\g\\graphics\\screens\\welmnx.png",
    a8_1_png: b"files\\g\\graphics\\screens\\a8_1.png",
    a8_2_png: b"files\\g\\graphics\\screens\\a8_2.png",
    a8_3_png: b"files\\g\\graphics\\screens\\a8_3.png",
    a8_4_png: b"files\\g\\graphics\\screens\\a8_4.png",
    a8_5_png: b"files\\g\\graphics\\screens\\a8_5.png",
    a8_6_png: b"files\\g\\graphics\\screens\\a8_6.png",
    setcar_png: b"files\\g\\graphics\\screens\\setcar.png",
    class_championship_standings_picture: b"files\\g\\graphics\\screens\\champoc.png",
    championship_standings_picture: b"files\\g\\graphics\\screens\\champoc.png",
    simple_select_players_picture: b"files\\g\\graphics\\screens\\arcade1.png",
    single_rally_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    arcade_review_options_picture: b"files\\g\\graphics\\screens\\roptions.png",
    time_trial_select_players_picture: b"files\\g\\graphics\\screens\\arcade1.png",
    arcade_next_level_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    arcade_retry_review_picture: b"files\\g\\graphics\\screens\\roptions.png",
    roptions_png: b"files\\g\\graphics\\screens\\roptions.png",
    arcade_level_end_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    stage_result_picture: b"files\\g\\graphics\\screens\\results1.png",
    results1_png_4: b"files\\g\\graphics\\screens\\results1.png",
    single_rally_two_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    single_rally_three_four_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    time_trial_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    single_rally_overall_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    results1_png_5: b"files\\g\\graphics\\screens\\results1.png",
    selstage_png: b"files\\g\\graphics\\screens\\selstage.png",
    selstage_png_2: b"files\\g\\graphics\\screens\\selstage.png",
    arcstage_png: b"files\\g\\graphics\\screens\\arcstage.png",
    arcade1_png: b"files\\g\\graphics\\screens\\arcade1.png",
    arcade_select_level_picture: b"files\\g\\graphics\\screens\\Selarc.png",
    arcade_results_picture: b"files\\g\\graphics\\screens\\results1.png",
    roptions_png_2: b"files\\g\\graphics\\screens\\roptions.png",
    racing_png: b"files\\g\\graphics\\screens\\racing.png",
    racing_png_2: b"files\\g\\graphics\\screens\\racing.png",
    mbuts00_pcx: b"files\\g\\graphics\\sprites\\mbuts00.pcx",
    serv00_png_4: b"files\\g\\graphics\\screens\\serv00.png",
    serv01_png: b"files\\g\\graphics\\screens\\serv01.png",
    serv02_png: b"files\\g\\graphics\\screens\\serv02.png",
    serv03_png: b"files\\g\\graphics\\screens\\serv03.png",
    serv04_png: b"files\\g\\graphics\\screens\\serv04.png",
    serv05_png: b"files\\g\\graphics\\screens\\serv05.png",
    serv06_png: b"files\\g\\graphics\\screens\\serv06.png",
    serv07_png: b"files\\g\\graphics\\screens\\serv07.png",
    serv08_png: b"files\\g\\graphics\\screens\\serv08.png",
    serv09_png: b"files\\g\\graphics\\screens\\serv09.png",
    serv10_png: b"files\\g\\graphics\\screens\\serv10.png",
    serv11_png: b"files\\g\\graphics\\screens\\serv11.png",
    serv12_png: b"files\\g\\graphics\\screens\\serv12.png",
    serv13_png: b"files\\g\\graphics\\screens\\serv13.png",
    serv14_png: b"files\\g\\graphics\\screens\\serv14.png",
    serv15_png: b"files\\g\\graphics\\screens\\serv15.png",
    serv16_png: b"files\\g\\graphics\\screens\\serv16.png",
    serv17_png: b"files\\g\\graphics\\screens\\serv17.png",
    serv18_png: b"files\\g\\graphics\\screens\\serv18.png",
    serv19_png: b"files\\g\\graphics\\screens\\serv19.png",
    serv20_png: b"files\\g\\graphics\\screens\\serv20.png",
    serv21_png: b"files\\g\\graphics\\screens\\serv21.png",
    serv22_png: b"files\\g\\graphics\\screens\\serv22.png",
    serv33_png: b"files\\g\\graphics\\screens\\serv33.png",
    vrw_001_png: b"files\\g\\graphics\\screens\\maps\\vrw_001.png",
    vrw_002_png: b"files\\g\\graphics\\screens\\maps\\vrw_002.png",
    vrw_003_png: b"files\\g\\graphics\\screens\\maps\\vrw_003.png",
    vrw_004_png: b"files\\g\\graphics\\screens\\maps\\vrw_004.png",
    vrw_005_png: b"files\\g\\graphics\\screens\\maps\\vrw_005.png",
    vrw_006_png: b"files\\g\\graphics\\screens\\maps\\vrw_006.png",
    pir_001_png: b"files\\g\\graphics\\screens\\maps\\pir_001.png",
    pir_002_png: b"files\\g\\graphics\\screens\\maps\\pir_002.png",
    pir_003_png: b"files\\g\\graphics\\screens\\maps\\pir_003.png",
    pir_004_png: b"files\\g\\graphics\\screens\\maps\\pir_004.png",
    pir_005_png: b"files\\g\\graphics\\screens\\maps\\pir_005.png",
    pir_006_png: b"files\\g\\graphics\\screens\\maps\\pir_006.png",
    sc_001_png: b"files\\g\\graphics\\screens\\maps\\sc_001.png",
    sc_002_png: b"files\\g\\graphics\\screens\\maps\\sc_002.png",
    sc_003_png: b"files\\g\\graphics\\screens\\maps\\sc_003.png",
    sc_004_png: b"files\\g\\graphics\\screens\\maps\\sc_004.png",
    sc_005_png: b"files\\g\\graphics\\screens\\maps\\sc_005.png",
    sc_006_png: b"files\\g\\graphics\\screens\\maps\\sc_006.png",
    jc_001_png: b"files\\g\\graphics\\screens\\maps\\jc_001.png",
    jc_002_png: b"files\\g\\graphics\\screens\\maps\\jc_002.png",
    jc_003_png: b"files\\g\\graphics\\screens\\maps\\jc_003.png",
    jc_004_png: b"files\\g\\graphics\\screens\\maps\\jc_004.png",
    jc_005_png: b"files\\g\\graphics\\screens\\maps\\jc_005.png",
    jc_006_png: b"files\\g\\graphics\\screens\\maps\\jc_006.png",
    ul_001_png: b"files\\g\\graphics\\screens\\maps\\ul_001.png",
    ul_002_png: b"files\\g\\graphics\\screens\\maps\\ul_002.png",
    ul_003_png: b"files\\g\\graphics\\screens\\maps\\ul_003.png",
    ul_004_png: b"files\\g\\graphics\\screens\\maps\\ul_004.png",
    ul_005_png: b"files\\g\\graphics\\screens\\maps\\ul_005.png",
    ul_006_png: b"files\\g\\graphics\\screens\\maps\\ul_006.png",
    mnx_001_png: b"files\\g\\graphics\\screens\\maps\\mnx_001.png",
    mnx_002_png: b"files\\g\\graphics\\screens\\maps\\mnx_002.png",
    mnx_003_png: b"files\\g\\graphics\\screens\\maps\\mnx_003.png",
    mnx_004_png: b"files\\g\\graphics\\screens\\maps\\mnx_004.png",
    mnx_005_png: b"files\\g\\graphics\\screens\\maps\\mnx_005.png",
    mnx_006_png: b"files\\g\\graphics\\screens\\maps\\mnx_006.png",
    mgnvc_png: b"files\\g\\graphics\\screens\\cspecs\\mgnvc.png",
    vxavc_png: b"files\\g\\graphics\\screens\\cspecs\\vxavc.png",
    stcvc_png: b"files\\g\\graphics\\screens\\cspecs\\stcvc.png",
    hycvc_png: b"files\\g\\graphics\\screens\\cspecs\\hycvc.png",
    sbivc_png: b"files\\g\\graphics\\screens\\cspecs\\sbivc.png",
    pgovc_png: b"files\\g\\graphics\\screens\\cspecs\\pgovc.png",
    mxpvc_png: b"files\\g\\graphics\\screens\\cspecs\\mxpvc.png",
    mtlvc_png: b"files\\g\\graphics\\screens\\cspecs\\mtlvc.png",
    vwgvc_png: b"files\\g\\graphics\\screens\\cspecs\\vwgvc.png",
    nsavc_png: b"files\\g\\graphics\\screens\\cspecs\\nsavc.png",
    ptwvc_png: b"files\\g\\graphics\\screens\\cspecs\\ptwvc.png",
    ctsvc_png: b"files\\g\\graphics\\screens\\cspecs\\ctsvc.png",
    hncvc_png: b"files\\g\\graphics\\screens\\cspecs\\hncvc.png",
    nsmvc_png: b"files\\g\\graphics\\screens\\cspecs\\nsmvc.png",
    pgtvc_png: b"files\\g\\graphics\\screens\\cspecs\\pgtvc.png",
    frtvc_png: b"files\\g\\graphics\\screens\\cspecs\\frtvc.png",
    sdovc_png: b"files\\g\\graphics\\screens\\cspecs\\sdovc.png",
    stivc_png: b"files\\g\\graphics\\screens\\cspecs\\stivc.png",
    sdfvc_png: b"files\\g\\graphics\\screens\\cspecs\\sdfvc.png",
    ptcvc_png: b"files\\g\\graphics\\screens\\cspecs\\ptcvc.png",
    fmxvc_png: b"files\\g\\graphics\\screens\\cspecs\\fmxvc.png",
    fpmvc_png: b"files\\g\\graphics\\screens\\cspecs\\fpmvc.png",
    oscvc_png: b"files\\g\\graphics\\screens\\cspecs\\oscvc.png",
    rclvc_png: b"files\\g\\graphics\\screens\\cspecs\\rclvc.png",
    mgndm_png: b"files\\g\\graphics\\screens\\cspecs\\mgndm.png",
    vxadm_png: b"files\\g\\graphics\\screens\\cspecs\\vxadm.png",
    stcdm_png: b"files\\g\\graphics\\screens\\cspecs\\stcdm.png",
    hycdm_png: b"files\\g\\graphics\\screens\\cspecs\\hycdm.png",
    sbidm_png: b"files\\g\\graphics\\screens\\cspecs\\sbidm.png",
    pgodm_png: b"files\\g\\graphics\\screens\\cspecs\\pgodm.png",
    mxpdm_png: b"files\\g\\graphics\\screens\\cspecs\\mxpdm.png",
    mtldm_png: b"files\\g\\graphics\\screens\\cspecs\\mtldm.png",
    vwgdm_png: b"files\\g\\graphics\\screens\\cspecs\\vwgdm.png",
    nsadm_png: b"files\\g\\graphics\\screens\\cspecs\\nsadm.png",
    ptwdm_png: b"files\\g\\graphics\\screens\\cspecs\\ptwdm.png",
    ctsdm_png: b"files\\g\\graphics\\screens\\cspecs\\ctsdm.png",
    hncdm_png: b"files\\g\\graphics\\screens\\cspecs\\hncdm.png",
    nsmdm_png: b"files\\g\\graphics\\screens\\cspecs\\nsmdm.png",
    pgtdm_png: b"files\\g\\graphics\\screens\\cspecs\\pgtdm.png",
    frtdm_png: b"files\\g\\graphics\\screens\\cspecs\\frtdm.png",
    sdodm_png: b"files\\g\\graphics\\screens\\cspecs\\sdodm.png",
    stidm_png: b"files\\g\\graphics\\screens\\cspecs\\stidm.png",
    sdfdm_png: b"files\\g\\graphics\\screens\\cspecs\\sdfdm.png",
    ptcdm_png: b"files\\g\\graphics\\screens\\cspecs\\ptcdm.png",
    fmxdm_png: b"files\\g\\graphics\\screens\\cspecs\\fmxdm.png",
    fpmdm_png: b"files\\g\\graphics\\screens\\cspecs\\fpmdm.png",
    oscdm_png: b"files\\g\\graphics\\screens\\cspecs\\oscdm.png",
    rcldm_png: b"files\\g\\graphics\\screens\\cspecs\\rcldm.png",
    mgnen_png: b"files\\g\\graphics\\screens\\cspecs\\mgnen.png",
    vxaen_png: b"files\\g\\graphics\\screens\\cspecs\\vxaen.png",
    stcen_png: b"files\\g\\graphics\\screens\\cspecs\\stcen.png",
    hycen_png: b"files\\g\\graphics\\screens\\cspecs\\hycen.png",
    sbien_png: b"files\\g\\graphics\\screens\\cspecs\\sbien.png",
    pgoen_png: b"files\\g\\graphics\\screens\\cspecs\\pgoen.png",
    mxpen_png: b"files\\g\\graphics\\screens\\cspecs\\mxpen.png",
    mtlen_png: b"files\\g\\graphics\\screens\\cspecs\\mtlen.png",
    vwgen_png: b"files\\g\\graphics\\screens\\cspecs\\vwgen.png",
    nsaen_png: b"files\\g\\graphics\\screens\\cspecs\\nsaen.png",
    ptwen_png: b"files\\g\\graphics\\screens\\cspecs\\ptwen.png",
    ctsen_png: b"files\\g\\graphics\\screens\\cspecs\\ctsen.png",
    hncen_png: b"files\\g\\graphics\\screens\\cspecs\\hncen.png",
    nsmen_png: b"files\\g\\graphics\\screens\\cspecs\\nsmen.png",
    pgten_png: b"files\\g\\graphics\\screens\\cspecs\\pgten.png",
    frten_png: b"files\\g\\graphics\\screens\\cspecs\\frten.png",
    sdoen_png: b"files\\g\\graphics\\screens\\cspecs\\sdoen.png",
    stien_png: b"files\\g\\graphics\\screens\\cspecs\\stien.png",
    sdfen_png: b"files\\g\\graphics\\screens\\cspecs\\sdfen.png",
    ptcen_png: b"files\\g\\graphics\\screens\\cspecs\\ptcen.png",
    fmxen_png: b"files\\g\\graphics\\screens\\cspecs\\fmxen.png",
    fpmen_png: b"files\\g\\graphics\\screens\\cspecs\\fpmen.png",
    oscen_png: b"files\\g\\graphics\\screens\\cspecs\\oscen.png",
    rclen_png: b"files\\g\\graphics\\screens\\cspecs\\rclen.png",
    mgntc_png: b"files\\g\\graphics\\screens\\cspecs\\mgntc.png",
    vxatc_png: b"files\\g\\graphics\\screens\\cspecs\\vxatc.png",
    stctc_png: b"files\\g\\graphics\\screens\\cspecs\\stctc.png",
    hyctc_png: b"files\\g\\graphics\\screens\\cspecs\\hyctc.png",
    sbitc_png: b"files\\g\\graphics\\screens\\cspecs\\sbitc.png",
    pgotc_png: b"files\\g\\graphics\\screens\\cspecs\\pgotc.png",
    mxptc_png: b"files\\g\\graphics\\screens\\cspecs\\mxptc.png",
    mtltc_png: b"files\\g\\graphics\\screens\\cspecs\\mtltc.png",
    vwgtc_png: b"files\\g\\graphics\\screens\\cspecs\\vwgtc.png",
    nsatc_png: b"files\\g\\graphics\\screens\\cspecs\\nsatc.png",
    ptwtc_png: b"files\\g\\graphics\\screens\\cspecs\\ptwtc.png",
    ctstc_png: b"files\\g\\graphics\\screens\\cspecs\\ctstc.png",
    hnctc_png: b"files\\g\\graphics\\screens\\cspecs\\hnctc.png",
    nsmtc_png: b"files\\g\\graphics\\screens\\cspecs\\nsmtc.png",
    pgttc_png: b"files\\g\\graphics\\screens\\cspecs\\pgttc.png",
    frttc_png: b"files\\g\\graphics\\screens\\cspecs\\frttc.png",
    sdotc_png: b"files\\g\\graphics\\screens\\cspecs\\sdotc.png",
    stitc_png: b"files\\g\\graphics\\screens\\cspecs\\stitc.png",
    sdftc_png: b"files\\g\\graphics\\screens\\cspecs\\sdftc.png",
    ptctc_png: b"files\\g\\graphics\\screens\\cspecs\\ptctc.png",
    fmxtc_png: b"files\\g\\graphics\\screens\\cspecs\\fmxtc.png",
    fpmtc_png: b"files\\g\\graphics\\screens\\cspecs\\fpmtc.png",
    osctc_png: b"files\\g\\graphics\\screens\\cspecs\\osctc.png",
    rcltc_png: b"files\\g\\graphics\\screens\\cspecs\\rcltc.png",
    mgnsl00_png: b";3files\\g\\slides\\mgnsl00.png",
    vxasl00_png: b";3files\\g\\slides\\vxasl00.png",
    stcsl00_png: b";3files\\g\\slides\\stcsl00.png",
    hycsl00_png: b";3files\\g\\slides\\hycsl00.png",
    sbisl00_png: b";3files\\g\\slides\\sbisl00.png",
    pgosl00_png: b";3files\\g\\slides\\pgosl00.png",
    mxpsl00_png: b";3files\\g\\slides\\mxpsl00.png",
    mtlsl00_png: b";3files\\g\\slides\\mtlsl00.png",
    vwgsl00_png: b";3files\\g\\slides\\vwgsl00.png",
    nsasl00_png: b";3files\\g\\slides\\nsasl00.png",
    ptwsl00_png: b";3files\\g\\slides\\ptwsl00.png",
    ctssl00_png: b";3files\\g\\slides\\ctssl00.png",
    hncsl00_png: b";3files\\g\\slides\\hncsl00.png",
    nsmsl00_png: b";3files\\g\\slides\\nsmsl00.png",
    pgtsl00_png: b";3files\\g\\slides\\pgtsl00.png",
    frtsl00_png: b";3files\\g\\slides\\frtsl00.png",
    sdosl00_png: b";3files\\g\\slides\\sdosl00.png",
    stisl00_png: b";3files\\g\\slides\\stisl00.png",
    sdfsl00_png: b";3files\\g\\slides\\sdfsl00.png",
    ptcsl00_png: b";3files\\g\\slides\\ptcsl00.png",
    fmxsl00_png: b";3files\\g\\slides\\fmxsl00.png",
    fpmsl00_png: b";3files\\g\\slides\\fpmsl00.png",
    oscsl00_png: b";3files\\g\\slides\\oscsl00.png",
    rclsl00_png: b";3files\\g\\slides\\rclsl00.png",
}

/// Each car's two menu speech samples (indexes into the menu speech names); a row ends with `SPEECH_END`.
const CAR_SPEECH: [[u16; 2]; CAR_ROWS] = [
    [84, 85],
    [86, 87],
    [88, 89],
    [90, 91],
    [92, 93],
    [94, 95],
    [104, 105],
    [96, 97],
    [98, 99],
    [100, 101],
    [102, 103],
    [104, 105],
    [106, 107],
    [108, 109],
    [110, 111],
    [112, 113],
    [114, 115],
    [116, 117],
    [118, 119],
    [120, 121],
    [122, 123],
    [124, 125],
    [126, 127],
    [128, 129],
];

/// Each stage's three menu speech samples (indexes into the menu speech names); a row ends with `SPEECH_END`.
const STAGE_SPEECH: [[u16; 3]; STAGE_ROWS] = [
    [0, 48, 12],
    [1, 49, 13],
    [2, 50, 14],
    [3, 51, 15],
    [4, 52, 16],
    [5, 53, 17],
    [0, 54, 18],
    [1, 55, 19],
    [2, 56, 20],
    [3, 57, 21],
    [4, 58, 22],
    [5, 59, 23],
    [0, 60, 24],
    [1, 61, 25],
    [2, 62, 26],
    [3, 63, 27],
    [4, 64, 28],
    [5, 65, 29],
    [0, 66, 30],
    [1, 67, 31],
    [2, 68, 32],
    [3, 69, 33],
    [4, 70, 34],
    [5, 71, 35],
    [0, 72, 36],
    [1, 73, 37],
    [2, 74, 38],
    [3, 75, 39],
    [4, 76, 40],
    [5, 77, 41],
    [0, 78, 42],
    [1, 79, 43],
    [2, 80, 44],
    [3, 81, 45],
    [4, 82, 46],
    [5, 83, 47],
];

/// A speech table's rows: each row's samples, then the end mark.
const fn speech_rows<const ROWS: usize, const SAMPLES: usize, const ENTRIES: usize>(
    samples: &[[u16; SAMPLES]; ROWS],
) -> [[u32; ENTRIES]; ROWS] {
    const { assert!(ENTRIES == SAMPLES + 1) };
    let mut rows = [[SPEECH_END; ENTRIES]; ROWS];
    let mut row = 0;
    while row < ROWS {
        let mut sample = 0;
        while sample < SAMPLES {
            rows[row][sample] = speech(NameList::MenuSpeech, samples[row][sample]);
            sample += 1;
        }
        row += 1;
    }
    rows
}

/// Per car, its two speech samples' names and the end mark.
const CAR_SPEECH_ENTRIES: [[u32; 3]; CAR_ROWS] = speech_rows(&CAR_SPEECH);
/// Per stage, three speech samples' names and the end mark.
const STAGE_SPEECH_ENTRIES: [[u32; 4]; STAGE_ROWS] = speech_rows(&STAGE_SPEECH);

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
fn speech_dword(n: usize) -> Option<u32> {
    if n < CAR_ENTRIES {
        Some(CAR_SPEECH_ENTRIES[n / 3][n % 3])
    } else if n < CAR_ENTRIES + STAGE_ENTRIES {
        let n = n - CAR_ENTRIES;
        Some(STAGE_SPEECH_ENTRIES[n / 4][n % 4])
    } else {
        None
    }
}

/// The entry at `place` of the speech row `row` (its key): the sample name's key
/// (`SampleName::key`), or `SPEECH_END`.
pub fn speech_entry(
    names: &crate::audio::names_store::SampleNames,
    row: u32,
    place: u32,
) -> Result<u32, crate::errors::BadInput> {
    let first = row
        .checked_sub(SPEECH_KEY_BASE)
        .ok_or(crate::errors::BadInput(
            "a speech row's key off the speech rows",
        ))?;
    let entry = speech_dword(first as usize + place as usize).ok_or(crate::errors::BadInput(
        "a speech entry off the speech rows",
    ))?;
    // 0: none.
    if entry == SPEECH_END || entry == 0 {
        return Ok(entry);
    }
    crate::audio::sample_records::name_in(names, entry)?;
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

    /// The speech tables' entries as the 1999 image addressed them (car rows then stage rows).
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
        let mut now: Vec<&u32> = Vec::new();
        for row in &CAR_SPEECH_ENTRIES {
            now.extend(row);
        }
        for row in &STAGE_SPEECH_ENTRIES {
            now.extend(row);
        }
        assert_eq!(now.len(), SPEECH_ADDRESSES.len());
        for (n, (key, address)) in now.iter().zip(SPEECH_ADDRESSES).enumerate() {
            let expected = if address == 0xffff_ffff {
                address
            } else {
                address - 0x506E6C + crate::audio::names_store::FIRST_KEY
            };
            assert_eq!(**key, expected, "entry {n}");
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
        assert_eq!(speech_dword(CAR_ENTRIES + STAGE_ENTRIES), None);
    }

    /// Every text's bytes are the ones the image held (a checksum of all of them, each with
    /// its NUL).
    #[test]
    fn the_texts_keep_the_bytes_of_the_image() {
        let texts = &crate::test_support::own::menus().texts;
        let mut bytes = Vec::new();
        for text in MenuText::ALL {
            let at = text_at(texts, text.key()).unwrap();
            bytes.extend(at.text("a test text").unwrap());
            bytes.push(0);
        }
        assert_eq!(bytes.len(), 7503);
        assert_eq!(crate::test_support::hash(&bytes), 12055103207217156549);
    }
}
