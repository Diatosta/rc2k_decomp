//! `menu::main_menu_loop` (0x4748D9): the loop of the main menu, the screen the game starts
//! on (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! - background `mainscr.png`, loaded first thing at set-up, and its carry is not looked at;
//! - `dword_758806` is cleared and the highlight put in place only after the first frame
//!   (`late_highlight`), and `dword_7585D8` = 1 is stored before every frame;
//! - an idle timer at the top of each frame (`idle`): while `dword_755551` is not 0 (set by
//!   `menu::update_blink` after 300,000 ms without a key) `dword_755555` counts up, and
//!   unless `g_noFmv` (0x5BE870, `nofmv` in `var\ral.cfg`) is 1, `sub_4161CD` is called with
//!   eax = 1 (it plays the intro film), the flag cleared and the screen set up again;
//! - after an action: `restart_action_check`;
//! - leaving is always asked about, and after a "yes" the loop returns at once
//!   (`always_asks`); the quit switch leads to the question too.
//!
//! The five calls and three stores between the last jump and the final `popa` (0x474CD9 ..
//! 0x474D0F) are never reached and are not in the Rust version. The name is a reading from the menu
//! (0x748CB8): its items open `start_rally_loop`, `load_championship_loop`, `network_rally_loop`
//! and `options_loop`, and the fifth is `menu::go_back`. Screen: the main menu. Caller:
//! `sub_47486C`. Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::{log, rva};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::cheats_store::CHEAT_VARS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene,
};
use crate::menu::state_store::MENU_STATE;
use crate::menu::update_blink::update_blink;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::CLOCKS;

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    EVENT_STATE.event_done.set(0u32);
    c.arg = crate::menu::texts_store::MENU_TEXTS
        .main_menu_picture
        .addr();
    Callee::LoadBackground.run(c)?; // menu::load_background (its carry not looked at)
    DRAW.view_offset[0].set(7u32);
    DRAW.view_offset[1].set(0xAu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(c); // scene::prepare_menu_scene
    LOOP_FLAGS.ended_a.set(0u32);
    Callee::TakeKey.run(c)?; // input::take_key
    DRAW_WORK.escaped.set(0u32);
    OPTION_WORK.menu_sound.set(0x6Fu32);
    SCREEN_FLOW.turning.set(1u32);
    MENU_STATE.carousel.menu.set(0u32);
    carousel(c)?;
    c.arg = SCREEN_ITEMS.current[slot::MAIN_MENU].get();
    LOOP_FLAGS.item.set(c.arg);
    c.arg = c.arg.wrapping_sub(1);
    MENU_STATE.carousel.front.set(c.arg);
    Ok(())
}

/// With the clock in `Carried::arg`.
fn before_first_frame(c: &mut Carried) -> Result<(), Refused> {
    CHAMP_VIEW.last_key_time.set(c.arg);
    CHEAT_VARS.notice_due.set(1u32);

    Ok(())
}

fn idle(c: &mut Carried) -> Result<Flow, Refused> {
    if LOOP_FLAGS.idle.get() == 0 {
        return Ok(Flow::On);
    }
    LOOP_FLAGS
        .idle_frames
        .set(LOOP_FLAGS.idle_frames.get().wrapping_add(1));
    if crate::film::NO_FMV.get() == 1 {
        return Ok(Flow::On);
    }
    c.arg = 1;
    log("menu::main_menu_loop: idle, sub_4161CD is called");
    // `menu::play_films` with mode 1 in eax; it keeps every register. edi is the menu
    // here (the loop's last store to it); ebp it does not read.
    crate::menu::play_films::play_films(true)?;
    log("menu::main_menu_loop: sub_4161CD has come back");
    LOOP_FLAGS.idle.set(0u32);
    update_blink();
    c.arg = CLOCKS.frame.get();
    CHAMP_VIEW.last_key_time.set(c.arg);
    CLOCKS.setup_time.set(c.arg);
    Ok(Flow::SetUp)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.main_menu.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::MAIN_MENU],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.main_menu_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.main_menu_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.main_menu_setup.id(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.main_menu_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(|| &LOOP_FLAGS.went_back, 0),
        Step::Write(|| &LOOP_FLAGS.back, 0),
        Step::Write(|| &LOOP_FLAGS.ended_b, 0),
        Step::Write(|| &EVENT_STATE.event_done, 0),
        Step::Write(|| &OPTION_WORK.menu_sound, 0x6F),
        Step::Write(|| &DRAW_WORK.escaped, 0),
        Step::Write(|| &SCREEN_ITEMS.current[slot::MAIN_MENU], 1),
        Step::Write(|| &LOOP_FLAGS.item, 1),
        Step::Call(Callee::SetUpFrontEnd),
        Step::Call(Callee::TakeKey), // input::take_key
        Step::Call(Callee::SetUpEngine),
        Step::Call(Callee::SetUpFrontEndScene),
        Step::Call(Callee::SetUpEngine),
        Step::Write(|| &OBJECT_WORK.light_lowest, 0x50),
        Step::Write(|| &OBJECT_WORK.light_highest, 0xFF),
        Step::Write(|| &OBJECT_WORK.light_level, 0),
        Step::Write(|| &OBJECT_WORK.light_strength, 0xFF),
        Step::Write(|| &OBJECT_WORK.light_turns[0], 0x6E),
        Step::Write(|| &OBJECT_WORK.light_turns[1], 0x1E),
        Step::Call(Callee::PlayTune4),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: Some(&[
        Step::Write(|| &LOOP_FLAGS.ended_a, 0),
        Step::Write(|| &DRAW_WORK.escaped, 0),
        Step::Write(|| &OPTION_WORK.menu_sound, 0x6F),
        Step::Write(|| &LOOP_FLAGS.back, 0),
    ]),
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
    late_highlight: true,
    before_first_frame: &[Step::Do(before_first_frame)],
    before_frame: &[Step::Write(|| &CHEAT_VARS.notice_due, 1)],
    after_blink: &[Step::DoFlow(idle)],
    restart_action_check: true,
    always_asks: true,
};
