//! The service area screen, from entering it to leaving it; the loop itself is
//! `menu::screen_loop`, and this file says what is the service area's own (`SCREEN`).
//!
//! It empties the key buffer, then sets up (again after every action that does not leave the
//! screen): the highlight comes back from the screen's saved item, the scene is prepared, the
//! screen's menu and its four function lists are put in place (the set-up list is run once),
//! the background is loaded and one frame drawn. Every frame: `menu::update_blink`, leave if
//! the game is quitting, `screens::run_frame`, then `menu::handle_keys`: 1 next frame, 2
//! (Escape) leave, 3 (Enter) the highlighted item's action runs (`menu::run_item_action`) and
//! then, by `ask_leave`, `event_over`, `leave_screen`, `action_mode` and `event_left`, the
//! screen is left, the next frame runs or it is set up again; anything else leaves. The same
//! action check is made at the top of a frame whose `menu_sound` is 0x14D. Wherever the
//! highlight is saved or taken back, `escape_pressed` and `leave_screen` are cleared if the
//! event's `clears_escape_flag` and `escape_pressed` are both 1.
//!
//! Leaving: if `left_by_key` is set, `menu::confirm_leaving` asks whether to; unless the answer
//! is 1, the background is loaded again and the screen is set up again. Otherwise the highlight
//! is saved, `left_service` = 1, the music stops and `skip_edit_values` = 0.
//!
//! Called by `menu::time_trial_service` (time trial) and `menu::service_pass` (championship).

use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

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
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.service_area.id(),
        |m| &m.screens.items.current[slot::SERVICE_AREA],
    )
};
