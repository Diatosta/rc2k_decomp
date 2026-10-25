//! `service::service_area_loop` (0x47284A): the service area screen, from entering it to
//! leaving it. The first of the game's 83 screen loops to be ported; the loop itself is
//! `menu::screen_loop`, and this file says what is the service area's own (`SCREEN`).
//!
//! In:  nothing
//! Out: every register preserved
//! It empties the key buffer (six `take_key`), then:
//!
//! Set-up (again after every action that does not leave the screen): the highlight comes
//! back from `dword_74777C`, the scene is prepared (`sub_46F7FD`, a few of its variables,
//! `sub_40F7A4`), the screen's menu `off_74D24C` and its four function lists are put in place
//! (the list at 0x759436 is run once, here), the background is loaded (`sub_470F55`), one
//! frame is drawn (`menu::draw_frame`, then `screens::run_frame` with the callback
//! `sub_474D4B`), and the time is noted in `dword_5189A0`.
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
//! Leaving: if `dword_758806` is 1, `sub_46FA90` asks whether to; unless the answer
//! (`g_yesNoAnswer`) is 1, the background is loaded again and the screen is set up again.
//! Otherwise the highlight is saved, `dword_7554E4` = 1, `sub_40F826`, `dword_75AAE0` = 0.
//!
//! Wherever the highlight is saved or taken back, `dword_755A3C` and `dword_755561` are
//! cleared if `dword_7558D8` and `dword_755A3C` are both 1.
//!
//! The registers are carried through as the original has them, because the functions it
//! calls get them: `handle_keys` reads ah on Enter, which here is a byte of the elapsed time.
//!
//! Callers: `sub_48E093` (time trial) and `sub_4A15E3` (championship).
//! Seven of its callees are not ported and are called with `call_game`: the four named
//! under "game data" below, and `menu::draw_frame`, `screens::run_frame` and
//! `menu::confirm_leaving` in the shared loop.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::raw;
use crate::menu::screen_loop::{Callee, FRAME_CALLBACK, List, Screen, Step};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || MENUS.service_area.addr(), // off_74D24C
    saved_item: || &SCREEN_ITEMS.current[slot::SERVICE_AREA], // g_serviceSavedItem
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list,            // dword_755504
            table: || DRAW_LISTS.service_area_scene.addr(), // off_759082: the scene
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list,
            table: || DRAW_LISTS.service_area_draw.addr(), // off_758D6E: `handle_edit_keys`
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list,            // dword_755500
            table: || DRAW_LISTS.service_area_setup.addr(), // off_759436: `load_setup_values`
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // dword_7554FC
            table: || DRAW_LISTS.service_area_overlay.addr(), // off_75924A: `draw_service_area`
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    // The key buffer is emptied first.
    before: &[
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Arg(0),
        Step::Write(|| &MENU_STATE.menus.skip_edit_values, 0),
    ],
    set_up_first: &[Step::Write(|| &SCREEN_FLOW.left_service, 0)],
    prepare: &[
        Step::Write(|| &DRAW.view_offset[0], 0), // dword_623FB0
        Step::Write(|| &DRAW.view_offset[1], 0), // dword_623FB4
        Step::Write(|| &DRAW.view_zoom, 0x100),  // dword_624080
        Step::Call(Callee::PrepareScene),
        Step::Write(|| &DRAW.clip_edges[2], 0), // dword_6241DC
        Step::Write(|| &DRAW.clip_edges[3], 800), // dword_6241E0
        // DRAW.clip_screen[2]: `WriteF32` takes a `Dword` cell, not a `Float`.
        Step::WriteF32(|| raw(rva(0x6241EC)), 0.0), // flt_6241EC
        // DRAW.clip_screen[3]
        Step::WriteF32(|| raw(rva(0x6241F0)), 800.0), // flt_6241F0
        Step::Call(Callee::PlayTune9),
    ],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadServiceBackground)],
    after_first_frame: &[Step::Write(|| &MENU_STATE.menus.skip_edit_values, 1)],
    confirm: Some(&[Step::Call(Callee::LoadServiceBackground)]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[
        Step::Write(|| &SCREEN_FLOW.left_service, 1),
        Step::Call(Callee::StopMusic),
        Step::Write(|| &MENU_STATE.menus.skip_edit_values, 0),
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
