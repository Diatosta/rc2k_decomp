//! `menu::main_menu_loop` (0x4748D9): the loop of the main menu, the screen the game starts
//! on (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! - background `mainscr.png`, loaded first thing at set-up, and its carry is not looked at;
//! - `captions.left_by_key` is cleared and the highlight put in place only after the first frame
//!   (`late_highlight`), and `cheats.notice_due` = 1 is stored before every frame;
//! - an idle timer at the top of each frame (`idle`): while `loop_flags.idle` is not 0 (set by
//!   `menu::update_blink` after 300,000 ms without a key) `loop_flags.idle_frames` counts up, and
//!   unless `g_noFmv` (0x5BE870, `nofmv` in `var\ral.cfg`) is 1, `menu::play_films` is called with
//!   eax = 1 (it plays the intro film), the flag cleared and the screen set up again;
//! - after an action: `restart_action_check`;
//! - leaving is always asked about, and after a "yes" the loop returns at once
//!   (`always_asks`); the quit switch leads to the question too.
//!
//! The five calls and three stores between the last jump and the end (0x474CD9 ..
//! 0x474D0F) are never reached and are not ported. The name is a reading from the menu
//! (0x748CB8): its items open `start_rally_loop`, `load_championship_loop`, `network_rally_loop`
//! and `options_loop`, and the fifth is `menu::go_back`. Screen: the main menu. Rust callers run
//! its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::framework::log;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Asking, Callee, Entry, Flow, Frames, Keys, List, MenuSound, Screen, Step,
    carousel, prepare_scene,
};
use crate::menu::update_blink::update_blink;

// ---- game data ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus.screens.event.event_over.set(false);
    // menu::load_background (whether it failed is not looked at)
    Callee::LoadBackground(|m| &m.tables.texts.main_menu_picture).run(entry)?;
    entry.front_end.window.view_offset[0].set(7u32);
    entry.front_end.window.view_offset[1].set(0xAu32);
    entry.front_end.window.view_zoom.set(0x100u32);
    prepare_scene(entry.scene); // scene::prepare_menu_scene
    menus.screens.loop_flags.ask_leave.set(false);
    Callee::TakeKey.run(entry)?; // input::take_key
    menus.screens.draw_work.escape_pressed.set(false);
    menus
        .screens
        .option_work
        .set_sound(rc2k_core::menu::keys::MenuSound::None);
    menus.screens.flow.turning.set(1u32);
    menus.screens.state.carousel.menu.set(0u32);
    carousel(None)?;
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
    if !menus.screens.loop_flags.idle.is_set() {
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
    // `menu::play_films` with mode 1.
    crate::menu::play_films::play_films(entry.owners, true)?;
    log("menu::main_menu_loop: `menu::play_films` has come back");
    menus.screens.loop_flags.idle.set(false);
    update_blink(menus, system);
    let now = system.session.clocks.frame.get();
    menus.play.champ_view.last_key_time.set(now);
    system.session.clocks.setup_time.set(now);
    Ok(Flow::SetUp)
}

pub const SCREEN: Screen = Screen {
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
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.main_menu_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.main_menu_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(|m| &m.screens.loop_flags.event_left, false),
        Step::WriteFlag(|m| &m.screens.loop_flags.leave_screen, false),
        Step::WriteFlag(|m| &m.screens.loop_flags.left_with_carry, false),
        Step::WriteFlag(|m| &m.screens.event.event_over, false),
        Step::Sound(MenuSound::None),
        Step::WriteFlag(|m| &m.screens.draw_work.escape_pressed, false),
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
    late_highlight: true,
    asking: Some(Asking {
        on_refusal: &[
            Step::WriteFlag(|m| &m.screens.loop_flags.ask_leave, false),
            Step::WriteFlag(|m| &m.screens.draw_work.escape_pressed, false),
            Step::Sound(MenuSound::None),
            Step::WriteFlag(|m| &m.screens.loop_flags.leave_screen, false),
        ],
        always: true,
        ..Asking::DEFAULT
    }),
    keys: Keys {
        action_check: ActionCheck::Restart,
        ..Keys::DEFAULT
    },
    frames: Frames {
        before_first_frame: &[Step::Do(before_first_frame)],
        before_frame: &[Step::Write(|m| &m.play.cheats.notice_due, 1)],
        after_blink: &[Step::DoFlow(idle)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.main_menu.id(),
        |m| &m.screens.items.current[slot::MAIN_MENU],
    )
};
