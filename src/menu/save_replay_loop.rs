//! `menu::save_replay_loop` (0x47F77B): the replay naming screen "GRAVAR REPETIÇÃO"; returns
//! the slot the replay goes to.
//!
//! In:  nothing
//! Out: eax = the result; every other register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it has a step of its own once a frame; it returns a value in eax;
//! the steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "GRAVAR REPETIÇÃO", not yet a
//! visit.
//! Callers: `sub_46F786`.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, Flow, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn each_frame(c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if OPTION_WORK.typing.get() == 0x1 {
            Callee::EditReplayName.run(c)?;
            if !c.carry {
                if OPTION_WORK.typing.get() != 0x0 {
                    return Ok(Flow::NextFrame);
                }
                if CAPTIONS.screen_left.get() == 0x1 {
                    CAPTIONS.screen_left.set(0x0u32);
                    c.arg = 0xFFFFFFFF;
                    DRAW_WORK.work_0c.set(0xFFFFFFFFu32);
                    return Ok(Flow::Leave);
                }
                c.arg = MENU_PAGES.replay_page.get();
                c.arg = c.arg.wrapping_mul(0x14);
                c.arg = c.arg.wrapping_add(LOOP_FLAGS.item.get());
                c.arg = c.arg.wrapping_sub(1);
                DRAW_WORK.work_0c.set(c.arg);
                return Ok(Flow::Leave);
            }
        }
        Ok(Flow::On)
    }
}

/// What the loop returns in eax, written by hand from the end of the listing (after the
/// `popa`): `dword_755A0C`, which the step above leaves at -1 or at the replay's number.
fn result() -> u32 {
    DRAW_WORK.work_0c.get()
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.save_replay.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::SAVE_REPLAY],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.unread_list, // g_unreadList
            table: || DRAW_LISTS.unread_758b72.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758bce.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.save_replay_setup.addr() as usize,
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_75914e.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.save_replay_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758bce.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_75914e.addr() as usize,
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(|| &DRAW_WORK.escaped, 0x0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .save_replay_picture
                .addr()) as usize
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: true,
    each_frame: &[Step::DoFlow(each_frame)],
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
    result: Some(result),
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
