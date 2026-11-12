//! `menu::options_loop` (0x477106): the loop of the options menu.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `opts.png`; leaving is not asked about; after
//! an action it is left or set up again, nothing else; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    crate::strings::units::set_units(menus.play.players.players[0].get().metric_units); // g_isImperialUnits
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry.scene);
    menus.screens.flow.turning.set(0x1u32);
    menus.screens.state.carousel.menu.set(0x7u32);
    carousel(None)?;
    let item = menus.screens.items.current[slot::OPTIONS].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.state.carousel.front.set(item.wrapping_sub(1));
    menus.screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.options_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[
        Step::Write(|m| &m.screens.event.clears_escape_flag, 0x1),
        Step::Call(Callee::LoadBackground(|m| &m.tables.texts.options_picture)),
    ],
    keys: Keys {
        action_check: ActionCheck::Short,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.options.id(),
        |m| &m.screens.items.current[slot::OPTIONS],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::options_loop", &SCREEN)
}
