//! `menu::unused_cspecs_loop` (0x4AA1A3): a car specification screen with the carousel of
//! menu 0xC, background `cspecs.png`. Nothing in RAL.EXE reaches it; reached here from the
//! debug UI's "Screens" tab (`debug::screens`). What it was for is not known; a reading.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};
use crate::scene::owner::scene;

/// `unk_7476E4`'s slot, the item kept between visits.
const SLOT: usize = (0x7476E4 - 0x747660) / 4;

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    let screens = &menus().screens;
    screens.items.current[SLOT].set(screens.items.first[SLOT].get());
    Ok(())
}

// The original prepares the scene before the carousel, which left the front car white
// (seen 2026-10-30): here as `menu::select_car_loop`, the scene last.
fn set_up_first(_: &Entry) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    let screens = &menus().screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xCu32);
    carousel(None)?;
    screens
        .loop_flags
        .item
        .set(screens.items.current[SLOT].get());
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
    drawing.scene.view_offset[0].set(0x7u32);
    drawing.scene.view_offset[1].set(0xFFFFFFCEu32);
    drawing.scene.view_zoom.set(0x100u32);
    prepare_scene();
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.menu_74ba3c.id(),
    saved_item: |m| &m.screens.items.current[SLOT],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.list_758f6e.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.list_758c52.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.list_75941a.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.list_7591b2.id(),
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
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.cspecs_png
    }))],
    after_first_frame: &[],
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
