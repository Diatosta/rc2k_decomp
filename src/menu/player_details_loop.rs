//! `menu::player_details_loop` (0x4773D6): the loop of the player details screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `pdetc.png`; leaving is not asked about; after
//! an action it is left or set up again, nothing else; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "PORMENORES DO JOGADOR", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0xFFFFFF64u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0x29u32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x140u32);
    prepare_scene(entry.scene);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x8u32);
    carousel(None)?;
    let item = screens.items.current[slot::PLAYER_DETAILS].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.player_details.id(),
    saved_item: |m| &m.screens.items.current[slot::PLAYER_DETAILS],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.player_details_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.player_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.player_details_overlay.id(),
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
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.player_details_picture
    }))],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: true,
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::player_details_loop", &SCREEN)
}
