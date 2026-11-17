//! The switches that say how the program runs on this machine, not what the game does: which
//! renderer draws (`--renderer`) and whether another copy may run beside this one
//! (`--allow-another`), or runs as a numbered instance with its own saved state (`--instance`). The words after the program's own name, as `menu::launch` takes them.

/// The switches that take a value (the next word, or after `=`).
pub const VALUED: [&str; 3] = ["renderer", "instance", "game-data"];

/// The switches that take none.
pub const PLAIN: [&str; 1] = ["--allow-another"];

/// What the program's words ask for.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Switches {
    /// `--renderer`'s value as given (the last one), for the renderer's choice to check.
    pub renderer: Option<String>,
    /// `--allow-another`: this copy starts although another copy is running in the folder.
    pub allow_another: bool,
    /// `--instance`'s number, 1 ..= `MOST_INSTANCES`: this copy is that instance of the folder.
    pub instance: Option<u32>,
    /// `--game-data`: the folder or zip the game's data is taken from when the program's folder
    /// has none (the first-run page's choice, given on the command line).
    pub game_data: Option<String>,
}

/// The most instances `--instance` names.
pub const MOST_INSTANCES: u32 = 8;

impl Switches {
    /// The switches among `args` (the words after the program's own name). Refused: a
    /// `--renderer` with no value after it.
    pub fn from_args<S: AsRef<str>>(args: &[S]) -> Result<Self, String> {
        let mut found = Self::default();
        let mut i = 0;
        while i < args.len() {
            let word = args[i].as_ref();
            i += 1;
            if PLAIN.contains(&word) {
                found.allow_another = true;
                continue;
            }
            let Some(long) = word.strip_prefix("--") else {
                continue;
            };
            let (key, inline) = match long.split_once('=') {
                Some((k, v)) => (k, Some(v.to_string())),
                None => (long, None),
            };
            // Another switch's value is no switch of ours, whatever it looks like.
            if !VALUED.contains(&key) {
                if inline.is_none() && crate::cli::takes_value(word) {
                    i += 1;
                }
                continue;
            }
            let value = match inline {
                Some(v) => v,
                None => {
                    let v = args
                        .get(i)
                        .ok_or_else(|| format!("--{key} needs a value"))?;
                    i += 1;
                    v.as_ref().to_string()
                }
            };
            if key == "instance" {
                let number = value
                    .parse::<u32>()
                    .ok()
                    .filter(|n| (1..=MOST_INSTANCES).contains(n))
                    .ok_or_else(|| {
                        format!("--instance {value}: a number from 1 to {MOST_INSTANCES} is wanted")
                    })?;
                found.instance = Some(number);
            } else if key == "game-data" {
                found.game_data = Some(value);
            } else {
                found.renderer = Some(value);
            }
        }
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switches_are_read_with_their_values() {
        assert_eq!(Switches::from_args::<&str>(&[]), Ok(Switches::default()));
        let s = Switches::from_args(&["--renderer", "software", "--allow-another"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("software"));
        assert!(s.allow_another);
        let s = Switches::from_args(&["--renderer=gpu", "--renderer", "software"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("software"));
        assert!(!s.allow_another);
    }

    #[test]
    fn an_instance_is_a_small_number() {
        let s = Switches::from_args(&["--instance", "3"]).unwrap();
        assert_eq!(s.instance, Some(3));
        assert_eq!(
            Switches::from_args(&["--instance=1"]).unwrap().instance,
            Some(1)
        );
        assert!(Switches::from_args(&["--instance", "0"]).is_err());
        assert!(Switches::from_args(&["--instance", "9"]).is_err());
        assert!(Switches::from_args(&["--instance=two"]).is_err());
        assert!(Switches::from_args(&["--instance"]).is_err());
    }

    #[test]
    fn a_value_is_not_a_switch_and_may_not_be_missing() {
        // The word after a valued switch is its value, whatever it looks like.
        let s = Switches::from_args(&["--renderer", "--allow-another"]).unwrap();
        assert_eq!(s.renderer.as_deref(), Some("--allow-another"));
        assert!(!s.allow_another);
        assert!(Switches::from_args(&["--renderer"]).is_err());
        // Nor is it ours when another switch takes it.
        let s = Switches::from_args(&["--stage", "--allow-another", "--car=x"]).unwrap();
        assert_eq!(s, Switches::default());
    }
}
