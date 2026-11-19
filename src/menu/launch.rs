//! A race or a replay started from the command line (`rc2k_core::menu::launch` reads the
//! switches and the launch file). A race: the front end runs the menus' own event (`--mode`: the
//! single rally, the time trial, the arcade or a new championship) with the car and stage
//! choices answered from the launch, then goes on as if the player had made them.
//!
//! The time trial's service area, a championship's welcome and its screens before the first
//! race are the player's, as in the menus; the launch answers only the car and stage choices.
//! A championship is always a new one: a saved game is only written by the save menu, which a
//! launch never opens, so a launch cannot overwrite a saved championship.
//!
//! What the answers leave is what the screens leave when a choice is confirmed: the car
//! screen the shown car (its place in the handed list) and the gearbox; the stage screen the
//! stage's group, rally and number, the map's mode and the race's stage and direction. A name or number
//! the game does not have ends the program with a message naming what it has, before the
//! race loads.
//!
//! A network game: the front end's own host or join item (`net::start_link`), run from the
//! main menu's set-up with the settings the switches gave (the player's name, the host's
//! address, the port) and the TCP/IP connection taken as the saved one, so that the settings
//! screen and the connection list are not asked. A joined session is the first one listed.
//! The settings the switches replaced are put back when the lobby is left.
//!
//! A replay: the replay viewer on a saved replay's slot or on a named file, as the replays
//! menu's item runs it once a slot is chosen on its loading screen; the replay plays at once.
//! A slot with no replay, or a file that is not one, ends the program with a message listing
//! the saved replays.

use crate::errors::{BadInput, Refused};
use crate::menu::flow_game::Game;
use crate::menu::lists_store::CarListParts;
use crate::menu::name_lists::NAME_LEN;
use crate::menu::player::player;
use crate::owners::Owners;
use crate::records::stages_store::TrackField;
use rc2k_core::menu::flow::CarList;
use rc2k_core::menu::launch::{
    ARCADE_LEVELS, Answers, Asked, Choice, Launch, LaunchError, Launched, Mode, NetLaunch, Pick,
    REPLAY_SLOTS, Replay, ReplayLaunch, Stage, Then,
};
use rc2k_core::menu::starters::{Championship, Seats};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// The launch the program was started with, read once.
pub fn asked() -> Option<&'static Asked> {
    static ASKED: OnceLock<Option<Asked>> = OnceLock::new();
    ASKED
        .get_or_init(|| {
            let args: Vec<String> = crate::platform::os::program_args()[1..].to_vec();
            let mut read = |path: &str| std::fs::read_to_string(path).map_err(|e| e.to_string());
            match Asked::from_args(&args, &mut read) {
                Ok(launch) => launch,
                Err(LaunchError(why)) => refuse(&why),
            }
        })
        .as_ref()
}

/// The program ends: a launch it cannot run, said in the log and on the error dialog (none in
/// the test builds, which run without a screen to answer it).
fn refuse(why: &str) -> ! {
    let text = format!("What the command line asked for cannot start:\n{why}");
    crate::logfile::program_log::log(&format!("menu::launch: {text}"));
    crate::standalone::finish_saying("a launch that cannot start", &text, 1)
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
            Asked::Net(launch) => run_net(entry.owners, launch),
        },
        _ => Ok(()),
    }
}

/// The first car choice(s) of an event whose flow is not the menus' `Game`'s (the arcade, a
/// championship): given the car list the screen would show, the list the car was chosen in,
/// or `None` when the screen is the player's.
pub type CarAnswer<'a> = &'a mut dyn FnMut(CarList) -> Result<Option<CarList>, Refused>;

/// Whether the program ends when the first race is over (a launch's `--then quit` in an event
/// the launch's wrapper does not see the race of).
static QUIT_AFTER_RACE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A race is over (`race::single_rally_race`): the program ends if the launch asked for that
/// after the first race of an arcade game or a championship.
pub fn race_over(o: Owners<'static>) {
    if QUIT_AFTER_RACE.swap(false, Ordering::Relaxed) {
        crate::logfile::program_log::log("menu::launch: the race is over; leaving (--then quit)");
        crate::standalone::leave_on_close(o);
    }
}

/// The event the launch asks for, with the launch's choices, run from the main menu's set-up;
/// after it the main menu goes on.
fn run(o: Owners<'static>, launch: &Launch) -> Result<(), Refused> {
    crate::logfile::program_log::log(&format!("menu::launch: {launch:?}"));
    match launch.mode {
        Mode::Single | Mode::TimeTrial => run_rounds(o, launch),
        Mode::Arcade => run_arcade(o, launch),
        Mode::Championship | Mode::A8 => run_championship(o, launch),
    }
}

/// The single rally or the time trial: the menus' own flow on a `Game` that answers the car and
/// stage choices.
fn run_rounds(o: Owners<'static>, launch: &Launch) -> Result<(), Refused> {
    let stage = find_stage(o, launch)?;
    let answers = GameAnswers {
        o,
        launch,
        stage: Some(stage),
    };
    let players = launch.players;
    Game::new(o).run(|game| {
        let mut launched = Launched::new(game, answers, players);
        match (launch.mode, players) {
            (Mode::TimeTrial, 1) => rc2k_core::menu::rounds::time_trial_one(&mut launched),
            (Mode::TimeTrial, _) => rc2k_core::menu::rounds::time_trial_split(&mut launched),
            (_, 1) => rc2k_core::menu::rounds::single_rally(&mut launched),
            (_, 2) => rc2k_core::menu::rounds::single_rally_split(&mut launched),
            (_, n) => rc2k_core::menu::rounds::single_rally_split_many(&mut launched, n),
        }
    })
}

/// The settings a network launch replaces for its run, as they were.
struct Kept {
    machine_name: [u8; 64],
    host_address: [u8; 64],
    port: u32,
    player_name: Vec<u8>,
}

impl Kept {
    fn take(o: Owners<'static>) -> Result<Kept, Refused> {
        let setup = o.net.setup();
        let names = &o.menus.play.slots.names.player_names;
        Ok(Kept {
            machine_name: setup.machine_name.read(),
            host_address: setup.host_address.read(),
            port: setup.port.get(),
            player_name: names.read_at(0, NAME_LEN, "player 1's name")?,
        })
    }

    fn put_back(&self, o: Owners<'static>) {
        let setup = o.net.setup();
        setup.machine_name.write(&self.machine_name);
        setup.host_address.write(&self.host_address);
        setup.port.set(self.port);
        // The room was read whole above.
        let _ =
            o.menus
                .play
                .slots
                .names
                .player_names
                .write_at(0, &self.player_name, "player 1's name");
    }
}

/// A text as a NUL-padded field of `N` bytes.
fn padded<const N: usize>(text: &str) -> [u8; N] {
    let mut field = [0u8; N];
    field[..text.len()].copy_from_slice(text.as_bytes());
    field
}

/// The network game the launch asked for: the settings set, the host or join item's own
/// function run, the settings put back; after it the main menu goes on, or the program ends.
fn run_net(o: Owners<'static>, launch: &NetLaunch) -> Result<(), Refused> {
    #[cfg(target_os = "emscripten")]
    {
        let _ = (o, launch);
        return Err(BadInput("the web build has no network play").into());
    }
    #[cfg(not(target_os = "emscripten"))]
    run_net_game(o, launch)
}

#[cfg(not(target_os = "emscripten"))]
fn run_net_game(o: Owners<'static>, launch: &NetLaunch) -> Result<(), Refused> {
    crate::logfile::program_log::log(&format!("menu::launch: {launch:?}"));
    let kept = Kept::take(o)?;
    let setup = o.net.setup();
    let mode = match &launch.role {
        rc2k_core::menu::launch::Role::Host => 1,
        rc2k_core::menu::launch::Role::Join(host) => {
            setup.host_address.write(&padded(host));
            2
        }
    };
    if let Some(name) = &launch.name {
        let field = padded::<NAME_LEN>(name);
        o.menus
            .play
            .slots
            .names
            .player_names
            .write_at(0, &field, "player 1's name")?;
        setup.machine_name.write(&padded(name));
    }
    if let Some(port) = launch.port {
        setup.port.set(u32::from(port));
    }
    // The only connection offered is TCP/IP; as the saved one it is taken without the list.
    setup.saved_connection.write(&crate::net::store::SP_TCPIP);
    let result = crate::net::start_link::start_link_launched(o, mode);
    kept.put_back(o);
    result?;
    if launch.then == Then::Quit {
        crate::logfile::program_log::log("menu::launch: the lobby is left; leaving (--then quit)");
        crate::standalone::leave_on_close(o);
    }
    Ok(())
}

/// What an event the wrapper does not see the race of waits for: the seed, and the end after
/// the first race.
fn prepare_unwrapped(launch: &Launch) {
    if let Some(seed) = launch.seed {
        SEED.store(WAITING | u64::from(seed), Ordering::Relaxed);
    }
    QUIT_AFTER_RACE.store(launch.then == Then::Quit, Ordering::Relaxed);
}

/// The answerer of the first `players` car choices.
fn car_answers<'a>(
    answers: &'a GameAnswers<'a>,
    players: u32,
) -> impl FnMut(CarList) -> Result<Option<CarList>, Refused> + 'a {
    let mut left = players;
    move |handed| {
        if left == 0 {
            return Ok(None);
        }
        left -= 1;
        answers.choose_car(handed).map(Some)
    }
}

/// The arcade's level, as its item on the level screen starts it: the screen's first item
/// starts the first level (the game's level 0) and is always open; each of the five after it
/// starts a level (1 .. 5) that its opened mark, set by winning the level before, opens. A level
/// not open is refused here (the item plays its closed sound).
fn run_arcade(o: Owners<'static>, launch: &Launch) -> Result<(), Refused> {
    use rc2k_core::menu::arcade::level_open;
    let (menus, play) = (o.menus, &o.menus.play);
    // The level as the screen counts it: 1 is its start item, 2 .. 6 its level items 1 .. 5.
    let shown = launch.level.unwrap_or(1);
    let answers = GameAnswers {
        o,
        launch,
        stage: None,
    };
    let mut answer = car_answers(&answers, 1);
    if shown == 1 {
        play.arcade.start_level(0);
    } else {
        let level = shown - 1;
        if !level_open(play.cheats.cheats.get(), || {
            play.arcade.opened(level as usize - 1)
        }) {
            let marks: Vec<String> = (0..ARCADE_LEVELS as usize - 1)
                .map(|i| play.arcade.opened(i).to_string())
                .collect();
            refuse(&format!(
                "arcade level {shown} is not open (the opened marks of levels 2 .. {ARCADE_LEVELS}: {}): win the level before it, or turn on the Unlock Arcade cheat",
                marks.join(" ")
            ));
        }
        play.arcade.start_level(level);
    }
    prepare_unwrapped(launch);
    crate::menu::take_player1_units::take_player1_units(menus, o.strings)?;
    crate::menu::arcade_event::arcade_event_with(o, Some(&mut answer))
}

/// A new championship for the players, as the start menu's item starts it.
fn run_championship(o: Owners<'static>, launch: &Launch) -> Result<(), Refused> {
    let (championship, seats) = (
        if launch.mode == Mode::A8 {
            Championship::A8
        } else {
            Championship::First
        },
        if launch.players == 1 {
            Seats::One
        } else {
            Seats::Split
        },
    );
    let answers = GameAnswers {
        o,
        launch,
        stage: None,
    };
    let mut answer = car_answers(&answers, launch.players);
    prepare_unwrapped(launch);
    crate::menu::start_event::start_event_with(
        o,
        &crate::menu::start_event::Starter {
            kind: rc2k_core::menu::starters::Starter {
                championship,
                seats,
            },
        },
        Some(&mut answer),
    )
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
    let name = kept::replay_name(&head).ok_or(not_one)?;
    // The own format's index is longer than the original's.
    let index_len = if head[..4] == rc2k_core::race::replay_v2::MAGIC.to_le_bytes() {
        rc2k_core::race::replay_v2::OWN_INDEX_LEN as u32
    } else {
        INDEX_LEN
    };
    let cut_short = "a replay cut short (no index at its end)";
    let mut magic = [0u8; 4];
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    if length < u64::from(HEADER_LEN) + u64::from(index_len) {
        return Err(cut_short.into());
    }
    file.seek(SeekFrom::End(-i64::from(index_len)))
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
        Replay::Slot(slot) => crate::race::replay_path::file_to_load(*slot, saved_replay(*slot)),
        Replay::File(file) => crate::files::ansi_bytes(file),
    };
    if let Err(why) = replay_name(&path) {
        let saved: Vec<String> = (0..REPLAY_SLOTS)
            .filter_map(|slot| {
                let name = replay_name(&crate::race::replay_path::file_to_load(
                    slot,
                    saved_replay(slot),
                ))
                .ok()?;
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
    let stage = launch
        .stage
        .as_ref()
        .ok_or(BadInput("a launch of a stage choice without a stage"))?;
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
    let at = match stage {
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
    /// The stage the stage screen is answered with (the modes that have one).
    stage: Option<Place>,
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
        let seat = (current as usize).min(rc2k_core::menu::launch::MOST_PLAYERS as usize - 1);
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
        let place = self
            .stage
            .ok_or(BadInput("a stage answered with none chosen"))?;
        let (menus, system) = (self.o.menus, self.o.system);
        let stages = &system.records().stage_tables;
        let chosen = &menus.screens.event.stage;
        chosen.group.set(place.group);
        chosen.sub.set(place.sub);
        chosen.number.set(place.number);
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
        self.o.race.shared.state.settings.direction.set(place.group);
        Ok(())
    }
}

/// The current player's tune put into the car as the service area's choices end up there: a
/// brake balance or steering sensitivity is the car's own value; a tyre, a gear ratio, a ride
/// height or a stiffness that differs from the car's is chosen, its repair flagged, and the
/// service area's own work (`service::apply_repairs`) carried out without showing it.
fn apply_tune(o: Owners<'static>, tune: &rc2k_core::menu::launch::Tune) -> Result<(), Refused> {
    let menus = o.menus;
    use crate::service::pages::{GEAR_RATIO, RIDE_HEIGHT, SUSPENSION_STIFFNESS, TYRE};
    use crate::service::state::{current_player, set_repair_flag};
    if !tune.is_given() {
        return Ok(());
    }
    let player = current_player(menus).ok_or(BadInput("a tune for a player that is not there"))?;
    player.update(|car| {
        if let Some(value) = tune.brake_balance {
            car.brake_balance = value;
        }
        if let Some(value) = tune.steering {
            car.steering_sensitivity = value;
        }
    });
    let car = player.get();
    let mut changed = false;
    for (given, in_car, page) in [
        (tune.gear_ratio, car.gear_ratio, GEAR_RATIO),
        (tune.tyres, car.tyre, TYRE),
        (tune.ride_height, car.ride_height, RIDE_HEIGHT),
        (
            tune.suspension,
            car.suspension_stiffness,
            SUSPENSION_STIFFNESS,
        ),
    ] {
        if let Some(value) = given
            && value != in_car
        {
            page.change.chosen(&menus.service).set(value);
            set_repair_flag(&menus.service, page.item, 1);
            changed = true;
        }
    }
    if changed {
        crate::service::apply_repairs::apply_repairs_unshown(menus)?;
    }
    let car = player.get();
    let tyre = crate::files::ansi_text(&text(o, 0x4B0 + car.tyre)?).to_string();
    crate::logfile::program_log::log(&format!(
        "menu::launch: tune for player {}: tyre {} ({tyre}), gear ratio {}, ride height {}, stiffness {}, brake balance {}, steering {}",
        menus.screens.event.current_player.get() + 1,
        car.tyre,
        car.gear_ratio,
        car.ride_height,
        car.suspension_stiffness,
        car.brake_balance,
        car.steering_sensitivity
    ));
    Ok(())
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

    fn service(&mut self, _game: &mut Game<'static>) -> Result<(), Refused> {
        let tune = &self.launch.tune;
        let o = self.o;
        crate::menu::time_trial_service::time_trial_service_unshown(o.menus, &mut |_| {
            apply_tune(o, tune)
        })
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
