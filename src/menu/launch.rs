//! A race or a replay started from the command line (`rc2k_core::menu::launch` reads the
//! switches and the launch file). A race: the front end runs the menus' own single rally with the car and stage
//! choices answered from the launch, then goes on as if the player had made them.
//!
//! What the answers leave is what the screens leave when a choice is confirmed: the car
//! screen the shown car (its place in the handed list) and the gearbox; the stage screen the
//! stage's group, rally and number, the map's mode and the race's stage and direction. A name or number
//! the game does not have ends the program with a message naming what it has, before the
//! race loads.
//!
//! A replay: the replay viewer on a saved replay's slot or on a named file, as the replays
//! menu's item runs it once a slot is chosen on its loading screen; the replay plays at once.
//! A slot with no replay, or a file that is not one, ends the program with a message listing
//! the saved replays.

use crate::errors::{BadInput, Refused};
use crate::menu::flow_game::Game;
use crate::menu::lists_store::CarListParts;
use crate::menu::player::player;
use crate::owners::Owners;
use crate::records::stages_store::TrackField;
use rc2k_core::menu::flow::CarList;
use rc2k_core::menu::launch::{
    Answers, Asked, Choice, Launch, LaunchError, Launched, Pick, REPLAY_SLOTS, Replay,
    ReplayLaunch, Stage, Then,
};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// The launch the program was started with, read once.
pub fn asked() -> Option<&'static Asked> {
    static ASKED: OnceLock<Option<Asked>> = OnceLock::new();
    ASKED
        .get_or_init(|| {
            let args: Vec<String> = std::env::args().skip(1).collect();
            let mut read = |path: &str| std::fs::read_to_string(path).map_err(|e| e.to_string());
            match Asked::from_args(&args, &mut read) {
                Ok(launch) => launch,
                Err(LaunchError(why)) => refuse(&why),
            }
        })
        .as_ref()
}

/// The program ends: a launch it cannot run, said in the log and in a message box (none in
/// the test builds, which run without a screen to answer it).
fn refuse(why: &str) -> ! {
    let text = format!("What the command line asked for cannot start:\n{why}");
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
        Some(asked) if !RAN.swap(true, Ordering::Relaxed) => match asked {
            Asked::Race(launch) => run(entry.owners, launch),
            Asked::Replay(launch) => run_replay(entry.owners, launch),
        },
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

/// The replay viewer on the launch's replay, run from the main menu's set-up; after it the main
/// menu goes on, or the program ends.
fn run_replay(o: Owners<'static>, launch: &ReplayLaunch) -> Result<(), Refused> {
    use crate::race::replay_viewer::{Viewed, launched_replay};
    let slot = find_replay(o, &launch.replay)?;
    crate::logfile::program_log::log(&format!("menu::launch: {launch:?}"));
    let quit = launch.then == Then::Quit;
    let viewed = launched_replay(o.race, o, slot, quit)?;
    if viewed == Viewed::Nothing {
        refuse(&format!(
            "replay {}: it could not be read through",
            said(&launch.replay)
        ));
    }
    if quit {
        crate::logfile::program_log::log("menu::launch: the replay is over; leaving (--then quit)");
        crate::standalone::leave_on_close(o);
    }
    Ok(())
}

/// The launch's replay as its messages name it.
fn said(replay: &Replay) -> String {
    match replay {
        Replay::Slot(slot) => format!("{slot} ({})", saved_replay_text(*slot)),
        Replay::File(file) => file.clone(),
    }
}

/// A saved replay's file as the game names it.
fn saved_replay(slot: u32) -> Vec<u8> {
    format!("savedata\\replay\\replay{slot:02}").into_bytes()
}

fn saved_replay_text(slot: u32) -> String {
    crate::files::ansi_text(&saved_replay(slot)).to_string()
}

/// The name a replay file holds, or why the file is not a replay the viewer can open: its head
/// and its index are looked at, as the viewer does when it opens one.
fn replay_name(path: &[u8]) -> Result<Vec<u8>, String> {
    use rc2k_core::race::replay::INDEX_MAGIC;
    use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};
    use rc2k_core::saves::kept;
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(crate::files::host_path(path))
        .map_err(|e| format!("no such file to read ({e})"))?;
    let mut head = [0u8; 0x30];
    let not_one = "not a replay of this game (no replay head)";
    file.read_exact(&mut head)
        .map_err(|_| not_one.to_string())?;
    let name = kept::slot_name(&head, kept::REPLAY, kept::REPLAY_VERSION).ok_or(not_one)?;
    let cut_short = "a replay cut short (no index at its end)";
    let mut magic = [0u8; 4];
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    if length < u64::from(HEADER_LEN) + u64::from(INDEX_LEN) {
        return Err(cut_short.into());
    }
    file.seek(SeekFrom::End(-i64::from(INDEX_LEN)))
        .and_then(|_| file.read_exact(&mut magic))
        .map_err(|e| e.to_string())?;
    if u32::from_le_bytes(magic) != INDEX_MAGIC {
        return Err(cut_short.into());
    }
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    Ok(name[..end].to_vec())
}

/// The viewer's slot for the launch's replay: the saved one's own, or the named file's (its
/// path kept for the viewer). One that is not there or not a replay ends the program with the
/// saved replays listed.
fn find_replay(o: Owners<'static>, replay: &Replay) -> Result<u32, Refused> {
    let path = match replay {
        Replay::Slot(slot) => saved_replay(*slot),
        Replay::File(file) => crate::files::ansi_bytes(file),
    };
    if let Err(why) = replay_name(&path) {
        let saved: Vec<String> = (0..REPLAY_SLOTS)
            .filter_map(|slot| {
                let name = replay_name(&saved_replay(slot)).ok()?;
                Some(format!("{slot} {}", crate::files::ansi_text(&name)))
            })
            .collect();
        let listed = if saved.is_empty() {
            "there are no saved replays (savedata\\replay)".to_string()
        } else {
            format!("the saved replays, by slot:\n{}", saved.join("\n"))
        };
        refuse(&format!("replay {}: {why}\n{listed}", said(replay)));
    }
    Ok(match replay {
        Replay::Slot(slot) => *slot,
        Replay::File(_) => {
            o.race
                .replay
                .given_file
                .set(path)
                .map_err(|_| BadInput("a replay file named twice"))?;
            rc2k_core::race::viewer::GIVEN_FILE
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
/// The stage screen's two sets (its groups): the simulation's and the arcade's, which runs each
/// stage the other way.
const SETS: u32 = 2;
const STAGES: u32 = 6;

/// A name the launch matches: its English one by the game's text number, else the game's text,
/// so that a launch reads the same whatever language the game runs in.
fn name(o: Owners<'static>, id: u32) -> Result<Vec<u8>, BadInput> {
    match rc2k_core::menu::launch::english_name(id) {
        Some(english) => Ok(english.as_bytes().to_vec()),
        None => text(o, id),
    }
}

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
        let name = crate::files::ansi_text(name.as_deref().unwrap_or_default());
        if label.is_empty() {
            name.to_string()
        } else {
            format!("{label} {name}")
        }
    };
    which.into_iter().map(line).collect::<Vec<_>>().join("\n")
}

/// The launch's stage on the screen's group, by name or by place.
fn find_stage(o: Owners<'static>, launch: &Launch) -> Result<Place, Refused> {
    let stages = &o.system.records().stage_tables;
    let group = match &launch.stage_set {
        None => 0,
        Some(choice) => {
            let mut sets = Vec::new();
            for set in 0..SETS {
                let title = crate::screens::stage_panel::group_title(set)?;
                sets.push(((set + 1).to_string(), Some(name(o, title)?)));
            }
            picked(choice, "stage set", &sets) as u32
        }
    };
    let mut places = Vec::new();
    let mut names = Vec::new();
    for sub in 0..RALLIES {
        let table = stages.group_event_table(group, sub)?;
        for number in 1..=STAGES {
            // The stage's name is its track's (the record's own is the event's title).
            let name = || -> Result<Vec<u8>, Refused> {
                let record = crate::records::stages_store::row(stages, table, number)?;
                let track = crate::records::trakpak_entry::track_of(stages, record)?;
                Ok(name(o, track.name(stages)?)?)
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
            &format!("stage (set {})", group + 1),
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
    /// The cars of `list` (number, name), up to its end mark.
    fn cars_of(&self, list: CarList) -> Result<Vec<(u32, Vec<u8>)>, Refused> {
        let menus = self.o.menus;
        let mut cars = Vec::new();
        for n in 0..0x100 {
            let e = menus.lists.car_entry(CarListParts::of(menus), list, n)?;
            if e == u32::MAX {
                break;
            }
            let d = crate::menu::cars_store::description(&menus.cars.table, e)?.get();
            cars.push((d.car, name(self.o, d.name)?));
        }
        if cars.is_empty() {
            return Err(BadInput("no cars in a list of cars").into());
        }
        Ok(cars)
    }

    /// The current player's car (by the launch, else the player's own) as its place in a list,
    /// and its gearbox; the list it is in: the handed one, or for a car by name not in it, the
    /// "Unlock Cars" cheat's list (every car), as the car screen shows with the cheat on.
    fn choose_car(&self, handed: CarList) -> Result<CarList, Refused> {
        let menus = self.o.menus;
        let current = menus.screens.event.current_player.get();
        let record = player(menus, current).ok_or(BadInput("a current player past the four"))?;
        let cars = self.cars_of(handed)?;
        let seat = (current as usize).min(1);
        let labelled = |cars: &[(u32, Vec<u8>)]| -> Vec<(String, Option<Vec<u8>>)> {
            cars.iter()
                .map(|(_, name)| (String::new(), Some(name.clone())))
                .collect()
        };
        let what = format!("car for player {}", seat + 1);
        let (list, shown) = match &self.launch.cars[seat] {
            None => {
                let numbers: Vec<u32> = cars.iter().map(|(car, _)| *car).collect();
                (
                    handed,
                    rc2k_core::menu::cars::find_car(&numbers, record.get().car_number),
                )
            }
            Some(given) => {
                // Cars go by name alone: a number is a part of one ("106"), as a car's place
                // in a list changes with what is unlocked.
                let choice = &match given {
                    Choice::Number(n) => Choice::Name(n.to_string()),
                    by_name => by_name.clone(),
                };
                let names = labelled(&cars);
                let shown: Vec<Option<&[u8]>> = names.iter().map(|(_, n)| n.as_deref()).collect();
                match (choice.pick(&shown), choice) {
                    (Pick::None, Choice::Name(_)) => {
                        let all = labelled(&self.cars_of(CarList::Cheat)?);
                        let at = picked(choice, &what, &all) as u32;
                        crate::logfile::program_log::log(&format!(
                            "menu::launch: {what}: not in the list shown; taken from every car's (the Unlock Cars list)"
                        ));
                        (CarList::Cheat, at)
                    }
                    _ => (handed, picked(choice, &what, &names) as u32),
                }
            }
        };
        menus.screens.carousel.front.set(shown);
        menus.screens.event.shown.car.set(shown);
        if let Some(choice) = &self.launch.gearbox {
            let gearing = self.gearbox(choice)?;
            record.update(|p| p.gearing = gearing);
        }
        menus.screens.flow.shown_gearing.set(record.get().gearing);
        Ok(list)
    }

    /// The gearbox choice's value, 0 .. 2, by its name as the car screen shows it or its number.
    fn gearbox(&self, choice: &Choice) -> Result<u32, Refused> {
        let mut names = Vec::new();
        for value in 0..GEARBOXES {
            names.push((
                (value + 1).to_string(),
                Some(name(self.o, GEARBOX_STRINGS + value)?),
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
        // The direction: the arcade's set runs each stage the other way (the screen writes it as
        // it draws the chosen stage, as it does the stage).
        self.o
            .race
            .shared
            .state
            .settings
            .direction
            .set(self.stage.group);
        Ok(())
    }
}

/// The car screen's gearbox names: strings 0x21F on, one per value.
const GEARBOX_STRINGS: u32 = 0x21F;
const GEARBOXES: u32 = 3;

impl Answers<Game<'static>> for GameAnswers<'_> {
    fn car(&mut self, game: &mut Game<'static>) -> Result<(), Refused> {
        let handed = game
            .handed_cars()
            .ok_or(BadInput("a car choice with no list handed"))?;
        let list = self.choose_car(handed)?;
        if list != handed {
            rc2k_core::menu::flow::Game::hand_cars(game, list);
        }
        Ok(())
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
