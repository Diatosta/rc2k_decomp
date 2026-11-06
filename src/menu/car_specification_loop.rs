//! `menu::car_specification_loop` (0x47A652): the screen titled "ESPECIFICAÇÕES DO CARRO".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; one call comes after the registers are put back; the steps that are more than a
//! call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESPECIFICAÇÕES DO CARRO", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    crate::menu::owner::menus()
        .screens
        .event
        .shown_car
        .set(crate::menu::owner::menus().play.cheats.kept_car.get());
    crate::menu::owner::menus()
        .screens
        .event
        .car_page
        .set(0x0u32);

    Ok(())
}

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    DRAW.view_offset[0].set(0xFFFFFF60u32);
    DRAW.view_offset[1].set(0xFFFFFF9Cu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene();
    crate::menu::owner::menus().screens.flow.turning.set(0x1u32);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .menu
        .set(0xBu32);
    carousel(None)?;
    let item = crate::menu::owner::menus().screens.items.current[slot::CAR_SPECIFICATION].get();
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
        .set(crate::menu::owner::menus().screens.event.shown_car.get());
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    crate::menu::owner::menus()
        .play
        .cheats
        .kept_car
        .set(crate::menu::owner::menus().screens.event.shown_car.get());

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .car_specification
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::CAR_SPECIFICATION],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .car_specification_scene
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
                    .car_specification_draw
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
                    .car_specification_setup
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
                    .car_specification_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Call(Callee::LoadCarBackground),
        Step::Call(Callee::PlayCarStream),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[Step::Call(Callee::StepCarStream)],
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
    after_restored: &[Step::Call(Callee::StopStream)],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::car_specification_loop", &SCREEN)
}
