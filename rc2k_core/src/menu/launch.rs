//! A race started from the command line: the switches and the launch file read into a
//! [`Launch`], and [`Launched`], a [`Game`] around the menus' own one that answers the car and
//! stage choices the launch made the first time the flow asks for them. Everything else the flow
//! runs as the menus do; after the race it goes on as if the player had chosen in the menus.
//!
//! Switches (each also `--name=value`; a launch file holds the same names as `name = value`
//! lines, `#` starting a comment):
//! - `--stage <name>` or `--stage <rally>-<stage>` (1-based, the stage screen's order): the
//!   stage; a launch needs it.
//! - `--stage-group 1|2`: the stage screen's group (1 unless given).
//! - `--car <name or number>`, `--car2 <...>` (the second player's; the first's unless given):
//!   a car of the list the car screen shows, by its name as the game shows it or its place in
//!   that list (1-based). The player's own car unless given.
//! - `--gearbox <name or number>`: one of the game's three gearbox choices (1-based).
//! - `--players 1|2`: one player, or two on a split screen.
//! - `--seed <n>` (decimal or `0x` hex): the random numbers start from it.
//! - `--then menus|quit`: after the race, its results and the menus (the default), or the
//!   program ends.
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
    /// The stage screen's group, 0-based.
    pub group: u32,
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
const VALUED: [&str; 9] = [
    "stage",
    "stage-group",
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

impl Launch {
    /// The launch the program's words (after its own name) ask for, if any; `read_file` reads a
    /// `--launch` file's text.
    pub fn from_args<S: AsRef<str>>(
        args: &[S],
        read_file: &mut dyn FnMut(&str) -> Result<String, String>,
    ) -> Result<Option<Launch>, LaunchError> {
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
        let stage = match all.last("stage") {
            Some(v) => stage(v)?,
            None => return Err(LaunchError("a launch needs --stage".into())),
        };
        let players = match all.last("players") {
            Some(v) => one_of("players", v, 1, 2)?,
            None => 1,
        };
        let car = all.last("car").map(Choice::read);
        let car2 = all.last("car2").map(Choice::read).or_else(|| car.clone());
        let then = match all.last("then").map(str::trim) {
            None | Some("menus") => Then::Menus,
            Some("quit") => Then::Quit,
            Some(other) => {
                return Err(LaunchError(format!(
                    "--then: `{other}` is neither `menus` nor `quit`"
                )));
            }
        };
        Ok(Some(Launch {
            stage,
            group: match all.last("stage-group") {
                Some(v) => one_of("stage-group", v, 1, 2)? - 1,
                None => 0,
            },
            cars: [car, car2],
            gearbox: all.last("gearbox").map(Choice::read),
            players,
            seed: all.last("seed").map(|v| number("seed", v)).transpose()?,
            then,
        }))
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

    fn read(args: &[&str]) -> Result<Option<Launch>, LaunchError> {
        Launch::from_args(args, &mut |_| Err("no file".into()))
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
            (l.players, l.seed, l.then, l.group),
            (2, Some(16), Then::Quit, 0)
        );
    }

    #[test]
    fn stage_by_place_and_bad_values() {
        let l = read(&["--stage", "4-5", "--stage-group", "2"])
            .unwrap()
            .unwrap();
        assert_eq!((l.stage, l.group), (Stage::Place { rally: 4, stage: 5 }, 1));
        assert!(read(&["--players", "3", "--stage", "x"]).is_err());
        assert!(read(&["--car", "1"]).is_err(), "no stage");
        assert!(read(&["--stage"]).is_err(), "no value");
        assert!(read(&["--stage", "x", "--then", "later"]).is_err());
    }

    #[test]
    fn file_then_switches() {
        let text = "# a held race\nstage = langton\ncar = 2 # the second\nthen = quit\n";
        let l = Launch::from_args(&["--launch", "a.txt", "--car", "5"], &mut |name| {
            assert_eq!(name, "a.txt");
            Ok(text.into())
        })
        .unwrap()
        .unwrap();
        assert_eq!(l.stage, Stage::Name("langton".into()));
        assert_eq!(l.cars[0], Some(Choice::Number(5)));
        assert_eq!(l.then, Then::Quit);
        let bad = Launch::from_args(&["--launch", "b"], &mut |_| Ok("weather = rain".into()));
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
