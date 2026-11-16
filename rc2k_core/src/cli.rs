//! The program's command line as a whole: every switch it takes, in one table that `--help`
//! prints and that unknown switches are checked against. The parts that read the switches keep
//! their own readers (`menu::launch`, `options::files`, `system::command_line`); a test holds
//! each reader's names to this table.

/// What a switch belongs to, in the order `--help` lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Race,
    Replay,
    AfterLaunch,
    Options,
    Original,
    Help,
}

impl Group {
    const ORDER: [Group; 6] = [
        Group::Race,
        Group::Replay,
        Group::AfterLaunch,
        Group::Options,
        Group::Original,
        Group::Help,
    ];

    fn title(self) -> &'static str {
        match self {
            Group::Race => "Straight into a race (a launch needs --stage):",
            Group::Replay => "Straight into a replay:",
            Group::AfterLaunch => "With either:",
            Group::Options => "The options files:",
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
        "--gearbox",
        Some("<manual | semi-automatic | automatic>"),
        "the gearbox (or 1 .. 3)",
        Group::Race,
    ),
    switch(
        "--players",
        Some("<1 | 2>"),
        "one player, or two on a split screen",
        Group::Race,
    ),
    switch(
        "--seed",
        Some("<number>"),
        "the race's random numbers start from it (decimal, or hexadecimal with 0x)",
        Group::Race,
    ),
    switch(
        "--replay",
        Some("<slot | file>"),
        "a saved replay's slot (0 .. 99), or a replay file's path",
        Group::Replay,
    ),
    switch(
        "--then",
        Some("<menus | quit>"),
        "after the race or the replay: the menus (the default), or the program ends",
        Group::AfterLaunch,
    ),
    switch(
        "--launch",
        Some("<file>"),
        "the same settings from a file of `name = value` lines; switches typed as well win",
        Group::AfterLaunch,
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
        for name in crate::options::files::SWITCHES {
            assert!(
                SWITCHES.iter().any(|s| s.name == name && s.value.is_none()),
                "{name} is read by the options but not listed"
            );
        }
    }
}
