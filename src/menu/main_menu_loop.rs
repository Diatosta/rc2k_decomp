//! The loop of the main menu, the screen the game starts on. Screen-specific part of
//! `menu::screen_loop`:
//! - background `mainscr.png`, loaded first thing at set-up;
//! - `captions.left_by_key` is cleared and the highlight put in place only after the first frame
//!   (`late_highlight`), and `cheats.notice_due` = 1 is stored before every frame;
//! - an idle timer at the top of each frame (`idle`): while `loop_flags.idle` is not 0 (set by
//!   `menu::update_blink` after 300,000 ms without a key) `loop_flags.idle_frames` counts up,
//!   and unless `nofmv` in `var\ral.cfg` is 1, `menu::play_films` plays the intro film, the flag
//!   is cleared and the screen set up again;
//! - after an action: `restart_action_check`;
//! - leaving is always asked about, and after a "yes" the loop returns at once
//!   (`always_asks`); the quit switch leads to the question too.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Asking, Callee, Entry, Flow, Frames, Keys, List, MenuSound, SceneVar, Screen,
    ScreenFlag, ScreenVar, Step, carousel, prepare_scene,
};
use crate::menu::update_blink::update_blink;
use crate::program_log::log;
use crate::system::owner::System;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus.screens.event.write().event_over = false;
    // menu::load_background (whether it failed is not looked at)
    Callee::LoadBackground(|m| &m.texts.main_menu_picture).run(entry)?;
    entry.front_end.window.view_offset[0].set(7u32);
    entry.front_end.window.view_offset[1].set(0xAu32);
    entry.front_end.window.view_zoom.set(0x100u32);
    prepare_scene(&entry.owners); // scene::prepare_menu_scene
    menus.screens.loop_flags.write().ask_leave = false;
    Callee::TakeKey.run(entry)?; // input::take_key
    menus.screens.draw_work.write().escape_pressed = false;
    menus
        .screens
        .entry
        .set_sound(rc2k_core::menu::keys::MenuSound::None);
    menus.screens.flow.write().turning = 1u32;
    menus.screens.carousel.menu.set(0u32);
    carousel(entry, None)?;
    let item = menus.screens.items.current(slot::MAIN_MENU);
    menus.screens.loop_flags.write().item = item;
    menus
        .screens
        .carousel
        .front
        .set(crate::menu::web_limits::carousel_entry(item));
    Ok(())
}

/// The loop has just set the set-up time to the frame clock.
fn before_first_frame(menus: &Menus, system: &System) -> Result<(), Refused> {
    let play = &menus.play;
    play.champ_view.write().key_clock.last_key_time = system.clocks().setup_time.get();
    play.cheats.write().notice_due = 1u32;

    Ok(())
}

fn idle(entry: &Entry) -> Result<Flow, Refused> {
    let system = entry.owners.system;
    let menus = entry.menus;
    if !menus.screens.loop_flags.read().idle {
        return Ok(Flow::On);
    }
    let mut flags = menus.screens.loop_flags.write();
    flags.idle_frames = flags.idle_frames.wrapping_add(1);
    drop(flags);
    if entry.owners.system.machine().no_fmv.get() == 1 {
        return Ok(Flow::On);
    }
    log("menu::main_menu_loop: idle, `menu::play_films` is called");
    // `menu::play_films` with mode 1.
    crate::menu::play_films::play_films(entry.owners, true)?;
    log("menu::main_menu_loop: `menu::play_films` has come back");
    menus.screens.loop_flags.write().idle = false;
    update_blink(menus, system);
    let now = system.clocks().frame.get();
    menus.play.champ_view.write().key_clock.last_key_time = now;
    system.clocks().setup_time.set(now);
    Ok(Flow::SetUp)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.main_menu_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.main_menu_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.main_menu_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.main_menu_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(ScreenFlag::EventLeft, false),
        Step::WriteFlag(ScreenFlag::LeaveScreen, false),
        Step::WriteFlag(ScreenFlag::LeftWithCarry, false),
        Step::WriteFlag(ScreenFlag::EventOver, false),
        Step::Sound(MenuSound::None),
        Step::WriteFlag(ScreenFlag::EscapePressed, false),
        Step::Write(ScreenVar::SavedItem(slot::MAIN_MENU), 1),
        Step::Write(ScreenVar::Highlight, 1),
        Step::Call(Callee::SetUpFrontEnd),
        Step::Call(Callee::TakeKey), // input::take_key
        Step::Call(Callee::SetUpEngine),
        Step::Call(Callee::SetUpFrontEndScene),
        Step::Call(Callee::SetUpEngine),
        Step::WriteScene(SceneVar::LightLowest, 0x50),
        Step::WriteScene(SceneVar::LightHighest, 0xFF),
        Step::WriteScene(SceneVar::LightLevel, 0),
        Step::WriteScene(SceneVar::ObjectLightStrength, 0xFF),
        Step::WriteScene(SceneVar::LightTurn0, 0x6E),
        Step::WriteScene(SceneVar::LightTurn1, 0x1E),
        // A race or a replay asked for on the command line, once: it runs from here, as the
        // menus' own choices run from a screen, with the engine and the scene set up.
        Step::Do(crate::menu::launch::run_once),
        Step::Call(Callee::PlayTune4),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    late_highlight: true,
    asking: Some(Asking {
        on_refusal: &[
            Step::WriteFlag(ScreenFlag::AskLeave, false),
            Step::WriteFlag(ScreenFlag::EscapePressed, false),
            Step::Sound(MenuSound::None),
            Step::WriteFlag(ScreenFlag::LeaveScreen, false),
        ],
        always: true,
        ..Asking::DEFAULT
    }),
    keys: Keys {
        action_check: ActionCheck::Restart,
        ..Keys::DEFAULT
    },
    frames: Frames {
        before_first_frame: &[Step::DoMenusSystem(before_first_frame)],
        before_frame: &[Step::Write(ScreenVar::NoticeDue, 1)],
        after_blink: &[Step::DoFlow(idle)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.main_menu.id(),
        slot::MAIN_MENU,
    )
};
