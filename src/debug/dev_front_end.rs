//! The developers' front end (`dev_front_end::main`, 0x442B01, no references to it; IDA
//! 2026-10-30): a menu of keys over `TITLE.PCX` that came before the real menus, to pick a
//! course, a car and the switches and race at once. Here the "Developers" tab with
//! its settings (first values the image's) and its races (test course 0x44306B, finish sequence
//! 0x442FEF, credits 0x443104, last replay 0x4430C0), run from the menu loop's next frame.
//!
//! Not here: `var\soptions` (the tab keeps no prefs between runs), and 0x459C1B, which reads a
//! developer's `e:\rally5\bin` file the game does not ship.

use crate::debug::kstar::write_at;
use crate::errors::Refused;
use crate::race::switches_store::FRAME_SWITCHES;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The settings' numbers.
struct Settings {
    /// `dword_621838`: the course, 0 .. 99 (42 at first).
    course: AtomicU32,
    /// `dword_62183C`: the car's kind, 0 .. 0x21.
    car: AtomicU32,
    /// `dword_6218D0`: forwards, backwards.
    direction: AtomicU32,
    /// `dword_6218A0`: manual, automatic, bumper car.
    gears: AtomicU32,
    /// `dword_621928`: the cars less one.
    cars: AtomicU32,
    /// `dword_62192C`: the screens less one.
    screens: AtomicU32,
    /// `dword_621920`: single, simulation, arcade, time trial, editor free play.
    mode: AtomicU32,
    /// `dword_621924`: mipmapping.
    mips: AtomicU32,
    /// `dword_6219B8`: the rally, 0 .. 5.
    rally: AtomicU32,
    /// `dword_6219BC`: the stage.
    stage: AtomicU32,
}

static SETTINGS: Settings = Settings {
    course: AtomicU32::new(42),
    car: AtomicU32::new(0),
    direction: AtomicU32::new(0),
    gears: AtomicU32::new(1),
    cars: AtomicU32::new(0),
    screens: AtomicU32::new(0),
    mode: AtomicU32::new(0),
    mips: AtomicU32::new(0),
    rally: AtomicU32::new(0),
    stage: AtomicU32::new(0),
};

/// A race is asked for.
static WANTED: AtomicU32 = AtomicU32::new(0);

/// Below this the courses (1 .. 6, 11 .. 16, some 30 KB each) have no pace notes, no set-up and
/// no splits: the third is the original's own fatal error 0x80601843 (seen 2026-10-30).
const FIRST_RACED: usize = 0x15;

const TEST: u32 = 1;
const FINISH: u32 = 2;
const CREDITS: u32 = 3;
const REPLAY: u32 = 4;

/// `g_carBlockStore`, the blocks 0xFE0 apart; the first two get the settings.
const CARS: usize = 0x70F3EC;
const CAR_LEN: usize = 0xFE0;

/// `dev_front_end::set_up_race` (0x446994), by IDA's addresses as the debug pages write.
fn set_up_race() {
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let gears = s.gears.load(Relaxed);
    write_at(0x623850, s.course.load(Relaxed)); // g_stageNumber
    write_at(0x623858, car); // g_raceCarKinds
    write_at(0x62385C, (car + 1) % 0x17);
    // The two blocks' start slots. The set-up figures and tyres stay the player's
    // (`copy_settings_in`; the original set defaults here; owner, 2026-10-30).
    for (place, first, second) in [(0x70FE94, 1, 2), (0x70FA70, 4, 4)] {
        write_at(place, first);
        write_at(place + CAR_LEN, second);
    }
    write_at(0x623878, gears); // g_carColours
    write_at(0x62387C, gears);
    write_at(0x623898, s.mips.load(Relaxed)); // g_mipSetting
    write_at(0x62389C, s.mode.load(Relaxed)); // g_runKind
    write_at(0x623854, s.direction.load(Relaxed)); // g_direction
    write_at(0x63DB90, s.cars.load(Relaxed) + 1); // g_raceCars
    write_at(0x63DB94, s.screens.load(Relaxed) + 1);
    write_at(0x6238CC, s.rally.load(Relaxed)); // g_raceLevel
    write_at(0x6238D0, s.stage.load(Relaxed)); // g_raceStage
    // `sub_446B3A`, both blocks: what a race's start had left of the last.
    for block in 0..2 {
        let at = CARS + block * CAR_LEN;
        for offset in (0x2B8..=0x364).step_by(4).chain((0x64..=0x80).step_by(4)) {
            write_at(at + offset, 0);
        }
        write_at(at + 0x298, u32::MAX);
    }
}

/// `f` between what the menus' own route does round a race (`race::single_rally`): the
/// game's mode changes of `sub_44326D` are the menus' here.
fn round_a_race(f: impl FnOnce() -> Result<(), Refused>) -> Result<(), Refused> {
    // `sub_44326D`'s, which the race's sound objects are made from (without them, a race
    // with no engine heard); the menus' own put back after.
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let menus: [(usize, u32); 6] = [
        (0x75ACE8, car),
        (0x75ACEC, (car + 1) % 0x17),
        (0x75ACD8, s.mode.load(Relaxed)),
        (0x75ACE0, s.screens.load(Relaxed) + 1),
        (0x75AE20, 0),
        (0x75AF8C, 1),
    ];
    let kept = menus.map(|(at, _)| crate::debug::wear_page::value_at(at).unwrap_or(0));
    // The player's HUD, keys and set-up, before the tab's own over them.
    crate::race::single_rally::player_settings_in()?;
    for (at, v) in menus {
        write_at(at, v);
    }
    use crate::race::single_rally::step;
    use crate::race::{race_end, race_shell, stage_free};
    step(2)?; // sound::stop_music
    step(3)?; // race::set_race_screen_mode
    step(4)?; // menu::show_loading_picture
    step(5)?; // race::set_up_race_sound_objects
    step(6)?; // sound::set_up_race_sounds
    stage_free::render3d_tidy()?;
    stage_free::objects_tidy()?;
    race_end::release_textures();
    FRAME_SWITCHES.mask_kind.set(0);
    race_shell::clear_player_absences();
    f()?;
    step(2)?; // sound::stop_music
    step(8)?; // gfx::set_menu_screen_mode
    step(4)?; // menu::show_loading_picture
    step(9)?; // race::clear_race_sound_objects
    step(10)?; // sound::apply_settings
    step(11)?; // scene::set_up_engine
    step(12)?; // scene::set_up_front_end_scene
    step(11)?;
    step(13)?; // input::take_key
    for ((at, _), v) in menus.into_iter().zip(kept) {
        write_at(at, v);
    }
    Ok(())
}

/// `dev_front_end::test_course` (0x44306B) and `dev_front_end::run_race` (0x443F96).
fn test_course() -> Result<(), Refused> {
    use crate::race::{race_end, race_shell, replay, stage_loading};
    round_a_race(|| {
        crate::race::numbers_store::NUMBERS.replay_mode.set_bits(1);
        write_at(0x60519C, 0x12345678); // g_noiseSeed
        write_at(0x60101C, 2); // g_arcadeKind
        FRAME_SWITCHES.editor_mode.set(0);
        // Free play leaves the car where it was: on another course, or with no race
        // yet, that is nowhere, and the first search ends the game (seen 2026-10-30).
        let course = SETTINGS.course.load(Relaxed);
        crate::debug::screens::place_free_play(
            !crate::debug::screens::raced()
                || crate::race::vars_store::RACE_VARS.stage_number.bits() != course,
        );
        // The player's settings, as a race from the menus; then this tab's.
        crate::race::settings_copy::copy_settings_in()?;
        set_up_race();
        crate::race::race_cars::set_up_arcade_cars()?;
        race_shell::set_start_states()?;
        stage_loading::load_stage()?;
        race_end::record_replay_header();
        replay::open_replay()?;
        crate::race::view_store::RACE_VIEW.after_credits.set(0);
        crate::drive::play_race::play_race()?;
        replay::close_replay()?;
        race_end::tidy_everything()
    })
}

/// `sub_442FEF`: the finish sequence, for the course's index (0 .. 5) and the car.
fn finish_sequence() -> Result<(), Refused> {
    round_a_race(|| {
        let s = &SETTINGS;
        let car = s.car.load(Relaxed);
        crate::race::numbers_store::NUMBERS.replay_mode.set_bits(1);
        write_at(0x64BB18, 1); // g_finishNoPlace
        write_at(0x75B0FC, s.course.load(Relaxed).saturating_sub(1).min(5));
        write_at(0x75B104, car);
        write_at(0x75B100, 0x45);
        write_at(0x75B108, car & 1);
        let result = crate::drive::finish_race::finish_race();
        write_at(0x64BB18, 0);
        result
    })
}

/// `sub_443104`, without its fonts (the menus' are up): the credits' race.
fn credits() -> Result<(), Refused> {
    round_a_race(|| {
        crate::race::sound_tables::silence_race_sounds()?;
        crate::race::credits_race::credits_race()
    })
}

/// `sub_4430C0`: the last race's replay (nothing happens without one).
fn last_replay() -> Result<(), Refused> {
    round_a_race(|| {
        write_at(0x5C5CE0, 2); // g_replayMode
        write_at(0x63DB7C, u32::MAX); // g_playerOptions
        write_at(0x63DB80, u32::MAX);
        let result =
            crate::race::replay_mode::replay_mode(crate::race::replay_viewer::Mode::LastRace);
        write_at(0x5C5CE0, 0);
        result
    })
}

/// A menu's frame: runs what was asked for; true if something ran.
pub fn serve() -> Result<bool, Refused> {
    let run: fn() -> Result<(), Refused> = match WANTED.swap(0, Relaxed) {
        TEST => test_course,
        FINISH => finish_sequence,
        CREDITS => credits,
        REPLAY => last_replay,
        _ => return Ok(false),
    };
    run()?;
    Ok(true)
}

mod tab {
    use super::*;
    use dear_imgui_rs::Ui;

    /// A setting with its names, stepped by buttons.
    fn pick(ui: &Ui, label: &str, value: &AtomicU32, names: &[&str]) {
        let now = (value.load(Relaxed) as usize).min(names.len() - 1);
        let mut chosen = now;
        ui.set_next_item_width(180.0);
        if ui.combo_simple_string(label, &mut chosen, names) && chosen != now {
            value.store(chosen as u32, Relaxed);
        }
    }

    /// A number within `0 ..= top`, dragged.
    fn number(ui: &Ui, label: &str, value: &AtomicU32, top: i32) {
        let mut v = value.load(Relaxed) as i32;
        ui.set_next_item_width(180.0);
        if ui.drag(label, &mut v) {
            value.store(v.clamp(0, top) as u32, Relaxed);
        }
    }

    /// The tab.
    pub fn tab(ui: &Ui) {
        let s = &SETTINGS;
        number(ui, "Course", &s.course, 99);
        let course = s.course.load(Relaxed) as usize;
        let present = FRAME_SWITCHES.course_present[course].get() != 0;
        ui.same_line();
        if ui.button("Find the courses")
            && let Err(e) = crate::race::find_present_courses::find_present_courses()
        {
            crate::framework::log(&format!("debug: find the courses: {e:?}"));
        }
        ui.same_line();
        ui.text_disabled(if present {
            "present"
        } else {
            "not found (or not looked for)"
        });
        number(ui, "Car", &s.car, 0x21);
        ui.same_line();
        let name = crate::race::car_specs_store::CAR_SPECS.cars[s.car.load(Relaxed) as usize]
            .name
            .read();
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        ui.text_disabled(String::from_utf8_lossy(&name[..end]));
        pick(ui, "Direction", &s.direction, &["forwards", "backwards"]);
        pick(
            ui,
            "Gears",
            &s.gears,
            &["manual", "semi-automatic", "automatic (\"bumper car\")"],
        );
        pick(
            ui,
            "Cars",
            &s.cars,
            &[
                "1 car", "2 cars", "3 cars", "4 cars", "5 cars", "6 cars", "7 cars", "8 cars",
            ],
        );
        pick(ui, "Screens", &s.screens, &["1 screen", "2 screens"]);
        pick(
            ui,
            "Game mode",
            &s.mode,
            &[
                "single",
                "simulation",
                "arcade",
                "time trial",
                "editor free play",
            ],
        );
        if ui.is_item_hovered() {
            ui.tooltip_text(
                "Editor free play: the car is not put at the start (it stays where the last \
                 race left it, until the game closes), no timers, no split times, no \
                 collisions between cars",
            );
        }
        pick(ui, "Mipmapping", &s.mips, &["no", "yes"]);
        ui.text_disabled("Rally and stage: only the arcade mode reads them (its cars' tables)");
        number(ui, "Rally", &s.rally, 5);
        number(ui, "Stage", &s.stage, 5);
        if course < FIRST_RACED {
            ui.text_disabled("f1: a test course: cannot be raced");
        } else if ui.button("f1: test current course") {
            WANTED.store(TEST, Relaxed);
        }
        ui.same_line();
        if ui.button("finish sequence") {
            WANTED.store(FINISH, Relaxed);
        }
        ui.same_line();
        if ui.button("credits race") {
            WANTED.store(CREDITS, Relaxed);
        }
        ui.same_line();
        if ui.button("last replay") {
            WANTED.store(REPLAY, Relaxed);
        }
        ui.new_line();
        ui.text_disabled(
            "runs at the next menu's frame; a course or car not on disk ends the game",
        );
    }
}

pub use tab::tab;
