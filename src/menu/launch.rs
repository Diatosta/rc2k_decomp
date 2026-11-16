//! A race started from the command line (`rc2k_core::menu::launch` reads the switches and the
//! launch file): the front end runs the menus' own single rally with the car and stage
//! choices answered from the launch, then goes on as if the player had made them.
//!
//! What the answers leave is what the screens leave when a choice is confirmed: the car
//! screen the shown car (its place in the handed list) and the gearbox; the stage screen the
//! stage's group, rally and number, the map's mode and the race's stage. A name or number
//! the game does not have ends the program with a message naming what it has, before the
//! race loads.

use crate::errors::{BadInput, Refused};
use crate::menu::flow_game::Game;
use crate::menu::lists_store::CarListParts;
use crate::menu::player::player;
use crate::owners::Owners;
use crate::records::stages_store::TrackField;
use rc2k_core::menu::flow::CarList;
use rc2k_core::menu::launch::{Answers, Choice, Launch, LaunchError, Launched, Pick, Stage, Then};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// The launch the program was started with, read once.
pub fn asked() -> Option<&'static Launch> {
    static ASKED: OnceLock<Option<Launch>> = OnceLock::new();
    ASKED
        .get_or_init(|| {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let mut read = |path: &str| std::fs::read_to_string(path).map_err(|e| e.to_string());
            match Launch::from_args(&args, &mut read) {
                Ok(launch) => launch,
                Err(LaunchError(why)) => refuse(&why),
            }
        })
        .as_ref()
}

/// The program ends: a launch it cannot run, said in the log and in a message box (none in
/// the test builds, which run without a screen to answer it).
fn refuse(why: &str) -> ! {
    let text = format!("The race asked for on the command line cannot start:\n{why}");
    crate::logfile::program_log::log(&format!("menu::launch: {text}"));
    #[cfg(not(feature = "remote"))]
    crate::platform::os::message_box(
        crate::platform::os::Icon::Warning,
        b"Rally Championship",
        text.as_bytes(),
    );
    crate::standalone::finish("a launch that cannot start", 1)
}

/// The seed a launch asked for, for the next stage to load (taken once): bit 32 set when one
/// waits.
static SEED: AtomicU64 = AtomicU64::new(0);
const WAITING: u64 = 1 << 32;

/// A stage about to load (`race::load_stage`): the random numbers from the launch's seed, if
/// one waits. The effects' generator from a value of its own made from it, so the two do not
/// run in step.
pub fn stage_loading(race: &crate::race::owner::Race, effects: &crate::effects::owner::Effects) {
    let taken = SEED.swap(0, Ordering::Relaxed);
    if taken & WAITING != 0 {
        let seed = taken as u32;
        reseed(
            &race.shared.random,
            effects,
            seed,
            seed.rotate_left(16) ^ 0x9E37_79B9,
        );
    }
}

/// Both random generators (the game's and the effects') from given states.
pub fn reseed(
    random: &crate::math::next_random::Randoms,
    effects: &crate::effects::owner::Effects,
    game: u32,
    effects_state: u32,
) {
    random.with_random(|r| r.state = game);
    random.seed_streams();
    effects.setup.work.seed.set(effects_state);
}

/// The main menu's set-up: the launch, if one was asked for and has not run yet.
pub fn run_once(entry: &crate::menu::screen_loop::Entry) -> Result<(), Refused> {
    static RAN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    match asked() {
        Some(launch) if !RAN.swap(true, Ordering::Relaxed) => run(entry.owners, launch),
        _ => Ok(()),
    }
}

/// The single rally with the launch's choices, run from the main menu's set-up; after it the
/// main menu goes on.
fn run(o: Owners<'static>, launch: &Launch) -> Result<(), Refused> {
    let stage = find_stage(o, launch)?;
    let answers = GameAnswers { o, launch, stage };
    crate::logfile::program_log::log(&format!("menu::launch: {launch:?}"));
    let players = launch.players;
    Game::new(o).run(|game| {
        let mut launched = Launched::new(game, answers, players);
        match players {
            1 => rc2k_core::menu::rounds::single_rally(&mut launched),
            _ => rc2k_core::menu::rounds::single_rally_split(&mut launched),
        }
    })
}

/// A stage on the stage screen: its group, rally (the screen's sub, 0-based) and number
/// (1-based).
#[derive(Clone, Copy, Debug)]
struct Place {
    group: u32,
    sub: u32,
    number: u32,
}

const RALLIES: u32 = 6;
const STAGES: u32 = 6;

/// A game string's text.
fn text(o: Owners<'static>, id: u32) -> Result<Vec<u8>, BadInput> {
    let mut out = Vec::new();
    crate::strings::append_string_by_id::push_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &mut out,
        id,
    )?;
    Ok(out)
}

/// The one of `names` (labels, and names as the game draws them; `None` where there is none)
/// `choice` picks, or the end: a message saying what `what` it could not find among them.
fn picked(choice: &Choice, what: &str, names: &[(String, Option<Vec<u8>>)]) -> usize {
    let shown: Vec<Option<&[u8]>> = names.iter().map(|(_, n)| n.as_deref()).collect();
    match choice.pick(&shown) {
        Pick::Found(i) => i,
        Pick::Ambiguous(several) => refuse(&format!(
            "{what} {choice:?}: more than one of these:\n{}",
            listing(names, several)
        )),
        Pick::None => not_there(&format!("{what} {choice:?}"), names),
    }
}

/// The end: `what` is not among `names`, which are listed.
fn not_there(what: &str, names: &[(String, Option<Vec<u8>>)]) -> ! {
    let all = (0..names.len()).filter(|&i| names[i].1.is_some()).collect();
    refuse(&format!("{what}: none of these:\n{}", listing(names, all)))
}

/// Lines `label name` for the names at `which`.
fn listing(names: &[(String, Option<Vec<u8>>)], which: Vec<usize>) -> String {
    let line = |i: usize| {
        let (label, name) = &names[i];
        format!(
            "{label} {}",
            crate::files::ansi_text(name.as_deref().unwrap_or_default())
        )
    };
    which.into_iter().map(line).collect::<Vec<_>>().join("\n")
}

/// The launch's stage on the screen's group, by name or by place.
fn find_stage(o: Owners<'static>, launch: &Launch) -> Result<Place, Refused> {
    let stages = &o.system.records().stage_tables;
    let group = launch.group;
    let mut places = Vec::new();
    let mut names = Vec::new();
    for sub in 0..RALLIES {
        let table = stages.group_event_table(group, sub)?;
        for number in 1..=STAGES {
            // The stage's name is its track's (the record's own is the event's title).
            let name = || -> Result<Vec<u8>, Refused> {
                let record = crate::records::stages_store::row(stages, table, number)?;
                let track = crate::records::trakpak_entry::track_of(stages, record)?;
                Ok(text(o, track.name(stages)?)?)
            };
            places.push(Place { group, sub, number });
            names.push((format!("{}-{number}", sub + 1), name().ok()));
        }
    }
    let at = match &launch.stage {
        Stage::Place { rally, stage } => places
            .iter()
            .position(|p| p.sub + 1 == *rally && p.number == *stage)
            .filter(|&i| names[i].1.is_some())
            .unwrap_or_else(|| not_there(&format!("stage {rally}-{stage}"), &names)),
        Stage::Name(n) => picked(
            &Choice::Name(n.clone()),
            &format!("stage (group {})", group + 1),
            &names,
        ),
    };
    Ok(places[at])
}

struct GameAnswers<'l> {
    o: Owners<'static>,
    launch: &'l Launch,
    stage: Place,
}

impl GameAnswers<'_> {
    /// The current player's car (by the launch, else the player's own) as its place in the
    /// handed list, and its gearbox.
    fn choose_car(&self, list: CarList) -> Result<(), Refused> {
        let menus = self.o.menus;
        let current = menus.screens.event.current_player.get();
        let record = player(menus, current).ok_or(BadInput("a current player past the four"))?;
        let entry = |n: u32| menus.lists.car_entry(CarListParts::of(menus), list, n);
        // The list's cars (number, name), up to its end mark.
        let mut cars = Vec::new();
        for n in 0..0x100 {
            let e = entry(n)?;
            if e == u32::MAX {
                break;
            }
            let d = crate::menu::cars_store::description(&menus.cars.table, e)?.get();
            cars.push((d.car, text(self.o, d.name)?));
        }
        if cars.is_empty() {
            return Err(BadInput("no cars in the handed list").into());
        }
        let seat = (current as usize).min(1);
        let shown = match &self.launch.cars[seat] {
            None => {
                let numbers: Vec<u32> = cars.iter().map(|(car, _)| *car).collect();
                rc2k_core::menu::cars::find_car(&numbers, record.get().car_number)
            }
            Some(choice) => {
                let names: Vec<(String, Option<Vec<u8>>)> = cars
                    .iter()
                    .enumerate()
                    .map(|(i, (_, name))| ((i + 1).to_string(), Some(name.clone())))
                    .collect();
                picked(choice, &format!("car for player {}", seat + 1), &names) as u32
            }
        };
        menus.screens.carousel.front.set(shown);
        menus.screens.event.shown.car.set(shown);
        if let Some(choice) = &self.launch.gearbox {
            let gearing = self.gearbox(choice)?;
            record.update(|p| p.gearing = gearing);
        }
        menus.screens.flow.shown_gearing.set(record.get().gearing);
        Ok(())
    }

    /// The gearbox choice's value, 0 .. 2, by its name as the car screen shows it or its number.
    fn gearbox(&self, choice: &Choice) -> Result<u32, Refused> {
        let mut names = Vec::new();
        for value in 0..GEARBOXES {
            names.push((
                (value + 1).to_string(),
                Some(text(self.o, GEARBOX_STRINGS + value)?),
            ));
        }
        Ok(picked(choice, "gearbox", &names) as u32)
    }

    /// The stage screen's confirmed choice: the place, the map on the choice, and the race's
    /// stage (the track entry's, which the stage panel keeps as it is drawn).
    fn choose_stage(&self) -> Result<(), Refused> {
        let (menus, system) = (self.o.menus, self.o.system);
        let stages = &system.records().stage_tables;
        let chosen = &menus.screens.event.stage;
        chosen.group.set(self.stage.group);
        chosen.sub.set(self.stage.sub);
        chosen.number.set(self.stage.number);
        crate::menu::mark_stage_choice::mark_stage_choice(&menus.play.champ_view);
        let selection = crate::screens::draw_stage_records::selection_in(menus, system)?
            .ok_or(BadInput("no stage tables for the choice"))?;
        let track = crate::records::trakpak_entry::track_of(stages, selection.record)?;
        self.o
            .race
            .shared
            .state
            .settings
            .stage
            .set(track.get(stages, TrackField::Stage)?);
        Ok(())
    }
}

/// The car screen's gearbox names: strings 0x21F on, one per value.
const GEARBOX_STRINGS: u32 = 0x21F;
const GEARBOXES: u32 = 3;

impl Answers<Game<'static>> for GameAnswers<'_> {
    fn car(&mut self, game: &mut Game<'static>) -> Result<(), Refused> {
        let list = game
            .handed_cars()
            .ok_or(BadInput("a car choice with no list handed"))?;
        self.choose_car(list)
    }

    fn stage(&mut self, _game: &mut Game<'static>) -> Result<(), Refused> {
        self.choose_stage()
    }

    fn before_race(&mut self, _game: &mut Game<'static>) -> Result<(), Refused> {
        if let Some(seed) = self.launch.seed {
            SEED.store(WAITING | u64::from(seed), Ordering::Relaxed);
        }
        Ok(())
    }

    fn after_race(&mut self, _game: &mut Game<'static>) -> Result<(), Refused> {
        if self.launch.then == Then::Quit {
            crate::logfile::program_log::log(
                "menu::launch: the race is over; leaving (--then quit)",
            );
            crate::standalone::leave_on_close(self.o);
        }
        Ok(())
    }
}
