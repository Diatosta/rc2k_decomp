//! `menu::select_car_loop` (0x47AC0C): the car selection screen, which the starters of
//! every kind of event run (the owner called it the car select menu, 2026-10-12; it was
//! `car_specs_47ac0c_loop` until then).
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): background `cspecs.png`; leaving is not asked about; it has steps of its own
//! first on leaving.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, carousel, prepare_scene, quitting,
};
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    // The current player's +0x30 (its car).
    let current: u32 = crate::menu::owner::menus()
        .screens
        .event
        .current_player
        .get();
    let Some(player) = crate::menu::player::player(current) else {
        return Err(crate::errors::BadInput("a current player past the four").into());
    };
    crate::menu::owner::menus()
        .screens
        .flow
        .shown_gearing
        .set(player.get().gearing);
    crate::menu::owner::menus().screens.flow.turning.set(0x1u32);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .menu
        .set(0xDu32);
    carousel(entry.cars)?;
    let item = crate::menu::owner::menus().screens.items.current[slot::SELECT_CAR].get();
    crate::menu::owner::menus()
        .screens
        .loop_flags
        .item
        .set(item);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .front
        .set(item.wrapping_sub(1));
    crate::menu::owner::menus().screens.flow.turning.set(0x0u32);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .front
        .set(0x0u32);
    crate::menu::owner::menus()
        .screens
        .event
        .shown_car
        .set(0x0u32);
    crate::menu::car_choice::find_player_car()?;
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xFFFFFFCEu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene();
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

fn leaving(_: &Entry) -> Result<(), Refused> {
    if crate::menu::owner::menus()
        .tables
        .captions
        .screen_left
        .get()
        == 0x1
    {
        Callee::GoBack.run()?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || crate::menu::owner::menus().tables.menus.select_car.id(),
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::SELECT_CAR],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .select_car_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .select_car_draw
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.setup_list, // dword_755500
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .select_car_setup
                    .id()
            },
            run: true,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .select_car_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus().tables.texts.select_car_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
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
    leaving: &[Step::Do(leaving)],
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
