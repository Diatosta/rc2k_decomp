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
use crate::framework::log;
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};
use crate::menu::update_blink::update_blink;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::CLOCKS;

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    let menus = menus();
    menus.screens.event.event_done.set(0u32);
    // menu::load_background (whether it failed is not looked at)
    Callee::LoadBackground(|| &crate::menu::owner::menus().tables.texts.main_menu_picture).run()?;
    DRAW.view_offset[0].set(7u32);
    DRAW.view_offset[1].set(0xAu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(); // scene::prepare_menu_scene
    menus.screens.loop_flags.ended_a.set(0u32);
    Callee::TakeKey.run()?; // input::take_key
    menus.screens.draw_work.escaped.set(0u32);
    menus.screens.option_work.menu_sound.set(0x6Fu32);
    menus.screens.flow.turning.set(1u32);
    menus.screens.state.carousel.menu.set(0u32);
    carousel(None)?;
    let item = menus.screens.items.current[slot::MAIN_MENU].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

/// The loop has just set the set-up time to the frame clock.
fn before_first_frame(_: &Entry) -> Result<(), Refused> {
    let play = &menus().play;
    play.champ_view.last_key_time.set(CLOCKS.setup_time.get());
    play.cheats.notice_due.set(1u32);

    Ok(())
}

fn idle(_: &Entry) -> Result<Flow, Refused> {
    let menus = menus();
    if menus.screens.loop_flags.idle.get() == 0 {
        return Ok(Flow::On);
    }
    menus
        .screens
        .loop_flags
        .idle_frames
        .set(menus.screens.loop_flags.idle_frames.get().wrapping_add(1));
    if crate::film::NO_FMV.get() == 1 {
        return Ok(Flow::On);
    }
    log("menu::main_menu_loop: idle, sub_4161CD is called");
    // `menu::play_films` with mode 1 in eax; it keeps every register. edi is the menu
    // here (the loop's last store to it); ebp it does not read.
    crate::menu::play_films::play_films(true)?;
    log("menu::main_menu_loop: sub_4161CD has come back");
    menus.screens.loop_flags.idle.set(0u32);
    update_blink(menus);
    let now = CLOCKS.frame.get();
    menus.play.champ_view.last_key_time.set(now);
    CLOCKS.setup_time.set(now);
    Ok(Flow::SetUp)
}

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.main_menu.id(),
    saved_item: || &menus().screens.items.current[slot::MAIN_MENU],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || menus().tables.draw_lists.main_menu_scene.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.main_menu_draw.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.setup_list, // dword_755500
            table: || menus().tables.draw_lists.main_menu_setup.id(),
            run: true,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || menus().tables.draw_lists.main_menu_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[
        Step::Write(|| &menus().screens.loop_flags.went_back, 0),
        Step::Write(|| &menus().screens.loop_flags.back, 0),
        Step::Write(|| &menus().screens.loop_flags.ended_b, 0),
        Step::Write(|| &menus().screens.event.event_done, 0),
        Step::Write(|| &menus().screens.option_work.menu_sound, 0x6F),
        Step::Write(|| &menus().screens.draw_work.escaped, 0),
        Step::Write(|| &menus().screens.items.current[slot::MAIN_MENU], 1),
        Step::Write(|| &menus().screens.loop_flags.item, 1),
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
        Step::Write(|| &menus().screens.loop_flags.ended_a, 0),
        Step::Write(|| &menus().screens.draw_work.escaped, 0),
        Step::Write(|| &menus().screens.option_work.menu_sound, 0x6F),
        Step::Write(|| &menus().screens.loop_flags.back, 0),
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
    before_frame: &[Step::Write(|| &menus().play.cheats.notice_due, 1)],
    after_blink: &[Step::DoFlow(idle)],
    restart_action_check: true,
    always_asks: true,
};
