//! `service::service_area_loop` (0x47284A): the service area screen, from entering it to
//! leaving it. The first of the game's 83 screen loops to be ported; the loop itself is
//! `menu::screen_loop`, and this file says what is the service area's own (`SCREEN`).
//!
//! It empties the key buffer (six `take_key`), then:
//!
//! Set-up (again after every action that does not leave the screen): the highlight comes
//! back from `dword_74777C`, the scene is prepared (`scene::prepare_menu_scene`, a few of its
//! variables,
//! `sound::play_tune_9`), the screen's menu `off_74D24C` and its four function lists are put in
//! place
//! (the list at 0x759436 is run once, here), the background is loaded (`service::load_background`),
//! one frame is drawn (`menu::draw_frame`, then `screens::run_frame` with the callback
//! `menu::frame_callback`), and the time is noted in `dword_5189A0`.
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! `dword_613830` = milliseconds since set-up; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved to `dword_74777C` before and taken back from it after). Then:
//!      leave if `dword_755569` or `dword_755980` is 1, or if `dword_755561` is 1 (which is
//!      cleared); else next frame if `dword_755A38` is 2; else leave if `dword_75555D` is 1
//!      (clearing `dword_755561`); else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `dword_7556A4` is 0x14D.
//!
//! Leaving: if `dword_758806` is 1, `menu::confirm_leaving` asks whether to; unless the answer
//! (`g_yesNoAnswer`) is 1, the background is loaded again and the screen is set up again.
//! Otherwise the highlight is saved, `dword_7554E4` = 1, `sound::stop_music`, `dword_75AAE0` = 0.
//!
//! Wherever the highlight is saved or taken back, `dword_755A3C` and `dword_755561` are
//! cleared if `dword_7558D8` and `dword_755A3C` are both 1.
//!
//! The registers are carried through as the original has them, because the functions it
//! calls get them: `handle_keys` reads ah on Enter, which here is a byte of the elapsed time.
//!
//! Callers: `menu::time_trial_service` (time trial) and `menu::service_pass` (championship). Seven
//! of its callees are not ported and are called by address: the four named under "game data" below,
//! and
//! `menu::draw_frame`, `screens::run_frame` and `menu::confirm_leaving` in the shared loop.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.service_area.id(), // off_74D24C
    saved_item: |m| &m.screens.items.current[slot::SERVICE_AREA], // g_serviceSavedItem
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // dword_755504
            table: |m| m.tables.draw_lists.service_area_scene.id(), // off_759082: the scene
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.service_area_draw.id(), // off_758D6E: `handle_edit_keys`
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.service_area_setup.id(), // off_759436: `load_setup_values`
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // dword_7554FC
            table: |m| m.tables.draw_lists.service_area_overlay.id(), // off_75924A: `draw_service_area`
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    // The key buffer is emptied first.
    before: &[
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Write(|m| &m.screens.state.menus.skip_edit_values, 0),
    ],
    set_up_first: &[Step::Write(|m| &m.screens.flow.left_service, 0)],
    prepare: &[
        Step::WriteScene(|s| &s.drawing.scene.view_offset[0], 0), // dword_623FB0
        Step::WriteScene(|s| &s.drawing.scene.view_offset[1], 0), // dword_623FB4
        Step::WriteScene(|s| &s.drawing.scene.view_zoom, 0x100),  // dword_624080
        Step::Call(Callee::PrepareScene),
        Step::WriteScene(|s| &s.drawing.scene.clip_edges[2], 0), // dword_6241DC
        Step::WriteScene(|s| &s.drawing.scene.clip_edges[3], 800), // dword_6241E0
        Step::WriteSceneF32(|s| &s.drawing.scene.clip_screen[2], 0.0), // flt_6241EC
        Step::WriteSceneF32(|s| &s.drawing.scene.clip_screen[3], 800.0), // flt_6241F0
        Step::Call(Callee::PlayTune9),
    ],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadServiceBackground)],
    after_first_frame: &[Step::Write(|m| &m.screens.state.menus.skip_edit_values, 1)],
    confirm: Some(&[Step::Call(Callee::LoadServiceBackground)]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[
        Step::Write(|m| &m.screens.flow.left_service, 1),
        Step::Call(Callee::StopMusic),
        Step::Write(|m| &m.screens.state.menus.skip_edit_values, 0),
    ],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
