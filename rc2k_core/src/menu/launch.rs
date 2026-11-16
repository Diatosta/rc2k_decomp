//! A race or a replay started from the command line: the switches and the launch file read into
//! an [`Asked`] (a race's [`Launch`] or a [`ReplayLaunch`]), and [`Launched`], a [`Game`] around the menus' own one that answers the car and
//! stage choices the launch made the first time the flow asks for them. Everything else the flow
//! runs as the menus do; after the race it goes on as if the player had chosen in the menus.
//!
//! Switches (each also `--name=value`; a launch file holds the same names as `name = value`
//! lines, `#` starting a comment):
//! - `--mode <single | time-trial | arcade | championship | a8>`: the kind of event, as the
//!   mode menu starts it (`single`, the single rally, when not given). Each runs its own flow
//!   with the launch's answers; a switch that does not fit the mode is refused with the reason.
//! - `--tyres`, `--gear-ratio`, `--ride-height`, `--suspension`, `--brake-balance`,
//!   `--steering` (time trial only): the car's tune, the service area's six values; the service
//!   area is not shown. Each not given stays as the player's record has it.
//! - `--stage <name>` or `--stage <rally>-<stage>` (1-based, the stage screen's order): the
//!   stage; a single rally and a time trial need it, the arcade's and the championships' stages
//!   come with the event.
//! - `--level <1 .. 6>`: the arcade's level as the level screen shows them (1 unless given: its
//!   start item; a level not yet opened by winning the one before it is refused unless the
//!   Unlock Arcade cheat is on).
//! - `--stage-set <name or number>`: the stage screen's set (its group switch: the simulation's
//!   or the arcade's, by the game's name for it or 1 / 2); the first unless given.
//! - `--car <name>`, `--car2 <name>` (the second player's; the first's unless given):
//!   a car of the list the car screen shows, by its name as the game shows it or its place in
//!   that list (1-based); a car by name not in it is taken from every car's list, as the
//!   "Unlock Cars" cheat shows it. The player's own car unless given.
//! - `--gearbox <name or number>`: one of the game's three gearbox choices (1-based).
//! - `--players 1|2`: one player, or two on a split screen (the arcade has one player).
//! - `--seed <n>` (decimal or `0x` hex): the random numbers start from it.
//! - `--replay <slot or file>`: a replay instead of a race, played in the replay viewer as one
//!   chosen in the replays menu is: a saved replay's slot (0 .. 99, the number at the end of its
//!   file's name, `savedata\replay\replayNN`; the menu's pages count them the same way, twenty
//!   a page from 0), or a replay file's path. No race switch goes with it.
//! - `--then menus|quit`: after the race, its results and the menus (the default), or the
//!   program ends (in a championship or the arcade: after the first race). After a replay: the viewer stays until it is left, then the menus; or the
//!   program ends when the replay has played.
//! - `--launch <file>`: the rest from a file; switches on the command line come after it and
//!   win.
//! - `--host` or `--join <address>`: straight into the network lobby, hosting a session or
//!   joining the host's (a host name or number, with `:port` if it is not the usual one);
//!   `--name <name>` is the player's name there, `--port <number>` the TCP/IP port; both for
//!   this run only. No race or replay switch goes with them (the lobby chooses the race); in a
//!   launch file `join`, `name` and `port` are lines (`--host` is a switch only).
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

/// The kind of event a launch starts, as the mode menu's items start them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The single rally.
    #[default]
    Single,
    TimeTrial,
    Arcade,
    /// The first championship (the game's "Mobil One Championship").
    Championship,
    /// The A8 championship.
    A8,
}

impl Mode {
    /// The names the switch takes, each mode's own first.
    pub const NAMES: [&'static str; 5] = ["single", "time-trial", "arcade", "championship", "a8"];

    /// The mode a `--mode` value names: its own name, or the same without case, dashes and
    /// spaces, or a longer name the menus use for it.
    pub fn read(value: &str) -> Result<Mode, LaunchError> {
        let folded = Choice::fold(value.bytes());
        Ok(match &folded[..] {
            b"single" | b"singlerally" => Mode::Single,
            b"timetrial" => Mode::TimeTrial,
            b"arcade" => Mode::Arcade,
            b"championship" | b"mobilonechampionship" => Mode::Championship,
            b"a8" | b"a8championship" | b"a8rally" => Mode::A8,
            _ => {
                return Err(LaunchError(format!(
                    "--mode: `{}` is none of {}",
                    value.trim(),
                    Mode::NAMES.join(", ")
                )));
            }
        })
    }

    /// The mode's own name, as `--mode` takes it.
    pub fn name(self) -> &'static str {
        Mode::NAMES[self as usize]
    }

    /// Whether the mode's stages are chosen on the stage screen (the others bring their own).
    pub fn chooses_stage(self) -> bool {
        matches!(self, Mode::Single | Mode::TimeTrial)
    }
}

/// The service area's tyres, in the order of its choice (the game's text numbers 0x4B0 on).
pub const TYRES: [&str; 6] = [
    "Slicks",
    "Intermediate",
    "Wets",
    "Dry Gravel",
    "Wet Gravel",
    "Snow",
];
/// The service area's gear ratio choices.
pub const GEAR_RATIOS: u32 = 5;
/// The four sliders' positions: 0 up to this.
pub const SLIDER_MAX: u32 = 100;

/// The time trial's tune, each value as the car's record keeps it (`None`: as the record has it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tune {
    /// The tyre's place in `TYRES`, 0-based.
    pub tyres: Option<u32>,
    /// The gear ratio choice, 0-based.
    pub gear_ratio: Option<u32>,
    pub ride_height: Option<u32>,
    pub suspension: Option<u32>,
    pub brake_balance: Option<u32>,
    pub steering: Option<u32>,
}

impl Tune {
    /// The switches that give a tune.
    pub const KEYS: [&'static str; 6] = [
        "tyres",
        "gear-ratio",
        "ride-height",
        "suspension",
        "brake-balance",
        "steering",
    ];

    fn read(all: &Given) -> Result<Tune, LaunchError> {
        let slider = |key| -> Result<Option<u32>, LaunchError> {
            all.last(key)
                .map(|v| one_of(key, v, 0, SLIDER_MAX))
                .transpose()
        };
        let tyres = match all.last("tyres").map(Choice::read) {
            None => None,
            Some(choice) => {
                let names: Vec<Option<&[u8]>> = TYRES.iter().map(|n| Some(n.as_bytes())).collect();
                match choice.pick(&names) {
                    Pick::Found(i) => Some(i as u32),
                    _ => {
                        return Err(LaunchError(format!(
                            "--tyres: `{}` is none of {} (or 1 .. {})",
                            all.last("tyres").unwrap_or("").trim(),
                            TYRES.join(", "),
                            TYRES.len()
                        )));
                    }
                }
            }
        };
        Ok(Tune {
            tyres,
            gear_ratio: all
                .last("gear-ratio")
                .map(|v| one_of("gear-ratio", v, 1, GEAR_RATIOS).map(|n| n - 1))
                .transpose()?,
            ride_height: slider("ride-height")?,
            suspension: slider("suspension")?,
            brake_balance: slider("brake-balance")?,
            steering: slider("steering")?,
        })
    }

    /// Whether any value is given.
    pub fn is_given(&self) -> bool {
        *self != Tune::default()
    }
}

/// The arcade's levels a launch can start at, as the level screen counts them (the screen's
/// start item, then its five level items).
pub const ARCADE_LEVELS: u32 = 6;

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
    pub mode: Mode,
    /// The stage, for the modes whose stages are chosen (`Mode::chooses_stage`).
    pub stage: Option<Stage>,
    /// The arcade's level, 1 .. 6 as the level screen counts (`None`: the first).
    pub level: Option<u32>,
    /// The time trial's tune.
    pub tune: Tune,
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
pub const VALUED: [&str; 21] = [
    "join",
    "name",
    "port",
    "mode",
    "level",
    "tyres",
    "gear-ratio",
    "ride-height",
    "suspension",
    "brake-balance",
    "steering",
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

/// The switches that take none (the launch file has no lines for them).
pub const FLAGS: [&str; 1] = ["host"];

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
        if FLAGS.contains(&key) {
            if inline.is_some() {
                return Err(LaunchError(format!("--{key} takes no value")));
            }
            out.push((key.to_string(), String::new()));
            continue;
        }
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

/// The longest player name the lobby takes, in bytes.
pub const NAME_LONGEST: usize = 30;
/// The longest address the settings screen's field holds, in bytes.
pub const ADDRESS_LONGEST: usize = 63;

/// What a network launch does in the lobby's place: make the session, or join one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Host,
    /// The host to join: its name or number, as the settings screen's address field holds it.
    Join(String),
}

/// A launch into the network lobby.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetLaunch {
    pub role: Role,
    /// The player's name for this run; the player's own unless given.
    pub name: Option<String>,
    /// The TCP/IP port for this run; the saved one (or the usual) unless given.
    pub port: Option<u16>,
    pub then: Then,
}

/// What the command line and the launch file asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    Race(Launch),
    Replay(ReplayLaunch),
    Net(NetLaunch),
}

/// A host's address as `--join` gives it: a name or an IPv4 number, then `:port` if it is
/// not the usual one. Gives the host and the port. (The network code takes IPv4 only.)
pub fn host_address(value: &str) -> Result<(String, Option<u16>), LaunchError> {
    let bad = |why: &str| LaunchError(format!("--join: `{value}` is not an address: {why}"));
    let v = value.trim();
    let (host, port) = match v.split_once(':') {
        None => (v, None),
        Some((host, port)) if !port.contains(':') => (host, Some(port)),
        Some(_) => return Err(bad("only IPv4 addresses and names are taken")),
    };
    if host.is_empty() {
        return Err(bad("no host before the colon"));
    }
    if host.len() > ADDRESS_LONGEST {
        return Err(bad(&format!("longer than {ADDRESS_LONGEST} characters")));
    }
    let label_ok = |l: &str| {
        !l.is_empty()
            && !l.starts_with('-')
            && !l.ends_with('-')
            && l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    };
    if !host.split('.').all(label_ok) {
        return Err(bad("a host name is letters, digits, hyphens and dots"));
    }
    let numbers_only = host.bytes().all(|b| b.is_ascii_digit() || b == b'.');
    if numbers_only && host.parse::<std::net::Ipv4Addr>().is_err() {
        return Err(bad("a number address is four numbers up to 255"));
    }
    let port = match port {
        None => None,
        Some(p) => Some(port_number("join", p)?),
    };
    Ok((host.to_string(), port))
}

fn port_number(key: &str, value: &str) -> Result<u16, LaunchError> {
    match number(key, value).map(u16::try_from) {
        Ok(Ok(port)) if port != 0 => Ok(port),
        _ => Err(LaunchError(format!(
            "--{key}: `{value}` is not a port (1 to 65535)"
        ))),
    }
}

fn player_name(value: &str) -> Result<String, LaunchError> {
    let name = value.trim();
    if name.is_empty()
        || name.len() > NAME_LONGEST
        || !name.bytes().all(|b| (0x20..0x7F).contains(&b))
    {
        return Err(LaunchError(format!(
            "--name: `{value}` is not a name (1 to {NAME_LONGEST} letters, digits, spaces or signs of ASCII)"
        )));
    }
    Ok(name.to_string())
}

/// The network launch the values ask for, if they ask for one.
fn net_launch(all: &Given) -> Result<Option<NetLaunch>, LaunchError> {
    let (host, join) = (all.last("host").is_some(), all.last("join"));
    if !host && join.is_none() {
        return match ["name", "port"].iter().find(|key| all.last(key).is_some()) {
            Some(key) => Err(LaunchError(format!(
                "--{key} goes with --host or --join (the lobby's switches)"
            ))),
            None => Ok(None),
        };
    }
    if host && join.is_some() {
        return Err(LaunchError(
            "--host and --join do not go together: one machine hosts, the others join".into(),
        ));
    }
    let way = if host { "--host" } else { "--join" };
    let replay_or_race = RACE_ONLY
        .iter()
        .chain(&["replay"])
        .find(|k| all.last(k).is_some());
    if let Some(other) = replay_or_race {
        return Err(LaunchError(format!(
            "{way} does not go with --{other}: a lobby chooses its own race"
        )));
    }
    let mut port = all
        .last("port")
        .map(|v| port_number("port", v))
        .transpose()?;
    let role = match join {
        None => Role::Host,
        Some(address) => {
            let (host, in_address) = host_address(address)?;
            match (port, in_address) {
                (Some(a), Some(b)) if a != b => {
                    return Err(LaunchError(format!(
                        "--join names port {b} and --port {a}: give one"
                    )));
                }
                (None, found) => port = found,
                _ => {}
            }
            Role::Join(host)
        }
    };
    Ok(Some(NetLaunch {
        role,
        name: all.last("name").map(player_name).transpose()?,
        port,
        then: then(all)?,
    }))
}

/// The switches that choose a race's parts: none goes with a replay, which holds its own.
const RACE_ONLY: [&str; 15] = [
    "mode",
    "level",
    "tyres",
    "gear-ratio",
    "ride-height",
    "suspension",
    "brake-balance",
    "steering",
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
        if let Some(net) = net_launch(&all)? {
            return Ok(Some(Asked::Net(net)));
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
        let mode = match all.last("mode") {
            Some(v) => Mode::read(v)?,
            None => Mode::Single,
        };
        let stage = match all.last("stage") {
            Some(v) => Some(stage(v)?),
            None if mode == Mode::Single => {
                return Err(LaunchError("a launch needs --stage or --replay".into()));
            }
            None if mode.chooses_stage() => {
                return Err(LaunchError(format!(
                    "--mode {} needs --stage (the stage to race)",
                    mode.name()
                )));
            }
            None => None,
        };
        let players = match all.last("players") {
            Some(v) => one_of("players", v, 1, 2)?,
            None => 1,
        };
        let level = match all.last("level") {
            Some(v) if mode == Mode::Arcade => Some(one_of("level", v, 1, ARCADE_LEVELS)?),
            _ => None,
        };
        let car = all.last("car").map(Choice::read);
        let car2 = all.last("car2").map(Choice::read).or_else(|| car.clone());
        let launch = Launch {
            mode,
            stage,
            level,
            tune: Tune::read(all)?,
            stage_set: all.last("stage-set").map(Choice::read),
            cars: [car, car2],
            gearbox: all.last("gearbox").map(Choice::read),
            players,
            seed: all.last("seed").map(|v| number("seed", v)).transpose()?,
            then: then(all)?,
        };
        launch.refuse_what_does_not_fit(all)?;
        Ok(launch)
    }

    /// The switches given that the mode has no use for, refused with the reason.
    fn refuse_what_does_not_fit(&self, all: &Given) -> Result<(), LaunchError> {
        let name = self.mode.name();
        let own_stages = "the arcade's levels hold their own stages";
        let in_order = "a championship's stages come in its order";
        let only_arcade = "only the arcade has levels";
        let misfits: &[(&str, &str)] = match self.mode {
            Mode::Single | Mode::TimeTrial => &[("level", only_arcade)],
            Mode::Arcade => &[
                ("stage", own_stages),
                ("stage-set", own_stages),
                ("car2", "the arcade has one player"),
            ],
            Mode::Championship | Mode::A8 => &[
                ("stage", in_order),
                ("stage-set", in_order),
                ("level", only_arcade),
            ],
        };
        if self.mode != Mode::TimeTrial
            && let Some(key) = Tune::KEYS.iter().find(|key| all.last(key).is_some())
        {
            return Err(LaunchError(format!(
                "--{key} does not go with --mode {name}: only a time trial has the service area's tune"
            )));
        }
        for (key, why) in misfits {
            if all.last(key).is_some() {
                return Err(LaunchError(format!(
                    "--{key} does not go with --mode {name}: {why}"
                )));
            }
        }
        if self.mode == Mode::Arcade && self.players != 1 {
            return Err(LaunchError(
                "--players 2 does not go with --mode arcade: the arcade has one player".into(),
            ));
        }
        Ok(())
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
            if !long.contains('=') && crate::cli::takes_value(word) {
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
    /// The time trial's service area, answered with the launch's tune instead of shown.
    fn service(&mut self, game: &mut G) -> Result<(), G::Error>;
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
    serviced: bool,
    raced: bool,
}

impl<'g, G: Game, A: Answers<G>> Launched<'g, G, A> {
    pub fn new(game: &'g mut G, answers: A, players: u32) -> Self {
        Self {
            game,
            answers,
            cars_left: players,
            stage_left: true,
            serviced: false,
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
            Call::SelectStageLoop | Call::TimeTrialSelectStage if self.stage_left => {
                self.stage_left = false;
                self.game.set(Var::Answer, 0);
                self.game.set_flag(Switch::EventLeft, false);
                self.answers.stage(self.game)
            }
            Call::TimeTrialService if !self.serviced => {
                self.serviced = true;
                self.game.set(Var::Answer, 0);
                self.answers.service(self.game)
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
                Asked::Net(net) => panic!("a lobby: {net:?}"),
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

    fn lobby(args: &[&str]) -> Result<Option<NetLaunch>, LaunchError> {
        Asked::from_args(args, &mut |_| Ok("join = box.local # a file".into())).map(|a| {
            a.map(|a| match a {
                Asked::Net(net) => net,
                other => panic!("not a lobby: {other:?}"),
            })
        })
    }

    #[test]
    fn host_and_join_read() {
        let host = lobby(&[
            "--host",
            "--name",
            "Ann Lee",
            "--port=5000",
            "--then",
            "quit",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(
            host,
            NetLaunch {
                role: Role::Host,
                name: Some("Ann Lee".into()),
                port: Some(5000),
                then: Then::Quit,
            }
        );
        let join = lobby(&["--join", "127.0.0.1"]).unwrap().unwrap();
        assert_eq!(join.role, Role::Join("127.0.0.1".into()));
        assert_eq!((join.name, join.port, join.then), (None, None, Then::Menus));
        let join = lobby(&["--join=game.example.org:47700"]).unwrap().unwrap();
        assert_eq!(join.role, Role::Join("game.example.org".into()));
        assert_eq!(join.port, Some(47700));
        let from_file = lobby(&["--launch", "a.txt", "--name", "Bo"])
            .unwrap()
            .unwrap();
        assert_eq!(from_file.role, Role::Join("box.local".into()));
        assert_eq!(from_file.name.as_deref(), Some("Bo"));
        assert_eq!(
            lobby(&["--join", "h:80", "--port", "80"])
                .unwrap()
                .unwrap()
                .port,
            Some(80)
        );
    }

    #[test]
    fn lobby_refusals() {
        let refused = |args: &[&str]| lobby(args).unwrap_err().0;
        assert!(refused(&["--host", "--join", "a"]).contains("do not go together"));
        assert!(refused(&["--host", "--stage", "langton"]).contains("--stage"));
        assert!(refused(&["--join", "a", "--replay", "3"]).contains("--replay"));
        assert!(refused(&["--name", "Ann"]).contains("--host or --join"));
        assert!(refused(&["--port", "99"]).contains("--host or --join"));
        assert!(refused(&["--host=yes"]).contains("no value"));
        assert!(refused(&["--host", "--port", "0"]).contains("1 to 65535"));
        assert!(refused(&["--host", "--port", "65536"]).contains("1 to 65535"));
        assert!(refused(&["--host", "--name", ""]).contains("--name"));
        assert!(refused(&["--host", "--name", &"x".repeat(31)]).contains("--name"));
        assert!(refused(&["--host", "--name", "caf\u{e9}"]).contains("--name"));
        assert!(refused(&["--join", "a:1", "--port", "2"]).contains("give one"));
        assert!(refused(&["--join"]).contains("needs a value"));
        assert!(refused(&["--host", "--then", "later"]).contains("--then"));
    }

    #[test]
    fn addresses_read() {
        let ok = |s: &str| host_address(s).unwrap();
        assert_eq!(ok("192.168.1.2"), ("192.168.1.2".into(), None));
        assert_eq!(ok(" box-1.lan:9 "), ("box-1.lan".into(), Some(9)));
        for bad in [
            "",
            ":80",
            "a b",
            "a:",
            "a:x",
            "a:0",
            "a:70000",
            "a:1:2",
            "::1",
            "[::1]:5",
            "-a",
            "a-.b",
            "a..b",
            ".a",
            "a.",
            "1.2.3",
            "1.2.3.4.5",
            "300.1.1.1",
            "ex_ample.org",
        ] {
            assert!(host_address(bad).is_err(), "`{bad}` taken");
        }
        assert!(host_address(&"a".repeat(64)).is_err());
        assert!(host_address(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn every_mode_read() {
        let mode = |args: &[&str]| read(args).map(|l| l.unwrap());
        for (word, wanted) in [
            ("single", Mode::Single),
            ("Single-Rally", Mode::Single),
            ("time-trial", Mode::TimeTrial),
            ("timetrial", Mode::TimeTrial),
            ("arcade", Mode::Arcade),
            ("championship", Mode::Championship),
            ("a8", Mode::A8),
            ("A8 Championship", Mode::A8),
        ] {
            let stage = ["--stage", "langton"];
            let args: Vec<&str> = if wanted.chooses_stage() {
                ["--mode", word].into_iter().chain(stage).collect()
            } else {
                vec!["--mode", word]
            };
            let l = mode(&args).unwrap();
            assert_eq!(l.mode, wanted, "{word}");
            assert_eq!(l.stage.is_some(), wanted.chooses_stage(), "{word}");
            assert_eq!(Mode::read(wanted.name()), Ok(wanted), "its own name");
        }
        assert_eq!(Mode::NAMES.len(), 5);
        let bad = mode(&["--mode", "rally", "--stage", "x"]).unwrap_err();
        assert!(
            bad.0.contains("time-trial") && bad.0.contains("rally"),
            "{bad}"
        );
        assert!(read(&["--mode"]).is_err(), "no value");
    }

    #[test]
    fn modes_need_what_they_need() {
        let refused = |args: &[&str]| read(args).unwrap_err().0;
        assert!(refused(&["--mode", "single"]).contains("--stage"));
        assert!(refused(&["--mode", "time-trial"]).contains("time-trial needs --stage"));
        // The arcade and the championships bring their stages.
        for mode in ["arcade", "championship", "a8"] {
            for key in ["--stage", "--stage-set"] {
                let said = refused(&["--mode", mode, key, "1"]);
                assert!(said.contains(key) && said.contains(mode), "{said}");
            }
        }
        // Levels are the arcade's.
        for mode in ["single", "time-trial", "championship", "a8"] {
            let said = refused(&["--mode", mode, "--stage", "x", "--level", "2"]);
            let said = if mode == "single" || mode == "time-trial" {
                said
            } else {
                refused(&["--mode", mode, "--level", "2"])
            };
            assert!(
                said.contains("--level") && said.contains("only the arcade"),
                "{said}"
            );
        }
        let arcade = |args: &[&str]| {
            let all: Vec<&str> = ["--mode", "arcade"].iter().chain(args).copied().collect();
            read(&all)
        };
        assert_eq!(arcade(&[]).unwrap().unwrap().level, None);
        assert_eq!(arcade(&["--level", "6"]).unwrap().unwrap().level, Some(6));
        assert!(
            arcade(&["--level", "7"])
                .unwrap_err()
                .0
                .contains("between 1 and 6")
        );
        assert!(arcade(&["--level", "0"]).is_err());
        assert!(
            arcade(&["--players", "2"])
                .unwrap_err()
                .0
                .contains("one player")
        );
        assert!(
            arcade(&["--car2", "puma"])
                .unwrap_err()
                .0
                .contains("one player")
        );
        let ok = arcade(&[
            "--car",
            "puma",
            "--gearbox",
            "2",
            "--seed",
            "3",
            "--then",
            "quit",
        ])
        .unwrap()
        .unwrap();
        assert_eq!((ok.then, ok.seed), (Then::Quit, Some(3)));
        // Two players on a championship's split screen.
        let two = read(&["--mode=a8", "--players", "2", "--car2", "x"])
            .unwrap()
            .unwrap();
        assert_eq!((two.players, two.mode), (2, Mode::A8));
        // A replay holds none of it.
        for key in ["--mode", "--level"] {
            let said = read(&["--replay", "3", key, "1"]).unwrap_err().0;
            assert!(said.contains(key), "{said}");
        }
        // The launch file takes them too.
        let file = Asked::from_args(&["--launch", "a"], &mut |_| {
            Ok("mode = arcade\nlevel = 2".into())
        });
        assert!(matches!(file, Ok(Some(Asked::Race(l))) if l.level == Some(2)));
    }

    #[test]
    fn the_time_trials_tune() {
        let tt = |args: &[&str]| {
            let all: Vec<&str> = ["--mode", "time-trial", "--stage", "langton"]
                .iter()
                .chain(args)
                .copied()
                .collect();
            read(&all).map(|l| l.unwrap().tune)
        };
        assert_eq!(tt(&[]).unwrap(), Tune::default());
        assert!(!Tune::default().is_given());
        let tune = tt(&[
            "--tyres",
            "dry-gravel",
            "--gear-ratio=5",
            "--ride-height",
            "0",
            "--suspension",
            "100",
            "--brake-balance",
            "40",
            "--steering",
            "55",
        ])
        .unwrap();
        assert_eq!(
            tune,
            Tune {
                tyres: Some(3),
                gear_ratio: Some(4),
                ride_height: Some(0),
                suspension: Some(100),
                brake_balance: Some(40),
                steering: Some(55),
            }
        );
        assert!(tune.is_given());
        assert_eq!(tt(&["--tyres", "6"]).unwrap().tyres, Some(5), "by number");
        assert_eq!(tt(&["--tyres", "snow"]).unwrap().tyres, Some(5));
        assert!(
            tt(&["--tyres", "wet"])
                .unwrap_err()
                .0
                .contains("Wet Gravel")
        );
        for (args, said) in [
            (["--tyres", "mud"], "--tyres"),
            (["--tyres", "7"], "--tyres"),
            (["--gear-ratio", "0"], "between 1 and 5"),
            (["--gear-ratio", "6"], "between 1 and 5"),
            (["--ride-height", "101"], "between 0 and 100"),
            (["--steering", "x"], "not a number"),
        ] {
            let refused = tt(&args).unwrap_err().0;
            assert!(refused.contains(said), "{refused}");
        }
        // Only a time trial has a tune.
        for mode in ["single", "arcade", "championship", "a8"] {
            let mut all = vec!["--mode", mode, "--brake-balance", "10"];
            if mode == "single" {
                all.extend(["--stage", "langton"]);
            }
            let refused = read(&all).unwrap_err().0;
            assert!(
                refused.contains("--brake-balance") && refused.contains("time trial"),
                "{refused}"
            );
        }
        let refused = read(&["--replay", "1", "--tyres", "1"]).unwrap_err().0;
        assert!(refused.contains("--tyres"), "{refused}");
        let file = Asked::from_args(&["--launch", "a"], &mut |_| {
            Ok("mode = time-trial\nstage = x\ntyres = slicks\nsteering = 9".into())
        });
        assert!(
            matches!(file, Ok(Some(Asked::Race(l))) if l.tune.tyres == Some(0) && l.tune.steering == Some(9))
        );
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
        assert_eq!(l.stage, Some(Stage::Name("Hamiltons Folly".into())));
        assert_eq!(l.mode, Mode::Single);
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
                Some(Stage::Place { rally: 4, stage: 5 }),
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
        assert_eq!(l.stage, Some(Stage::Name("langton".into())));
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
        // Every valued switch of the table has its value passed over, not only the launch's.
        assert_eq!(
            log_path(&["--renderer", "software", "my.log", "--allow-another"]),
            Some("my.log".into())
        );
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
        fn service(&mut self, g: &mut Log) -> Result<(), <Log as Game>::Error> {
            g.set(Var::Round, 250);
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
    fn a_time_trial_answers_the_cars_and_the_stage_screen_of_its_own() {
        // The time trial's stage screen, then a yes on the stage choice after the race.
        let mut log = Log::default().on("TimeTrialSelectStage", vec![vec![(Var::Answer, 1)]]);
        super::super::rounds::time_trial_one(&mut Launched::new(&mut log, Noted, 1)).unwrap();
        assert_eq!(log.count("SelectCarLoop"), 0, "the car answered");
        assert_eq!(log.count("Round=100"), 1);
        assert_eq!(log.count("Round=200"), 1, "the stage answered once");
        assert_eq!(
            log.count("TimeTrialSelectStage"),
            1,
            "the second is the screen's"
        );
        assert_eq!(log.count("Round=300"), 1);
        assert_eq!(log.count("Round=400"), 1);
        // The service area is answered once, not shown; a later round's is the screen's.
        assert_eq!(log.count("TimeTrialService"), 0);
        assert_eq!(log.count("Round=250"), 1);
        let at = |step: &str| log.steps.iter().position(|s| s == step).unwrap();
        assert!(at("Round=200") < at("Round=250"));
        assert!(at("Round=250") < at("Round=300"));
        let mut log = Log::default().on("TimeTrialSelectStage", vec![vec![(Var::Answer, 1)]]);
        super::super::rounds::time_trial_split(&mut Launched::new(&mut log, Noted, 2)).unwrap();
        assert_eq!((log.count("Round=100"), log.count("Round=200")), (2, 1));
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
