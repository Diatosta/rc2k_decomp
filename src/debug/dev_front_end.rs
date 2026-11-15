//! The developers' front end: a menu of keys that came before the real menus, to pick a course,
//! a car and the switches and race at once. Here the "Developers" tab has its settings and its
//! races (test course, finish sequence, credits, last replay), run from the menu loop's next
//! frame. Not here: `var\soptions` (the tab keeps no settings between runs) and a developer's
//! `e:\rally5\bin` file.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::scene::owner::Scene;
use crate::session::DwordLike;
use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// The settings' numbers.
struct Settings {
    /// The course, 0 .. 99 (42 at first).
    course: AtomicU32,
    /// The car's kind, 0 .. 0x21.
    car: AtomicU32,
    /// Forwards, backwards.
    direction: AtomicU32,
    /// Manual, automatic, bumper car.
    gears: AtomicU32,
    /// The cars less one.
    cars: AtomicU32,
    /// The screens less one.
    screens: AtomicU32,
    /// Single, simulation, arcade, time trial, editor free play.
    mode: AtomicU32,
    /// Mipmapping.
    mips: AtomicU32,
    /// The rally, 0 .. 5.
    rally: AtomicU32,
    /// The stage.
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
/// no splits: the third is the game's own fatal error 0x80601843.
const FIRST_RACED: usize = 0x15;

const TEST: u32 = 1;
const FINISH: u32 = 2;
const CREDITS: u32 = 3;
const REPLAY: u32 = 4;

/// The race's variables from the settings.
fn set_up_race(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let gears = s.gears.load(Relaxed);
    shared.vars.get().stage_number.set(s.course.load(Relaxed));
    shared.vars.get().race_car_kinds[0].set(car);
    shared.vars.get().race_car_kinds[1].set((car + 1) % 0x17);
    // The two blocks' start slots. The set-up figures and tyres stay the player's
    // (`copy_settings_in`; the developers' front end set defaults here).
    for (block, slot) in race.car_blocks().cars[..2].iter().zip([1, 2]) {
        block.start_slot_value.set(slot);
    }
    for block in &race.car_blocks().cars[..2] {
        block.camera_view.set(4);
    }
    shared.vars.get().car_colours[0].set(gears);
    shared.vars.get().car_colours[1].set(gears);
    shared.vars.get().mip_setting.set(s.mips.load(Relaxed));
    shared.vars.get().run_kind.set(s.mode.load(Relaxed));
    shared.vars.get().direction.set(s.direction.load(Relaxed));
    shared.view.get().race_cars.set(s.cars.load(Relaxed) + 1);
    shared.view.get().players.set(s.screens.load(Relaxed) + 1);
    shared.vars.get().race_level.set(s.rally.load(Relaxed));
    shared.vars.get().race_stage.set(s.stage.load(Relaxed));
    // What a race's start had left of the last, in both blocks: the damage and wear gauges
    // (the neutral gear's wear too, which the network race's clearing leaves).
    for block in &race.car_blocks().cars[..2] {
        crate::net::clear_car_state::clear_car_state(block)?;
        block.gear_wear[1].set(0.0);
    }
    Ok(())
}

/// `f` between what the menus' own route does round a race (`race::single_rally`): the
/// game's mode changes round a test race are the menus' here.
fn round_a_race(
    o: Owners<'static>,
    f: impl FnOnce() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let race = o.race;
    let shared = &race.shared;
    // The game's own mode change, which the race's sound objects are made from (without them, a race
    // with no engine heard); the menus' own put back after.
    let s = &SETTINGS;
    let car = s.car.load(Relaxed);
    let settings = &shared.state.settings;
    let menus: [(&dyn DwordLike, u32); 6] = [
        (&settings.car_kind_current, car),
        (&settings.car_kind_first, (car + 1) % 0x17),
        (&settings.run_kind, s.mode.load(Relaxed)),
        (&settings.race_mode, s.screens.load(Relaxed) + 1),
        (&settings.cars[0].codriver_sex, 0),
        (&settings.cars[1].codriver_sex, 1),
    ];
    let kept = menus.map(|(cell, _)| cell.get());
    // The player's HUD, keys and set-up, before the tab's own over them.
    crate::race::single_rally::player_settings_in(o.menus, o.race)?;
    for (cell, v) in menus {
        cell.set(v);
    }
    use crate::race::single_rally::{Call, step};
    use crate::race::{race_end, race_shell, stage_free};
    step(o, Call::StopMusic)?;
    step(o, Call::SetRaceScreenMode)?;
    step(o, Call::ShowLoadingPicture)?;
    step(o, Call::SetUpRaceSoundObjects)?;
    step(o, Call::SetUpRaceSounds)?;
    stage_free::render3d_tidy(race, &o.scene.scratch)?;
    stage_free::objects_tidy(race, &o.scene.scratch)?;
    race_end::release_textures(o.scene, o.gfx);
    shared.switches.get().mask_kind.set(0);
    race_shell::clear_player_absences(race);
    f()?;
    step(o, Call::StopMusic)?;
    step(o, Call::SetMenuScreenMode)?;
    step(o, Call::ShowLoadingPicture)?;
    step(o, Call::ClearRaceSoundObjects)?;
    step(o, Call::ApplySettings)?;
    step(o, Call::SetUpEngine)?;
    step(o, Call::SetUpFrontEndScene)?;
    step(o, Call::SetUpEngine)?;
    step(o, Call::TakeKey)?;
    for ((cell, _), v) in menus.into_iter().zip(kept) {
        cell.set(v);
    }
    Ok(())
}

fn test_course(o: Owners<'static>) -> Result<(), Refused> {
    use crate::race::{race_end, race_shell, replay, stage_loading};
    let race = o.race;
    round_a_race(o, || {
        race.course.numbers.get().replay_mode.set(1);
        race.shared.tables().noise_seed.set(0x1234_5678);
        race.physics.split_tables().arcade_kind.set(2);
        race.shared.switches.get().editor_mode.set(0);
        // Free play leaves the car where it was: on another course, or with no race
        // yet, that is nowhere, and the first search ends the game.
        let course = SETTINGS.course.load(Relaxed);
        crate::debug::screens::place_free_play(
            !crate::debug::screens::raced() || race.shared.vars.get().stage_number.get() != course,
        );
        // The player's settings, as a race from the menus; then this tab's.
        crate::race::settings_copy::copy_settings_in(race, o.gfx, &o.saves.best_tables)?;
        set_up_race(race)?;
        crate::race::race_cars::set_up_arcade_cars(race)?;
        race_shell::set_start_states(race, &o.net.race_slot)?;
        stage_loading::load_stage(race, o)?;
        race_end::record_replay_header(race);
        replay::open_replay(race, o)?;
        race.shared.view.get().after_credits.set(0);
        crate::drive::play_race::play_race(race, o)?;
        replay::close_replay(race, o.system)?;
        race_end::tidy_everything(race, o)
    })
}

/// The finish sequence, for the course's index (0 .. 5) and the car.
fn finish_sequence(o: Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    round_a_race(o, || {
        let s = &SETTINGS;
        let car = s.car.load(Relaxed);
        race.course.numbers.get().replay_mode.set(1);
        let results = &race.shared.state.results;
        race.replay.finish.get().no_place.set(1);
        results
            .stage_index
            .set(s.course.load(Relaxed).saturating_sub(1).min(5));
        results.finish_kind.set(car);
        results.finish_player.set(0x45);
        results.finished.set(car & 1);
        let result = crate::drive::finish_race::finish_race(race, o);
        race.replay.finish.get().no_place.set(0);
        result
    })
}

/// The credits' race, without its fonts (the menus' are up).
fn credits(o: Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    round_a_race(o, || {
        crate::race::sound_tables::silence_race_sounds(o)?;
        crate::race::credits_race::credits_race(race, o)
    })
}

/// The last race's replay; nothing happens without one.
fn last_replay(o: Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    let shared = &race.shared;
    let course = &race.course;
    round_a_race(o, || {
        course.numbers.get().replay_mode.set(2);
        shared.view.get().player_options[0].set(u32::MAX);
        shared.view.get().player_options[1].set(u32::MAX);
        let result = crate::race::replay_mode::replay_mode(
            race,
            o,
            crate::race::replay_viewer::Mode::LastRace,
        );
        course.numbers.get().replay_mode.set(0);
        result
    })
}

/// A menu's frame: runs what was asked for; true if something ran.
pub fn serve(o: Owners<'static>) -> Result<bool, Refused> {
    let run: fn(Owners<'static>) -> Result<(), Refused> = match WANTED.swap(0, Relaxed) {
        TEST => test_course,
        FINISH => finish_sequence,
        CREDITS => credits,
        REPLAY => last_replay,
        _ => return Ok(false),
    };
    run(o)?;
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
    pub fn tab(race: &'static Race, scene: &'static Scene, ui: &Ui) {
        let s = &SETTINGS;
        number(ui, "Course", &s.course, 99);
        let course = s.course.load(Relaxed) as usize;
        let present = race.shared.switches.get().course_present[course].get() != 0;
        ui.same_line();
        if ui.button("Find the courses")
            && let Err(e) = crate::race::find_present_courses::find_present_courses(race, scene)
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
        let name = race.physics.car_specs().cars[s.car.load(Relaxed) as usize]
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
