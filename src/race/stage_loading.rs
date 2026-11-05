//! `race::load_stage` (0x444040, IDA 2026-10-17): "SHAUNINITEVERYTHING": the loading
//! picture (unless `dword_71C3E0`), the cars' sounds woken, `race::loadlevel_init`, the
//! in-game music, the controls, race init (`sub_4462B9`, the game's until ported), the
//! clock restarted and the race's starting values; each step logged and indented.

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::Refused;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::tables_store::PHYSICS;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::clock_ticks_store::CLOCK_TICKS;
use crate::race::overlay_store::RACE_OVERLAY;
use crate::race::panel_store::PANEL;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;

/// "------", "SHAUNINITEVERYTHING - START", ": loading screen / ..", ": loadlevel_init",
/// ": doingamemusic", ": createcontrols", ": race init", "- FINIS", "------".
const WORDS: [&std::ffi::CStr; 9] = [
    c"----------------------------",
    c"SHAUNINITEVERYTHING - START",
    c"SHAUNINITEVERYTHING : loading screen / sounds",
    c"SHAUNINITEVERYTHING : loadlevel_init",
    c"SHAUNINITEVERYTHING : doingamemusic",
    c"SHAUNINITEVERYTHING : createcontrols",
    c"SHAUNINITEVERYTHING : race init",
    c"SHAUNINITEVERYTHING - FINIS",
    c"----------------------------",
];

/// `race::load_stage` (0x444040): a line break, a rule, "START", "loading screen" (indent
/// in): the loading picture (`menu::show_loading_picture`) unless `dword_71C3E0`,
/// `dword_70F3E8` -1, `race::wake_car_sounds`; then each with its word and the indent round
/// it: `loadlevel_init`, the music (`game_music` with `dword_64BB18`, else
/// `game_music_8`), `create_controls`, race init (`drive::race_init`) and
/// `scene::restart_clock`; the starting values (`STARTS` with the clock, the blocks'
/// +0xFB4 3 and +0xFB0 8, `LAST_STARTS`); "FINIS", a rule, a line break. The registers
/// and flags it leaves (eax the clock, carry clear) are read by none of its seven callers
/// (`taint.py` at each site: no read point; conversion phase 3, 2026-10-24).
pub fn load_stage() -> Result<(), Refused> {
    let indent = |by: i32| {
        crate::logfile::write_line::state::INDENT
            .atomic()
            .fetch_add(by as u32, std::sync::atomic::Ordering::Relaxed);
    };
    let mut word = 0;
    let mut log = || {
        crate::logfile::log_write::log_nul(&WORDS[word], true, 1);
        word += 1;
    };
    let newline = || crate::logfile::log_write::log_nul(&c"", true, 1);
    newline();
    log();
    log();
    log();
    indent(3);
    if RACE_WORK.in_credits.get() == 0
    /* dword_71C3E0 */
    {
        // As `menu::show_loading_picture`.
        crate::gfx::show_picture_file::show_picture_file(crate::pointers::GamePtr::to(
            &crate::menu::texts_store::MENU_TEXTS.loading_pcx,
        ))?;
    }
    REPLAY_FRAMES.last_kept.set(u32::MAX);
    crate::race::sounds::wake_car_sounds()?;
    indent(-3);
    log();
    indent(3);
    use crate::race::loading_bar_frame::STAGE_LOADING;
    STAGE_LOADING.store(true, std::sync::atomic::Ordering::Relaxed);
    let begun = std::time::Instant::now();
    crate::race::loadlevel::loadlevel_init()?;
    STAGE_LOADING.store(false, std::sync::atomic::Ordering::Relaxed);
    let (shown, micros) = crate::race::loading_bar_frame::shown();
    crate::framework::log(&format!(
        "race::load_stage: loaded in {} ms, {shown} bar frames in {} ms",
        begun.elapsed().as_millis(),
        micros / 1000
    ));
    indent(-3);
    log();
    indent(3);
    if crate::drive::finish_store::FINISH.no_place.get() != 0 {
        crate::race::music::game_music();
    } else {
        crate::race::music::game_music_8();
    }
    indent(-3);
    log();
    indent(3);
    crate::race::create_controls::create_controls();
    indent(-3);
    log();
    indent(3);
    crate::drive::race_init::race_init()?;
    crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    let clock: u32 = PHYSICS.scene_clock.bits();
    CLOCK_TICKS.clock_b.set(clock);
    CLOCK_TICKS.sound_ticks.set(0);
    PHYSICS.race_clock_now.set(clock);
    PHYSICS.race_clock_base.set(clock);
    RACE_OVERLAY.start_lights_lit.set(0);
    PANEL.lights[0].lit.set(u32::MAX);
    PANEL.lights[1].lit.set(u32::MAX);
    for car in &CAR_BLOCKS.cars[..8] {
        car.light_hold_steps.set(3);
    }
    for car in &CAR_BLOCKS.cars[..8] {
        car.value_jump_steps.set(8);
    }
    FRAME_SWITCHES.other_way.set(1);
    FRAME_SWITCHES.view_marks_on.set(1);
    GROUND_WORK.stage_loading.set(0);
    RACE_VIEW.view_on[0].set(1);
    RACE_VIEW.view_on[1].set(1);
    indent(-3);
    log();
    log();
    newline();
    Ok(())
}
