//! `menu::main_menu_loop` (0x4748D9): the loop of the main menu, the screen the game starts
//! on (a reading)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! - background `mainscr.png`, loaded first thing at set-up, and its carry is not looked at;
//! - `dword_758806` is cleared and the highlight put in place only after the first frame
//!   (`late_highlight`), and `dword_7585D8` = 1 is stored before every frame;
//! - an idle timer at the top of each frame (`idle`): while `dword_755551` is not 0 (set by
//!   `menu::update_blink` after 300,000 ms without a key) `dword_755555` counts up, and
//!   unless `g_noFmv` (0x5BE870, `nofmv` in `var\ral.cfg`) is 1, `menu::play_films` is called with
//!   eax = 1 (it plays the intro film), the flag cleared and the screen set up again;
//! - after an action: `restart_action_check`;
//! - leaving is always asked about, and after a "yes" the loop returns at once
//!   (`always_asks`); the quit switch leads to the question too.
//!
//! The five calls and three stores between the last jump and the end (0x474CD9 ..
//! 0x474D0F) are never reached and are not in the Rust version. The name is a reading from the menu
//! (0x748CB8): its items open `start_rally_loop`, `load_championship_loop`, `network_rally_loop`
//! and `options_loop`, and the fifth is `menu::go_back`. Screen: the main menu. Rust callers run
//! its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};
use crate::menu::update_blink::update_blink;

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let drawing = &entry.scene.drawing;
    let menus = entry.menus;
    menus.screens.event.event_done.set(0u32);
    // menu::load_background (whether it failed is not looked at)
    Callee::LoadBackground(|m| &m.tables.texts.main_menu_picture).run(entry)?;
    drawing.scene.view_offset[0].set(7u32);
    drawing.scene.view_offset[1].set(0xAu32);
    drawing.scene.view_zoom.set(0x100u32);
    prepare_scene(entry.scene); // scene::prepare_menu_scene
    menus.screens.loop_flags.ended_a.set(0u32);
    Callee::TakeKey.run(entry)?; // input::take_key
    menus.screens.draw_work.escaped.set(0u32);
    menus.screens.option_work.menu_sound.set(0x6Fu32);
    menus.screens.flow.turning.set(1u32);
    menus.screens.state.carousel.menu.set(0u32);
    carousel(None, entry.scene)?;
    let item = menus.screens.items.current[slot::MAIN_MENU].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

/// The loop has just set the set-up time to the frame clock.
fn before_first_frame(entry: &Entry) -> Result<(), Refused> {
    let play = &entry.menus.play;
    play.champ_view
        .last_key_time
        .set(entry.system.session.clocks.setup_time.get());
    play.cheats.notice_due.set(1u32);

    Ok(())
}

fn idle(entry: &Entry) -> Result<Flow, Refused> {
    let system = entry.system;
    let menus = entry.menus;
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
    log("menu::main_menu_loop: idle, `menu::play_films` is called");
    // `menu::play_films` with mode 1 in eax; it keeps every register. edi is the menu
    // here (the loop's last store to it); ebp it does not read.
    crate::menu::play_films::play_films(true)?;
    log("menu::main_menu_loop: `menu::play_films` has come back");
    menus.screens.loop_flags.idle.set(0u32);
    update_blink(menus);
    let now = system.session.clocks.frame.get();
    menus.play.champ_view.last_key_time.set(now);
    system.session.clocks.setup_time.set(now);
    Ok(Flow::SetUp)
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.main_menu.id(),
    saved_item: |m| &m.screens.items.current[slot::MAIN_MENU],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.main_menu_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.main_menu_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.main_menu_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.main_menu_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[
        Step::Write(|m| &m.screens.loop_flags.went_back, 0),
        Step::Write(|m| &m.screens.loop_flags.back, 0),
        Step::Write(|m| &m.screens.loop_flags.ended_b, 0),
        Step::Write(|m| &m.screens.event.event_done, 0),
        Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F),
        Step::Write(|m| &m.screens.draw_work.escaped, 0),
        Step::Write(|m| &m.screens.items.current[slot::MAIN_MENU], 1),
        Step::Write(|m| &m.screens.loop_flags.item, 1),
        Step::Call(Callee::SetUpFrontEnd),
        Step::Call(Callee::TakeKey), // input::take_key
        Step::Call(Callee::SetUpEngine),
        Step::Call(Callee::SetUpFrontEndScene),
        Step::Call(Callee::SetUpEngine),
        Step::WriteLight(|l| &l.light_lowest, 0x50),
        Step::WriteLight(|l| &l.light_highest, 0xFF),
        Step::WriteLight(|l| &l.light_level, 0),
        Step::WriteObject(|o| &o.light_strength, 0xFF),
        Step::WriteLight(|l| &l.light_turns[0], 0x6E),
        Step::WriteLight(|l| &l.light_turns[1], 0x1E),
        Step::Call(Callee::PlayTune4),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: Some(&[
        Step::Write(|m| &m.screens.loop_flags.ended_a, 0),
        Step::Write(|m| &m.screens.draw_work.escaped, 0),
        Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F),
        Step::Write(|m| &m.screens.loop_flags.back, 0),
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
    before_frame: &[Step::Write(|m| &m.play.cheats.notice_due, 1)],
    after_blink: &[Step::DoFlow(idle)],
    restart_action_check: true,
    always_asks: true,
};
