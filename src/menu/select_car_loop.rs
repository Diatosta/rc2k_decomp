//! `menu::select_car_loop` (0x47AC0C): the car selection screen, which the starters of
//! every kind of event call.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; it
//! has steps of its own first on leaving; the steps that are more than a call or a store are
//! the functions below, generated from the listing.
//! Screen: the car selection when an event is started (the owner called it the car select
//! menu, 2026-10-12; it ran in the Rust version under each of the five start screens that
//! day). Named `car_specs_47ac0c_loop` until then.
//! Callers: `sub_4182FA`, `sub_482382`, `sub_4824C1`, `sub_4824C1`, `sub_4826A2`, `sub_4826A2`,
//! `sub_4828B0`, `sub_4828B0`, `sub_4828B0`, `sub_482B7A`, `sub_482B7A`, `sub_482B7A`,
//! `sub_482B7A`, `sub_487E2C`, `sub_48AE39`, `sub_48AFD0`, `sub_48AFD0`, `sub_48B23D`,
//! `sub_48B23D`, `sub_48B54B`, `sub_48B54B`, `sub_48B54B`, `sub_48B90F`, `sub_48B90F`,
//! `sub_48B90F`, `sub_48B90F`, `sub_4A25F4`, `sub_4A26B6`, `sub_4A26B6`, `sub_4A27B5`,
//! `sub_4A27B5`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A28B4`, `sub_4A29F4`, `sub_4A29F4`,
//! `sub_4A29F4`, `sub_4A29F4`, `sub_4A9C9B`, `sub_4A9D5C`, `sub_4A9D5C`, `sub_4A9E3D`,
//! `sub_4A9E3D`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4A9F1E`, `sub_4AA040`, `sub_4AA040`,
//! `sub_4AA040`, `sub_4AA040`. Rust callers run its `entry` (`routes.rs`); the registers above are
//! the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene, quitting, read_at,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    // The current player's +0x30 (its car).
    let current: u32 = EVENT_STATE.current_player.get();
    let player = read_at((PLAYER_TABLE.pointers[0].addr()).wrapping_add(current.wrapping_mul(4)))?;
    c.arg = read_at(player.wrapping_add(0x30))?;
    SCREEN_FLOW.shown_gearing.set(c.arg);
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0xDu32);
    carousel(c)?;
    c.arg = SCREEN_ITEMS.current[slot::SELECT_CAR].get();
    LOOP_FLAGS.item.set(c.arg);
    c.arg = c.arg.wrapping_sub(1);
    MENU_STATE.carousel.front.set(c.arg);
    SCREEN_FLOW.turning.set(0x0u32);
    MENU_STATE.carousel.front.set(0x0u32);
    EVENT_STATE.shown_car.set(0x0u32);
    crate::menu::car_choice::find_player_car()?;
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xFFFFFFCEu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(c);
    Ok(())
}

fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

unsafe fn leaving(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        if CAPTIONS.screen_left.get() == 0x1 {
            Callee::GoBack.run(c)?;
        }
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.select_car.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::SELECT_CAR],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.select_car_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.select_car_draw.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.select_car_setup.addr() as usize,
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.select_car_overlay.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .select_car_picture
                .addr()) as usize
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
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
