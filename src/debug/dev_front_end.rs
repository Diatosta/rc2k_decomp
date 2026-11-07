//! The developers' front end (`dev_front_end::main`, 0x442B01, no references to it; IDA
//! 2026-10-30): a menu of keys over `TITLE.PCX` that came before the real menus, to pick a
//! course, a car and the switches and race at once. Here the "Developers" tab with
//! its settings (first values the image's) and its races (test course 0x44306B, finish sequence
//! 0x442FEF, credits 0x443104, last replay 0x4430C0), run from the menu loop's next frame.
//!
//! Not here: `var\soptions` (the tab keeps no prefs between runs), and 0x459C1B, which reads a
//! developer's `e:\rally5\bin` file the game does not ship.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::session::Dword;
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

/// `dev_front_end::set_up_race` (0x446994): the race's variables from the settings.
fn set_up_race(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    use crate::race::blocks_store::CAR_BLOCKS;
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let gears = s.gears.load(Relaxed);
    shared.vars.stage_number.set(s.course.load(Relaxed));
    shared.vars.race_car_kinds[0].set(car);
    shared.vars.race_car_kinds[1].set((car + 1) % 0x17);
    // The two blocks' start slots. The set-up figures and tyres stay the player's
    // (`copy_settings_in`; the original set defaults here; owner, 2026-10-30).
    for (block, slot) in CAR_BLOCKS.cars[..2].iter().zip([1, 2]) {
        block.start_slot_value.set(slot);
    }
    for block in &CAR_BLOCKS.cars[..2] {
        block.camera_view.set(4);
    }
    shared.vars.car_colours[0].set(gears);
    shared.vars.car_colours[1].set(gears);
    shared.vars.mip_setting.set(s.mips.load(Relaxed));
    shared.vars.run_kind.set(s.mode.load(Relaxed));
    shared.vars.direction.set(s.direction.load(Relaxed));
    shared.view.race_cars.set(s.cars.load(Relaxed) + 1);
    shared.view.players.set(s.screens.load(Relaxed) + 1);
    shared.vars.race_level.set(s.rally.load(Relaxed));
    shared.vars.race_stage.set(s.stage.load(Relaxed));
    // What a race's start had left of the last, in both blocks: the damage and wear gauges
    // (the neutral gear's wear too, which the network race's clearing leaves).
    for block in &CAR_BLOCKS.cars[..2] {
        crate::net::clear_car_state::clear_car_state(block)?;
        block.gear_wear[1].set_bits(0);
    }
    Ok(())
}

/// `f` between what the menus' own route does round a race (`race::single_rally`): the
/// game's mode changes of `sub_44326D` are the menus' here.
fn round_a_race(race: &Race, f: impl FnOnce() -> Result<(), Refused>) -> Result<(), Refused> {
    let shared = &race.shared;
    // `sub_44326D`'s, which the race's sound objects are made from (without them, a race
    // with no engine heard); the menus' own put back after.
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let settings = &shared.state.settings;
    let menus: [(&Dword, u32); 6] = [
        (&settings.car_kind_current, car),
        (&settings.car_kind_first, (car + 1) % 0x17),
        (&settings.run_kind, s.mode.load(Relaxed)),
        (&settings.race_mode, s.screens.load(Relaxed) + 1),
        (&settings.cars[0].codriver_sex, 0),
        (&settings.cars[1].codriver_sex, 1),
    ];
    let kept = menus.map(|(cell, _)| cell.get());
    // The player's HUD, keys and set-up, before the tab's own over them.
    crate::race::single_rally::player_settings_in()?;
    for (cell, v) in menus {
        cell.set(v);
    }
    use crate::race::single_rally::{Call, step};
    use crate::race::{race_end, race_shell, stage_free};
    step(Call::StopMusic)?;
    step(Call::SetRaceScreenMode)?;
    step(Call::ShowLoadingPicture)?;
    step(Call::SetUpRaceSoundObjects)?;
    step(Call::SetUpRaceSounds)?;
    stage_free::render3d_tidy(race)?;
    stage_free::objects_tidy(race)?;
    race_end::release_textures();
    shared.switches.mask_kind.set(0);
    race_shell::clear_player_absences();
    f()?;
    step(Call::StopMusic)?;
    step(Call::SetMenuScreenMode)?;
    step(Call::ShowLoadingPicture)?;
    step(Call::ClearRaceSoundObjects)?;
    step(Call::ApplySettings)?;
    step(Call::SetUpEngine)?;
    step(Call::SetUpFrontEndScene)?;
    step(Call::SetUpEngine)?;
    step(Call::TakeKey)?;
    for ((cell, _), v) in menus.into_iter().zip(kept) {
        cell.set(v);
    }
    Ok(())
}

/// `dev_front_end::test_course` (0x44306B) and `dev_front_end::run_race` (0x443F96).
fn test_course() -> Result<(), Refused> {
    let race = race();
    use crate::race::{race_end, race_shell, replay, stage_loading};
    round_a_race(race, || {
        race.course.numbers.replay_mode.set(1);
        race.shared.tables.noise_seed.set(0x1234_5678);
        race.physics.split_tables.arcade_kind.set(2);
        race.shared.switches.editor_mode.set(0);
        // Free play leaves the car where it was: on another course, or with no race
        // yet, that is nowhere, and the first search ends the game (seen 2026-10-30).
        let course = SETTINGS.course.load(Relaxed);
        crate::debug::screens::place_free_play(
            !crate::debug::screens::raced() || race.shared.vars.stage_number.bits() != course,
        );
        // The player's settings, as a race from the menus; then this tab's.
        crate::race::settings_copy::copy_settings_in(race)?;
        set_up_race(race)?;
        crate::race::race_cars::set_up_arcade_cars(race)?;
        race_shell::set_start_states(race)?;
        stage_loading::load_stage(race)?;
        race_end::record_replay_header(race);
        replay::open_replay(race)?;
        race.shared.view.after_credits.set(0);
        crate::drive::play_race::play_race(race)?;
        replay::close_replay(race)?;
        race_end::tidy_everything(race)
    })
}

/// `sub_442FEF`: the finish sequence, for the course's index (0 .. 5) and the car.
fn finish_sequence() -> Result<(), Refused> {
    let race = race();
    round_a_race(race, || {
        let s = &SETTINGS;
        let car = s.car.load(Relaxed);
        race.course.numbers.replay_mode.set(1);
        let results = &race.shared.state.results;
        race.replay.finish.no_place.set(1);
        results
            .stage_index
            .set(s.course.load(Relaxed).saturating_sub(1).min(5));
        results.finish_kind.set(car);
        results.finish_player.set(0x45);
        results.finished.set(car & 1);
        let result = crate::drive::finish_race::finish_race(race);
        race.replay.finish.no_place.set(0);
        result
    })
}

/// `sub_443104`, without its fonts (the menus' are up): the credits' race.
fn credits() -> Result<(), Refused> {
    let race = race();
    round_a_race(race, || {
        crate::race::sound_tables::silence_race_sounds()?;
        crate::race::credits_race::credits_race(race)
    })
}

/// `sub_4430C0`: the last race's replay (nothing happens without one).
fn last_replay() -> Result<(), Refused> {
    let race = race();
    let shared = &race.shared;
    let course = &race.course;
    round_a_race(race, || {
        course.numbers.replay_mode.set(2);
        shared.view.player_options[0].set(u32::MAX);
        shared.view.player_options[1].set(u32::MAX);
        let result =
            crate::race::replay_mode::replay_mode(race, crate::race::replay_viewer::Mode::LastRace);
        course.numbers.replay_mode.set(0);
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
    pub fn tab(race: &Race, ui: &Ui) {
        let s = &SETTINGS;
        number(ui, "Course", &s.course, 99);
        let course = s.course.load(Relaxed) as usize;
        let present = race.shared.switches.course_present[course].get() != 0;
        ui.same_line();
        if ui.button("Find the courses")
            && let Err(e) = crate::race::find_present_courses::find_present_courses(race)
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
        let name = race.physics.car_specs.cars[s.car.load(Relaxed) as usize]
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
