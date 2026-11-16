//! A race or a replay started from the command line: the switches and the launch file read into
//! an [`Asked`] (a race's [`Launch`] or a [`ReplayLaunch`]), and [`Launched`], a [`Game`] around the menus' own one that answers the car and
//! stage choices the launch made the first time the flow asks for them. Everything else the flow
//! runs as the menus do; after the race it goes on as if the player had chosen in the menus.
//!
//! Switches (each also `--name=value`; a launch file holds the same names as `name = value`
//! lines, `#` starting a comment):
//! - `--stage <name>` or `--stage <rally>-<stage>` (1-based, the stage screen's order): the
//!   stage; a launch needs it.
//! - `--stage-set <name or number>`: the stage screen's set (its group switch: the simulation's
//!   or the arcade's, by the game's name for it or 1 / 2); the first unless given.
//! - `--car <name>`, `--car2 <name>` (the second player's; the first's unless given):
//!   a car of the list the car screen shows, by its name as the game shows it or its place in
//!   that list (1-based); a car by name not in it is taken from every car's list, as the
//!   "Unlock Cars" cheat shows it. The player's own car unless given.
//! - `--gearbox <name or number>`: one of the game's three gearbox choices (1-based).
//! - `--players 1|2`: one player, or two on a split screen.
//! - `--seed <n>` (decimal or `0x` hex): the random numbers start from it.
//! - `--replay <slot or file>`: a replay instead of a race, played in the replay viewer as one
//!   chosen in the replays menu is: a saved replay's slot (0 .. 99, the number at the end of its
//!   file's name, `savedata\replay\replayNN`; the menu's pages count them the same way, twenty
//!   a page from 0), or a replay file's path. No race switch goes with it.
//! - `--then menus|quit`: after the race, its results and the menus (the default), or the
//!   program ends. After a replay: the viewer stays until it is left, then the menus; or the
//!   program ends when the replay has played.
//! - `--launch <file>`: the rest from a file; switches on the command line come after it and
//!   win.
//!
//! Switches of other parts (`--load-old-options`, the original's `-a`, `-c`, ..) are left alone.

use super::flow::{Call, CarList, Game, Picture, Var};
use super::leaving::Switch;

/// A choice given by name (as the game shows it, in its language) or by number (1-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    Name(String),
    Number(u32),
}

impl Choice {
    fn read(value: &str) -> Choice {
        match value.trim().parse::<u32>() {
            Ok(n) => Choice::Number(n),
            Err(_) => Choice::Name(value.trim().to_string()),
        }
    }

    /// Letters and digits without case: what names are compared by, so that
    /// `hamiltons-folly` names "Hamilton's Folly".
    fn fold(bytes: impl Iterator<Item = u8>) -> Vec<u8> {
        bytes
            .filter(u8::is_ascii_alphanumeric)
            .map(|b| b.to_ascii_lowercase())
            .collect()
    }

    /// Which of `shown` (names as the game draws them; `None` where there is none) this choice
    /// names: by number its place (1-based); by name the one named whole, else the one name it
    /// is a part of.
    pub fn pick(&self, shown: &[Option<&[u8]>]) -> Pick {
        let name = match self {
            // A number is a place; one past the list is a part of a name ("106").
            Choice::Number(n) => match (*n as usize).checked_sub(1) {
                Some(i) if shown.get(i).is_some_and(Option::is_some) => return Pick::Found(i),
                _ => n.to_string().into_bytes(),
            },
            Choice::Name(name) => Self::fold(name.bytes()),
        };
        if name.is_empty() {
            return Pick::None;
        }
        let folded: Vec<Option<Vec<u8>>> = shown
            .iter()
            .map(|s| s.map(|s| Self::fold(s.iter().copied())))
            .collect();
        if let Some(i) = folded.iter().position(|f| f.as_ref() == Some(&name)) {
            return Pick::Found(i);
        }
        let parts: Vec<usize> = folded
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.as_ref()
                    .is_some_and(|f| f.windows(name.len()).any(|w| w == name))
            })
            .map(|(i, _)| i)
            .collect();
        match parts[..] {
            [] => Pick::None,
            [one] => Pick::Found(one),
            _ => Pick::Ambiguous(parts),
        }
    }
}

/// What a choice picked out of a list of names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    Found(usize),
    /// A name part of several: their places.
    Ambiguous(Vec<usize>),
    None,
}

/// The launch's names, in English whatever language the game runs in: by the number of the
/// game's own text for the name (the same in every language). A name not here is matched by
/// the game's text.
const ENGLISH: &[(u32, &str)] = &[
    // The car screen's gearboxes.
    (0x21F, "Manual"),
    (0x220, "Semi-automatic"),
    (0x221, "Automatic"),
    // The cars.
    (0x222, "Citroen Saxo (A6)"),
    (0x223, "Ford Escort RS2000 (A7)"),
    (0x224, "Honda Civic (A6)"),
    (0x225, "Hyundai Coupe Evo II (A7)"),
    (0x226, "Mitsubishi Lancer Evo V (A8)"),
    (0x227, "Nissan Almera (A7)"),
    (0x228, "Nissan Micra Maxi (A5)"),
    (0x229, "Peugeot 106 Maxi (A6)"),
    (0x22A, "Peugeot 206 WRC (A8)"),
    (0x22B, "Proton Compact (A6)"),
    (0x22C, "Proton Wira/Persona (A8)"),
    (0x22D, "Renault Megane (A7)"),
    (0x22E, "Seat Ibiza Cupra Sport (A7)"),
    (0x22F, "Seat Cordoba WRC (A8)"),
    (0x230, "Skoda Felicia (A6)"),
    (0x231, "Skoda Octavia (A7)"),
    (0x232, "Subaru Impreza WRC (A8)"),
    (0x233, "Vauxhall Astra (A7)"),
    (0x234, "Volkswagen Golf GTI Mk IV (A7)"),
    (0x235, "Ford Escort Maxi (A7)"),
    (0x236, "Ford Puma (A5)"),
    (0x237, "Mitsubishi Lancer Evo IV (A8)"),
    (0x238, "Land Rover (A8)"),
    (0x239, "Tractor (A5)"),
    (0x23A, "MF Turbo (A8)"),
    (0x23B, "MF Hothatch (A7)"),
    (0x23C, "Bernie's Dicemobile (A7)"),
    (0x23D, "Max Power Car (A8)"),
    (0x23E, "Skip (A5)"),
    (0x23F, "Turbotater (A8)"),
    (0x240, "Moo 1.8bse Turbo (A8)"),
    (0x241, "Lambaaghini (A6)"),
    (0x242, "Radio Control Car (A7)"),
    (0x243, "Welsh Sheep Wagon (A7)"),
    (0x244, "Local Shop (A6)"),
    // The stage screen's two sets.
    (0x340, "Simulation"),
    (0x341, "Arcade"),
    // The stages.
    (0x3E8, "Clocaenog Mid"),
    (0x3E9, "Penmachno South"),
    (0x3EA, "Myherin"),
    (0x3EB, "Hafren"),
    (0x3EC, "Dyfi"),
    (0x3ED, "Gartheiniog"),
    (0x3EE, "Chirdonhead"),
    (0x3EF, "Falstone"),
    (0x3F0, "Kershope"),
    (0x3F1, "Pundershaw"),
    (0x3F2, "Riccarton"),
    (0x3F3, "Newcastleton"),
    (0x3F4, "Twiglees"),
    (0x3F5, "Yair"),
    (0x3F6, "Cardrona"),
    (0x3F7, "Black Loch"),
    (0x3F8, "Glentrool"),
    (0x3F9, "Ae"),
    (0x3FA, "Moon and Star"),
    (0x3FB, "Bothwell"),
    (0x3FC, "Whitchester"),
    (0x3FD, "Eccles"),
    (0x3FE, "Langton"),
    (0x3FF, "Fogo"),
    (0x400, "Hamilton's Folly"),
    (0x401, "Tyrone's Ditches"),
    (0x402, "Feeney"),
    (0x403, "Parkanaur"),
    (0x404, "Lisnamuck"),
    (0x405, "Tardree"),
    (0x406, "Port Soderick"),
    (0x407, "Ballagyr"),
    (0x408, "Curraghs"),
    (0x409, "Tholt-y-Will"),
    (0x40A, "Injerbreck"),
    (0x40B, "Cringle"),
];

/// The English name the launch matches for the game's text number `id`, if it has one.
pub fn english_name(id: u32) -> Option<&'static str> {
    ENGLISH
        .iter()
        .find(|(text, _)| *text == id)
        .map(|(_, name)| *name)
}

/// The stage: by name, or by its rally and stage (1-based).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stage {
    Name(String),
    Place { rally: u32, stage: u32 },
}

/// What follows the race.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Then {
    /// Its results, then the menus.
    #[default]
    Menus,
    /// The program ends.
    Quit,
}

/// A launch: what the command line and the launch file asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Launch {
    pub stage: Stage,
    /// The stage screen's set (its group).
    pub stage_set: Option<Choice>,
    /// Per player.
    pub cars: [Option<Choice>; 2],
    pub gearbox: Option<Choice>,
    pub players: u32,
    pub seed: Option<u32>,
    pub then: Then,
}

/// A switch or file line that could not be read: the message says which and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchError(pub String);

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The switches that take a value (the next word, or after `=`).
const VALUED: [&str; 10] = [
    "replay",
    "stage",
    "stage-set",
    "car",
    "car2",
    "gearbox",
    "players",
    "seed",
    "then",
    "launch",
];

/// The launch's values as given, before they are checked.
#[derive(Default)]
struct Given {
    values: Vec<(String, String)>,
}

impl Given {
    fn last(&self, key: &str) -> Option<&str> {
        self.values
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// The launch switches among `args` (the program's words after its own name), with each one's
/// value: the next word, or what follows `=`.
fn switches<S: AsRef<str>>(args: &[S]) -> Result<Vec<(String, String)>, LaunchError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let word = args[i].as_ref();
        i += 1;
        let Some(long) = word.strip_prefix("--") else {
            continue;
        };
        let (key, inline) = match long.split_once('=') {
            Some((k, v)) => (k, Some(v.to_string())),
            None => (long, None),
        };
        if !VALUED.contains(&key) {
            continue;
        }
        let value = match inline {
            Some(v) => v,
            None => {
                let v = args
                    .get(i)
                    .ok_or_else(|| LaunchError(format!("--{key} needs a value")))?;
                i += 1;
                v.as_ref().to_string()
            }
        };
        out.push((key.to_string(), value));
    }
    Ok(out)
}

/// A launch file's `name = value` lines.
fn file_lines(text: &str, file: &str) -> Result<Vec<(String, String)>, LaunchError> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            LaunchError(format!("{file}, line {}: not a `name = value` line", n + 1))
        })?;
        let key = key.trim();
        if !VALUED.contains(&key) || key == "launch" {
            return Err(LaunchError(format!(
                "{file}, line {}: no setting `{key}`",
                n + 1
            )));
        }
        out.push((key.to_string(), value.trim().to_string()));
    }
    Ok(out)
}

fn number(key: &str, value: &str) -> Result<u32, LaunchError> {
    let v = value.trim();
    let parsed = match v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16),
        None => v.parse(),
    };
    parsed.map_err(|_| LaunchError(format!("--{key}: `{value}` is not a number")))
}

fn one_of(key: &str, value: &str, low: u32, high: u32) -> Result<u32, LaunchError> {
    let n = number(key, value)?;
    if (low..=high).contains(&n) {
        Ok(n)
    } else {
        Err(LaunchError(format!(
            "--{key}: {n} is not between {low} and {high}"
        )))
    }
}

fn stage(value: &str) -> Result<Stage, LaunchError> {
    let v = value.trim();
    if let Some((rally, stage)) = v.split_once('-')
        && let (Ok(rally), Ok(stage)) = (rally.trim().parse(), stage.trim().parse())
    {
        return Ok(Stage::Place { rally, stage });
    }
    if v.is_empty() {
        return Err(LaunchError("--stage: no stage named".into()));
    }
    Ok(Stage::Name(v.to_string()))
}

/// The replay a launch plays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Replay {
    /// A saved replay by its slot, the number its file's name ends in.
    Slot(u32),
    /// A replay file by its path.
    File(String),
}

/// The saved replays' slots: 0 up to this.
pub const REPLAY_SLOTS: u32 = 100;

/// A launch into the replay viewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayLaunch {
    pub replay: Replay,
    pub then: Then,
}

/// What the command line and the launch file asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Race(Launch),
    Replay(ReplayLaunch),
}

/// The switches that choose a race's parts: none goes with a replay, which holds its own.
const RACE_ONLY: [&str; 7] = [
    "stage",
    "stage-set",
    "car",
    "car2",
    "gearbox",
    "players",
    "seed",
];

fn replay(value: &str) -> Result<Replay, LaunchError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(LaunchError("--replay: no replay named".into()));
    }
    if !v.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(Replay::File(v.to_string()));
    }
    match v.parse::<u32>() {
        Ok(slot) if slot < REPLAY_SLOTS => Ok(Replay::Slot(slot)),
        _ => Err(LaunchError(format!(
            "--replay: {v} is not a slot (0 to {}); a file whose name is a number needs its folder (./{v})",
            REPLAY_SLOTS - 1
        ))),
    }
}

fn then(all: &Given) -> Result<Then, LaunchError> {
    match all.last("then").map(str::trim) {
        None | Some("menus") => Ok(Then::Menus),
        Some("quit") => Ok(Then::Quit),
        Some(other) => Err(LaunchError(format!(
            "--then: `{other}` is neither `menus` nor `quit`"
        ))),
    }
}

impl Asked {
    /// What the program's words (after its own name) ask for, if anything; `read_file` reads a
    /// `--launch` file's text.
    pub fn from_args<S: AsRef<str>>(
        args: &[S],
        read_file: &mut dyn FnMut(&str) -> Result<String, String>,
    ) -> Result<Option<Asked>, LaunchError> {
        let given = switches(args)?;
        let mut all = Given::default();
        if let Some((_, file)) = given.iter().rev().find(|(k, _)| k == "launch") {
            let text =
                read_file(file).map_err(|why| LaunchError(format!("--launch {file}: {why}")))?;
            all.values = file_lines(&text, file)?;
        }
        all.values
            .extend(given.into_iter().filter(|(k, _)| k != "launch"));
        if all.values.is_empty() {
            return Ok(None);
        }
        let Some(named) = all.last("replay") else {
            return Launch::from_given(&all).map(|l| Some(Asked::Race(l)));
        };
        if let Some(other) = RACE_ONLY.iter().find(|key| all.last(key).is_some()) {
            return Err(LaunchError(format!(
                "--replay does not go with --{other}: a replay holds its own stage, cars and random numbers"
            )));
        }
        Ok(Some(Asked::Replay(ReplayLaunch {
            replay: replay(named)?,
            then: then(&all)?,
        })))
    }
}

impl Launch {
    /// The race the given values ask for.
    fn from_given(all: &Given) -> Result<Launch, LaunchError> {
        let stage = match all.last("stage") {
            Some(v) => stage(v)?,
            None => return Err(LaunchError("a launch needs --stage or --replay".into())),
        };
        let players = match all.last("players") {
            Some(v) => one_of("players", v, 1, 2)?,
            None => 1,
        };
        let car = all.last("car").map(Choice::read);
        let car2 = all.last("car2").map(Choice::read).or_else(|| car.clone());
        Ok(Launch {
            stage,
            stage_set: all.last("stage-set").map(Choice::read),
            cars: [car, car2],
            gearbox: all.last("gearbox").map(Choice::read),
            players,
            seed: all.last("seed").map(|v| number("seed", v)).transpose()?,
            then: then(all)?,
        })
    }
}

/// The log's path among the program's words (after its own name): the first that is not a
/// switch, a switch's value or a program (`.exe`, the old `rc2k.exe RAL.EXE [log]`).
pub fn log_path<S: AsRef<str>>(args: &[S]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        let word = args[i].as_ref();
        i += 1;
        if let Some(long) = word.strip_prefix("--") {
            if !long.contains('=') && VALUED.contains(&long) {
                i += 1;
            }
            continue;
        }
        if word.starts_with('-') || word.to_ascii_lowercase().ends_with(".exe") {
            continue;
        }
        return Some(word.to_string());
    }
    None
}

/// How a launch answers the flow: the game side's, which knows the menus' places.
pub trait Answers<G: Game> {
    /// The car screen's choice for the current player (`Var::Current`), as the screen leaves
    /// it when a car is confirmed.
    fn car(&mut self, game: &mut G) -> Result<(), G::Error>;
    /// The stage screen's choice, as the screen leaves it when a stage is confirmed.
    fn stage(&mut self, game: &mut G) -> Result<(), G::Error>;
    /// Just before the launched race.
    fn before_race(&mut self, game: &mut G) -> Result<(), G::Error>;
    /// Just after it.
    fn after_race(&mut self, game: &mut G) -> Result<(), G::Error>;
}

/// The menus' `Game` with a launch's answers: the first `players` car choices and the first
/// stage choice answered, the first race announced; all else passed through.
pub struct Launched<'g, G, A> {
    game: &'g mut G,
    answers: A,
    cars_left: u32,
    stage_left: bool,
    raced: bool,
}

impl<'g, G: Game, A: Answers<G>> Launched<'g, G, A> {
    pub fn new(game: &'g mut G, answers: A, players: u32) -> Self {
        Self {
            game,
            answers,
            cars_left: players,
            stage_left: true,
            raced: false,
        }
    }
}

impl<G: Game, A: Answers<G>> Game for Launched<'_, G, A> {
    type Error = G::Error;

    fn get(&mut self, v: Var) -> u32 {
        self.game.get(v)
    }
    fn set(&mut self, v: Var, value: u32) {
        self.game.set(v, value);
    }
    fn flag(&mut self, s: Switch) -> bool {
        self.game.flag(s)
    }
    fn set_flag(&mut self, s: Switch, on: bool) {
        self.game.set_flag(s, on);
    }
    fn run(&mut self, c: Call) -> Result<(), Self::Error> {
        match c {
            Call::SelectCarLoop if self.cars_left > 0 => {
                self.cars_left -= 1;
                self.game.set_flag(Switch::LeftByKey, false);
                self.answers.car(self.game)
            }
            Call::SelectStageLoop if self.stage_left => {
                self.stage_left = false;
                self.game.set(Var::Answer, 0);
                self.game.set_flag(Switch::EventLeft, false);
                self.answers.stage(self.game)
            }
            Call::Race { .. } if !self.raced => {
                self.raced = true;
                self.answers.before_race(self.game)?;
                self.game.run(c)?;
                self.answers.after_race(self.game)
            }
            _ => self.game.run(c),
        }
    }
    fn next_record_kind(&mut self) -> Result<u32, Self::Error> {
        self.game.next_record_kind()
    }
    fn load_picture(&mut self, p: Picture) -> Result<bool, Self::Error> {
        self.game.load_picture(p)
    }
    fn take_key(&mut self) -> Result<Option<u8>, Self::Error> {
        self.game.take_key()
    }
    fn wait_or_key(&mut self, ms: u32) -> Result<Option<u8>, Self::Error> {
        self.game.wait_or_key(ms)
    }
    fn stream_playing(&mut self) -> Result<bool, Self::Error> {
        self.game.stream_playing()
    }
    fn hand_cars(&mut self, list: CarList) {
        self.game.hand_cars(list);
    }
    fn fatal(&mut self, code: u32, message: &'static str) -> Self::Error {
        self.game.fatal(code, message)
    }
}

#[cfg(test)]
mod tests {
    use super::super::flow::log::Log;
    use super::*;

    fn race(asked: Result<Option<Asked>, LaunchError>) -> Result<Option<Launch>, LaunchError> {
        asked.map(|a| {
            a.map(|a| match a {
                Asked::Race(launch) => launch,
                Asked::Replay(replay) => panic!("a replay: {replay:?}"),
            })
        })
    }

    fn read(args: &[&str]) -> Result<Option<Launch>, LaunchError> {
        race(Asked::from_args(args, &mut |_| Err("no file".into())))
    }

    #[test]
    fn replay_by_slot_or_file() {
        let read = |args: &[&str]| Asked::from_args(args, &mut |_| Ok("replay = 7 # held".into()));
        let replay = |replay, then| Ok(Some(Asked::Replay(ReplayLaunch { replay, then })));
        assert_eq!(
            read(&["--replay", "0"]),
            replay(Replay::Slot(0), Then::Menus)
        );
        assert_eq!(
            read(&["--replay=99", "--then", "quit"]),
            replay(Replay::Slot(99), Then::Quit)
        );
        assert_eq!(
            read(&["--replay", "held/replay00"]),
            replay(Replay::File("held/replay00".into()), Then::Menus)
        );
        assert_eq!(
            read(&["--replay", "./12"]),
            replay(Replay::File("./12".into()), Then::Menus)
        );
        assert_eq!(
            read(&["--launch", "a.txt"]),
            replay(Replay::Slot(7), Then::Menus)
        );
        assert!(
            read(&["--replay", "100"])
                .unwrap_err()
                .0
                .contains("0 to 99")
        );
        assert!(read(&["--replay"]).is_err(), "no value");
        for race_only in ["--stage", "--car", "--seed", "--players"] {
            let refused = read(&["--replay", "3", race_only, "1"]).unwrap_err();
            assert!(refused.0.contains(race_only), "{refused}");
        }
    }

    #[test]
    fn no_switches_no_launch() {
        assert_eq!(read(&["-a", "--load-old-options", "my.log"]), Ok(None));
    }

    #[test]
    fn switches_read() {
        let l = read(&[
            "--stage",
            "Hamiltons Folly",
            "--car=3",
            "--players",
            "2",
            "--seed",
            "0x10",
            "--then",
            "quit",
            "-a",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(l.stage, Stage::Name("Hamiltons Folly".into()));
        assert_eq!(l.cars, [Some(Choice::Number(3)), Some(Choice::Number(3))]);
        assert_eq!(
            (l.players, l.seed, l.then, l.stage_set),
            (2, Some(16), Then::Quit, None)
        );
    }

    #[test]
    fn stage_by_place_and_bad_values() {
        let l = read(&["--stage", "4-5", "--stage-set", "arcade"])
            .unwrap()
            .unwrap();
        assert_eq!(
            (l.stage, l.stage_set),
            (
                Stage::Place { rally: 4, stage: 5 },
                Some(Choice::Name("arcade".into()))
            )
        );
        assert!(read(&["--players", "3", "--stage", "x"]).is_err());
        assert!(read(&["--car", "1"]).is_err(), "no stage");
        assert!(read(&["--stage"]).is_err(), "no value");
        assert!(read(&["--stage", "x", "--then", "later"]).is_err());
    }

    #[test]
    fn file_then_switches() {
        let text = "# a held race\nstage = langton\ncar = 2 # the second\nthen = quit\n";
        let l = race(Asked::from_args(
            &["--launch", "a.txt", "--car", "5"],
            &mut |name| {
                assert_eq!(name, "a.txt");
                Ok(text.into())
            },
        ))
        .unwrap()
        .unwrap();
        assert_eq!(l.stage, Stage::Name("langton".into()));
        assert_eq!(l.cars[0], Some(Choice::Number(5)));
        assert_eq!(l.then, Then::Quit);
        let bad = Asked::from_args(&["--launch", "b"], &mut |_| Ok("weather = rain".into()));
        assert!(bad.unwrap_err().0.contains("line 1"));
    }

    #[test]
    fn names_picked_whole_or_by_a_part() {
        let names: [Option<&[u8]>; 4] = [
            Some(b"Peugeot 106 Maxi (A6)"),
            Some(b"Peugeot 206 WRC (A8)"),
            None,
            Some(b"Hamilton's Folly"),
        ];
        let pick = |s: &str| Choice::read(s).pick(&names);
        assert_eq!(pick("hamiltons-folly"), Pick::Found(3));
        assert_eq!(
            pick("106"),
            Pick::Found(0),
            "past the list: a part of a name"
        );
        assert_eq!(pick("206 wrc"), Pick::Found(1));
        assert_eq!(pick("peugeot"), Pick::Ambiguous(vec![0, 1]));
        assert_eq!(pick("clio"), Pick::None);
        assert_eq!(pick("2"), Pick::Found(1), "a number is a place");
        assert_eq!(
            pick("3"),
            Pick::None,
            "no name there, and no name holds a 3"
        );
        assert_eq!(pick("9"), Pick::None);
    }

    #[test]
    fn english_names_each_once() {
        let mut ids: Vec<u32> = ENGLISH.iter().map(|(id, _)| *id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ENGLISH.len());
        assert_eq!(english_name(0x3FE), Some("Langton"));
        assert_eq!(english_name(1), None);
    }

    #[test]
    fn the_two_lancers_told_apart() {
        let names: Vec<Option<&[u8]>> = [0x226, 0x237]
            .iter()
            .map(|&id| english_name(id).map(str::as_bytes))
            .collect();
        let pick = |s: &str| Choice::read(s).pick(&names);
        assert_eq!(pick("lancer"), Pick::Ambiguous(vec![0, 1]));
        assert_eq!(pick("lancer evo v"), Pick::Found(0));
        assert_eq!(pick("evo iv"), Pick::Found(1));
        assert_eq!(pick("Mitsubishi Lancer Evo IV (A8)"), Pick::Found(1));
    }

    #[test]
    fn log_path_skips_values() {
        assert_eq!(
            log_path(&["--stage", "langton", "RAL.EXE", "my.log"]),
            Some("my.log".into())
        );
        assert_eq!(log_path(&["--stage=langton", "-a"]), None);
    }

    struct Noted;

    impl Answers<Log> for Noted {
        fn car(&mut self, g: &mut Log) -> Result<(), <Log as Game>::Error> {
            g.set(Var::Round, 100);
            Ok(())
        }
        fn stage(&mut self, g: &mut Log) -> Result<(), <Log as Game>::Error> {
            g.set(Var::Round, 200);
            Ok(())
        }
        fn before_race(&mut self, g: &mut Log) -> Result<(), <Log as Game>::Error> {
            g.set(Var::Round, 300);
            Ok(())
        }
        fn after_race(&mut self, g: &mut Log) -> Result<(), <Log as Game>::Error> {
            g.set(Var::Round, 400);
            Ok(())
        }
    }

    #[test]
    fn first_choices_answered_then_the_menus() {
        // The race, then the second stage choice (the real screen) answered yes: the event ends.
        let mut log = Log::default().on("SelectStageLoop", vec![vec![(Var::Answer, 1)]]);
        super::super::rounds::single_rally(&mut Launched::new(&mut log, Noted, 1)).unwrap();
        assert_eq!(log.count("SelectCarLoop"), 0);
        assert_eq!(
            log.count("SelectStageLoop"),
            1,
            "only the second, the screen's"
        );
        let at = |step: &str| log.steps.iter().position(|s| s == step).unwrap();
        assert!(at("Round=100") < at("TakeShownCar"));
        assert!(at("Round=200") < at("Round=300"));
        let race = log
            .steps
            .iter()
            .position(|s| s.starts_with("Race "))
            .unwrap();
        assert!(at("Round=300") < race && race < at("Round=400"));
        assert!(at("Round=400") < at("SelectStageLoop"));
    }

    #[test]
    fn split_screen_answers_both_cars() {
        let mut log = Log::default().on("SelectStageLoop", vec![vec![(Var::Answer, 1)]]);
        super::super::rounds::single_rally_split(&mut Launched::new(&mut log, Noted, 2)).unwrap();
        assert_eq!(log.count("SelectCarLoop"), 0);
        assert_eq!(log.count("Round=100"), 2);
        assert_eq!(log.count("Round=400"), 1);
    }
}
