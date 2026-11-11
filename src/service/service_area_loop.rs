//! `service::service_area_loop` (0x47284A): the service area screen, from entering it to
//! leaving it. The first of the game's 83 screen loops to be ported; the loop itself is
//! `menu::screen_loop`, and this file says what is the service area's own (`SCREEN`).
//!
//! It empties the key buffer (six `take_key`), then:
//!
//! Set-up (again after every action that does not leave the screen): the highlight comes
//! back from the screen's saved item (`g_serviceSavedItem`), the scene is prepared
//! (`scene::prepare_menu_scene`, a few of its window variables, `sound::play_tune_9`), the
//! screen's menu and its four function lists (the scene, draw, set-up and overlay lists)
//! are put in place (the set-up list is run once, here), the background is loaded
//! (`service::load_background`), one frame is drawn (`menu::draw_frame`, then
//! `screens::run_frame` with the callback `menu::frame_callback`), and the time is noted
//! (`session.setup_time`).
//!
//! Every frame: `menu::update_blink`; leave if the game is quitting; `screens::run_frame`;
//! the scene clock (`scene_clock`) = milliseconds since set-up; then `menu::handle_keys`:
//!   1  next frame
//!   2  (Escape) leave
//!   3  (Enter) the highlighted item's action runs (`menu::run_item_action`, with the
//!      highlight saved before and taken back after). Then: leave if `ask_leave` or
//!      `event_over` is set, or if `leave_screen` is set (which is cleared); else next frame if
//!      `action_mode` is 2; else leave if `event_left` is set (clearing `leave_screen`);
//!      else set up again.
//!   else leave
//! The same action check is made at the top of a frame whose `menu_sound` is 0x14D.
//!
//! Leaving: if `left_by_key` is set, `menu::confirm_leaving` asks whether to; unless the
//! answer (`g_yesNoAnswer`) is 1, the background is loaded again and the screen is set up
//! again. Otherwise the highlight is saved, `left_service` = 1, `sound::stop_music`,
//! `skip_edit_values` = 0.
//!
//! Wherever the highlight is saved or taken back, `escape_pressed` and `leave_screen` are
//! cleared if the event's `clears_escape_flag` and `escape_pressed` are both 1.
//!
//! Callers: `menu::time_trial_service` (time trial) and `menu::service_pass` (championship).

use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.service_area_scene.id(), // the scene list
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.service_area_draw.id(), // `handle_edit_keys`
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.service_area_setup.id(), // `load_setup_values`
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.service_area_overlay.id(), // `draw_service_area`
            run: false,
        },
    ],
    // The key buffer is emptied first.
    before: &[
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::Call(Callee::TakeKey),
        Step::WriteFlag(|m| &m.screens.state.menus.skip_edit_values, false),
    ],
    set_up_first: &[Step::Write(|m| &m.screens.flow.left_service, 0)],
    prepare: &[
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[0],
            0,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[1],
            0,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_zoom,
            0x100,
        ),
        Step::Call(Callee::PrepareScene),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.clip_edges[2],
            0,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.clip_edges[3],
            800,
        ),
        Step::WriteSceneF32(
            |_| &crate::scene::polygon_work::front_end().window.clip_screen[2],
            0.0,
        ),
        Step::WriteSceneF32(
            |_| &crate::scene::polygon_work::front_end().window.clip_screen[3],
            800.0,
        ),
        Step::Call(Callee::PlayTune9),
    ],
    background: &[Step::Call(Callee::LoadServiceBackground)],
    after_first_frame: &[Step::WriteFlag(
        |m| &m.screens.state.menus.skip_edit_values,
        true,
    )],
    after: &[
        Step::Write(|m| &m.screens.flow.left_service, 1),
        Step::Call(Callee::StopMusic),
        Step::WriteFlag(|m| &m.screens.state.menus.skip_edit_values, false),
    ],
    asking: Some(Asking {
        on_refusal: &[Step::Call(Callee::LoadServiceBackground)],
        ..Asking::DEFAULT
    }),
    // The saved item is `g_serviceSavedItem`.
    ..Screen::new(
        |m| m.tables.menus.service_area.id(),
        |m| &m.screens.items.current[slot::SERVICE_AREA],
    )
};
