//! The program's command line as a whole: every switch it takes, in one table that `--help`
//! prints and that unknown switches are checked against. The parts that read the switches keep
//! their own readers (`menu::launch`, `options::files`, `system::command_line`); a test holds
//! each reader's names to this table.

/// What a switch belongs to, in the order `--help` lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Race,
    Event,
    Tune,
    Replay,
    AfterLaunch,
    Network,
    Options,
    Machine,
    Original,
    Help,
}

impl Group {
    const ORDER: [Group; 10] = [
        Group::Race,
        Group::Event,
        Group::Tune,
        Group::Replay,
        Group::AfterLaunch,
        Group::Network,
        Group::Options,
        Group::Machine,
        Group::Original,
        Group::Help,
    ];

    fn title(self) -> &'static str {
        match self {
            Group::Race => "Straight into a race (a single rally and a time trial need --stage):",
            Group::Tune => {
                "The time trial's tune, as the service area sets it (--mode time-trial; the service area is not shown):"
            }
            Group::Event => "The kind of event a launch starts (the single rally unless given):",
            Group::Replay => "Straight into a replay:",
            Group::AfterLaunch => "With a race, a replay or the lobby:",
            Group::Network => "Straight into the network lobby (no race switch goes with these):",
            Group::Options => "The options files:",
            Group::Machine => "How the program runs here:",
            Group::Original => "The original program's switches:",
            Group::Help => "This list:",
        }
    }
}

/// One switch: its name with its dashes, what its value is (`None`: it takes none), what it
/// does.
#[derive(Clone, Copy, Debug)]
pub struct Switch {
    pub name: &'static str,
    pub value: Option<&'static str>,
    pub text: &'static str,
    pub group: Group,
}

const fn switch(
    name: &'static str,
    value: Option<&'static str>,
    text: &'static str,
    group: Group,
) -> Switch {
    Switch {
        name,
        value,
        text,
        group,
    }
}

/// Every switch the program reads.
pub const SWITCHES: &[Switch] = &[
    switch(
        "--stage",
        Some("<name | rally-stage>"),
        "the stage, by name or as two numbers in the stage screen's order (4-5)",
        Group::Race,
    ),
    switch(
        "--stage-set",
        Some("<simulation | arcade>"),
        "the stage screen's set (or 1 / 2); the arcade set runs each stage the other way",
        Group::Race,
    ),
    switch(
        "--car",
        Some("<name | number>"),
        "player 1's car; without it, the car last chosen",
        Group::Race,
    ),
    switch(
        "--car2",
        Some("<name | number>"),
        "player 2's car; without it, the same as --car",
        Group::Race,
    ),
    switch(
        "--car3",
        Some("<name | number>"),
        "player 3's car (--players 3 or 4); without it, the same as --car",
        Group::Race,
    ),
    switch(
        "--car4",
        Some("<name | number>"),
        "player 4's car (--players 4); without it, the same as --car",
        Group::Race,
    ),
    switch(
        "--gearbox",
        Some("<manual | semi-automatic | automatic>"),
        "the gearbox (or 1 .. 3)",
        Group::Race,
    ),
    switch(
        "--players",
        Some("<1 .. 4>"),
        "one player, or two on a split screen; three or four share one in the single rally",
        Group::Race,
    ),
    switch(
        "--seed",
        Some("<number>"),
        "the race's random numbers start from it (decimal, or hexadecimal with 0x)",
        Group::Race,
    ),
    switch(
        "--mode",
        Some("<single | time-trial | arcade | championship | a8>"),
        "the event: the single rally, a time trial, the arcade, or a new championship",
        Group::Event,
    ),
    switch(
        "--level",
        Some("<1 .. 6>"),
        "the arcade's level as the level screen counts them (1 unless given; one not yet opened is refused)",
        Group::Event,
    ),
    switch(
        "--tyres",
        Some("<slicks | intermediate | wets | dry-gravel | wet-gravel | snow>"),
        "the tyres (or 1 .. 6, in that order)",
        Group::Tune,
    ),
    switch(
        "--gear-ratio",
        Some("<1 .. 5>"),
        "the gear ratio choice, counted from the first the service area offers",
        Group::Tune,
    ),
    switch(
        "--ride-height",
        Some("<0 .. 100>"),
        "the ride height slider's position, as the set-up files count it",
        Group::Tune,
    ),
    switch(
        "--suspension",
        Some("<0 .. 100>"),
        "the suspension stiffness slider's position",
        Group::Tune,
    ),
    switch(
        "--brake-balance",
        Some("<0 .. 100>"),
        "the brake balance slider's position",
        Group::Tune,
    ),
    switch(
        "--steering",
        Some("<0 .. 100>"),
        "the steering sensitivity slider's position",
        Group::Tune,
    ),
    switch(
        "--replay",
        Some("<slot | file>"),
        "a saved replay's slot (0 .. 99), or a replay file's path",
        Group::Replay,
    ),
    switch(
        "--render-replay",
        Some("<slot | file>"),
        "render a replay to a video (with --out) instead of playing it, and end; headless, faster than real time",
        Group::Replay,
    ),
    switch(
        "--out",
        Some("<file>"),
        "the video file of --render-replay (.mp4 or .mkv); without an ffmpeg on the path, a folder of numbered PNGs named after it",
        Group::Replay,
    ),
    switch(
        "--size",
        Some("<WxH>"),
        "the video's size (the race screen's size for the render; default: the options' race size)",
        Group::Replay,
    ),
    switch(
        "--fps",
        Some("<1 .. 480>"),
        "the video's frames a second (default 60); frames between the 100 ticks a second are blended",
        Group::Replay,
    ),
    switch(
        "--quality",
        Some("<low | medium | high | best>"),
        "how well the encoder keeps the picture of --render-replay (default high)",
        Group::Replay,
    ),
    switch(
        "--encoder",
        Some("<automatic | software>"),
        "which encoder writes the picture of --render-replay: a graphics card's when one works (default), or only the software one",
        Group::Replay,
    ),
    switch(
        "--from",
        Some("<seconds>"),
        "where in the replay the render of --render-replay starts (seconds, like 12.5; default the start)",
        Group::Replay,
    ),
    switch(
        "--to",
        Some("<seconds>"),
        "where the render ends, to the whole second after it (default the replay's end)",
        Group::Replay,
    ),
    switch(
        "--no-sound",
        None,
        "a video of --render-replay without sound",
        Group::Replay,
    ),
    switch(
        "--then",
        Some("<menus | quit>"),
        "after the race, the replay or the lobby: the menus (the default), or the program ends",
        Group::AfterLaunch,
    ),
    switch(
        "--launch",
        Some("<file>"),
        "the same settings from a file of `name = value` lines; switches typed as well win",
        Group::AfterLaunch,
    ),
    switch(
        "--host",
        None,
        "host a network game: the lobby, with this machine's session in it",
        Group::Network,
    ),
    switch(
        "--join",
        Some("<host[:port]>"),
        "join the network game of that host (a name or an IPv4 number) and go to its lobby",
        Group::Network,
    ),
    switch(
        "--name",
        Some("<player name>"),
        "your name in the lobby (up to 30 letters, digits and signs), for this run only",
        Group::Network,
    ),
    switch(
        "--port",
        Some("<1 .. 65535>"),
        "the TCP/IP port, for this run only (the same on every machine; usually 47624)",
        Group::Network,
    ),
    switch(
        "--load-old-options",
        None,
        "read the options from the original's var/ral.dat, whatever var/rc2k.toml holds",
        Group::Options,
    ),
    switch(
        "--save-old-options",
        None,
        "write the original's var/ral.dat as well when the options are saved",
        Group::Options,
    ),
    switch(
        "--renderer",
        Some("<gpu | software>"),
        "what draws the 3D: the GPU (the default) or the software rasteriser; wins over RC2K_RENDERER",
        Group::Machine,
    ),
    switch(
        "--allow-another",
        None,
        "start although another copy runs in this folder (the last to quit wins the options)",
        Group::Machine,
    ),
    switch(
        "--instance",
        Some("<1 .. 8>"),
        "run as that instance of this folder, beside the others: its own var and savedata folders (instances/<n>/, the first start copying var), log and lock; --instance 1 is the folder's own",
        Group::Machine,
    ),
    switch(
        "--game-data",
        Some("<folder | zip>"),
        "where to take the game's data from when this folder has none (an installed game's folder, the download / CD layout, or a zip of either); without it the first-run page asks",
        Group::Machine,
    ),
    switch("-v", None, "show the version and end", Group::Original),
    switch("-a", None, "no sound", Group::Original),
    switch(
        "-u",
        None,
        "uninstall the game in this folder",
        Group::Original,
    ),
    switch(
        "-p, -i",
        None,
        "an installation without the installer's pages (the installer's own call)",
        Group::Original,
    ),
    switch(
        "-c",
        Some("<1 | 2>"),
        "read and not used (it once left out a 3D card)",
        Group::Original,
    ),
    switch(
        "--help",
        None,
        "show this list and end (also -h)",
        Group::Help,
    ),
];

/// Whether `name` (a switch with its dashes, no `=`) is a switch of the table that takes a
/// value, which is then the next word.
pub(crate) fn takes_value(name: &str) -> bool {
    SWITCHES.iter().any(|s| s.name == name && s.value.is_some())
}

/// The words that ask for the list.
const HELP_WORDS: [&str; 4] = ["--help", "-h", "-?", "/?"];

/// Whether the program's words (after its own name) ask for the list.
pub fn wants_help<S: AsRef<str>>(args: &[S]) -> bool {
    args.iter().any(|a| HELP_WORDS.contains(&a.as_ref()))
}

/// The first `--` switch among the program's words that the program does not know, with the
/// message that names it. A switch's value (the word after a switch that takes one) is passed
/// over, so a value that starts with `--` is no switch.
pub fn unknown<S: AsRef<str>>(args: &[S]) -> Option<String> {
    let mut i = 0;
    while i < args.len() {
        let word = args[i].as_ref();
        i += 1;
        let Some(long) = word.strip_prefix("--") else {
            continue;
        };
        let (key, inline) = match long.split_once('=') {
            Some((k, _)) => (k, true),
            None => (long, false),
        };
        let name = format!("--{key}");
        match SWITCHES.iter().find(|s| s.name == name) {
            Some(s) if s.value.is_some() && !inline => i += 1,
            Some(_) => {}
            None => {
                return Some(format!(
                    "{name} is not a switch of this program; `rc2k --help` lists them"
                ));
            }
        }
    }
    None
}

/// The list `--help` prints: what the program is, how it is started, every switch by group.
pub fn help_text(version: &str) -> String {
    let mut out = format!(
        "Rally Championship (rc2k {version})\n\n\
         Usage: rc2k [switches] [log file]\n\n\
         Run it in the game's folder (the one with `files` and `var`). The log is rc2k.log there\n\
         unless another file is named. Each switch with a value also takes it after `=`\n\
         (--stage=langton).\n"
    );
    let shown = |s: &Switch| match s.value {
        Some(v) => format!("{} {v}", s.name),
        None => s.name.to_string(),
    };
    // A switch longer than the column has its text on the next line.
    const COLUMN: usize = 30;
    for group in Group::ORDER {
        out.push('\n');
        out.push_str(group.title());
        out.push('\n');
        for s in SWITCHES.iter().filter(|s| s.group == group) {
            let left = shown(s);
            if left.len() <= COLUMN {
                out.push_str(&format!("  {left:<COLUMN$}  {}\n", s.text));
            } else {
                out.push_str(&format!("  {left}\n  {:COLUMN$}  {}\n", "", s.text));
            }
        }
    }
    out.push_str(
        "\nExamples:\n  rc2k --stage langton\n  \
         rc2k --stage \"hamiltons folly\" --car felicia --gearbox manual --players 2 --seed 42\n  \
         rc2k --replay 0 --then quit\n\n\
         Environment variables (RC2K_RENDERER=software and others) and the switches' details:\n\
         the project wiki's \"Running and options\" page.\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_is_asked_by_any_of_its_words() {
        assert!(wants_help(&["--stage", "x", "-h"]));
        assert!(wants_help(&["/?"]));
        assert!(!wants_help(&["--stage", "help"]));
    }

    #[test]
    fn unknown_switches_are_named_and_values_passed_over() {
        assert_eq!(unknown(&["--stage", "--odd-name", "--then=quit"]), None);
        assert_eq!(unknown(&["-a", "my.log", "--load-old-options"]), None);
        let refused = unknown(&["--stage", "x", "--stages", "y"]).unwrap();
        assert!(refused.contains("--stages"), "{refused}");
        assert!(unknown(&["--speed=2"]).is_some());
    }

    #[test]
    fn the_list_names_every_switch_once() {
        let text = help_text("0.1.0");
        for s in SWITCHES {
            assert!(text.contains(s.name), "{} missing", s.name);
        }
        let mut names: Vec<&str> = SWITCHES.iter().map(|s| s.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), SWITCHES.len(), "a switch listed twice");
    }

    /// The readers' own names are all in the table, with a value where they take one.
    #[test]
    fn the_readers_agree_with_the_table() {
        for key in crate::menu::launch::VALUED {
            let name = format!("--{key}");
            let found = SWITCHES.iter().find(|s| s.name == name);
            assert!(
                found.is_some_and(|s| s.value.is_some()),
                "{name} is read by the launch but not listed with a value"
            );
        }
        for key in crate::menu::launch::FLAGS {
            let name = format!("--{key}");
            let found = SWITCHES.iter().find(|s| s.name == name);
            assert!(
                found.is_some_and(|s| s.value.is_none()),
                "{name} is read by the launch but not listed as a switch without a value"
            );
        }
        for key in crate::system::program_switches::VALUED {
            let name = format!("--{key}");
            let found = SWITCHES.iter().find(|s| s.name == name);
            assert!(
                found.is_some_and(|s| s.value.is_some()),
                "{name} is read by the program's switches but not listed with a value"
            );
        }
        for name in crate::system::program_switches::PLAIN {
            assert!(
                SWITCHES.iter().any(|s| s.name == name && s.value.is_none()),
                "{name} is read by the program's switches but not listed"
            );
        }
        for name in crate::options::files::SWITCHES {
            assert!(
                SWITCHES.iter().any(|s| s.name == name && s.value.is_none()),
                "{name} is read by the options but not listed"
            );
        }
    }
}
