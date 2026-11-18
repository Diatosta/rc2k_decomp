//! The settings in the code: the options file's text with only what differs from a new
//! install's options, so a player who changed a few things carries a few lines.
//!
//! Not in it: what belongs to the machine (the screen modes, the window, the renderer, the
//! network's addresses and name, the controllers and their bindings) and the records
//! (`best`); the importing machine keeps its own.

use super::CodeError;
use crate::options::document::Document;
use crate::options::{Options, toml_file};
use toml::{Table, Value};

/// The most text a code may carry for the settings.
pub const MOST_TEXT: usize = 64 * 1024;

/// `options` as the code's settings see them: the machine's parts and the records put back to
/// a new install's.
pub fn portable(options: &Options) -> Options {
    let fresh = Options::default();
    let mut out = options.clone();
    out.display = fresh.display;
    out.network = fresh.network;
    out.records = fresh.records;
    out.players.devices = fresh.players.devices;
    out.players.profiles = fresh.players.profiles;
    out
}

fn table_of(options: &Options) -> Result<Table, CodeError> {
    toml_file::write(options, None)
        .map_err(|_| CodeError::Range("options that cannot be written"))?
        .parse::<Table>()
        .map_err(|_| CodeError::Range("options that cannot be written"))
}

/// What of `table` differs from `base`: a table only where something under it does.
fn difference(table: &Table, base: &Table) -> Table {
    let mut out = Table::new();
    for (key, value) in table {
        match (value, base.get(key)) {
            (Value::Table(a), Some(Value::Table(b))) => {
                let inner = difference(a, b);
                if !inner.is_empty() {
                    out.insert(key.clone(), Value::Table(inner));
                }
            }
            (v, Some(b)) if v == b => {}
            (v, _) => {
                out.insert(key.clone(), v.clone());
            }
        }
    }
    out
}

/// `into` with every key of `over` set: tables merged key by key, anything else replaced.
fn merge(into: &mut Table, over: Table) {
    for (key, value) in over {
        match (into.get_mut(&key), value) {
            (Some(Value::Table(a)), Value::Table(b)) => merge(a, b),
            (_, value) => {
                into.insert(key, value);
            }
        }
    }
}

/// The settings text of `options` (with the parts `portable` leaves out already left out).
pub fn text_of(options: &Options) -> Result<String, CodeError> {
    let options = portable(options);
    let fresh = table_of(&Options::default())?;
    let full = table_of(&options)?;
    let diff = difference(&full, &fresh);
    let text =
        toml::to_string(&diff).map_err(|_| CodeError::Range("options that cannot be written"))?;
    // The difference must give the options back; if a default the file leaves out makes it
    // not, the whole text is carried.
    if options_of(&text, &Options::default()).as_ref() == Ok(&options) {
        Ok(text)
    } else {
        toml::to_string(&full).map_err(|_| CodeError::Range("options that cannot be written"))
    }
}

/// The options a settings text gives, over `current` (whose machine's parts and records stay).
/// Refused: text that is not TOML, a value out of its range or of the wrong type.
pub fn options_of(text: &str, current: &Options) -> Result<Options, CodeError> {
    if text.len() > MOST_TEXT {
        return Err(CodeError::Range("the settings are too long"));
    }
    let given: Table = text
        .parse()
        .map_err(|_| CodeError::Damaged("the settings are not valid text"))?;
    if given.contains_key("records") {
        return Err(CodeError::Range("records among the settings"));
    }
    let mut table = table_of(&Options::default())?;
    merge(&mut table, given);
    // Read strictly: a value out of its range is a refusal here, not a default.
    let Ok(document) = Value::Table(table).try_into::<Document>() else {
        return Err(CodeError::Range("settings the game cannot read"));
    };
    let Ok(mut options) = document.into_options() else {
        return Err(CodeError::Range("settings the game cannot read"));
    };
    options.display = current.display.clone();
    options.network = current.network.clone();
    options.records = current.records.clone();
    options.players.devices = current.players.devices.clone();
    options.players.profiles = current.players.profiles.clone();
    check(&options)?;
    Ok(options)
}

/// The ranges the settings keep to.
fn check(o: &Options) -> Result<(), CodeError> {
    let sound = &o.sound;
    let volumes = [
        sound.master_volume,
        sound.music_volume,
        sound.race_music_volume,
        sound.general_volume,
        sound.menu_music_volume,
        sound.race_music_level,
        sound.menu_effects_volume,
        sound.race_effects_volume,
        sound.menu_voices_volume,
        sound.codriver_voice_volume,
        sound.engine_volume,
    ];
    if volumes.iter().any(|&v| v > 0x100) {
        return Err(CodeError::Range("a volume out of its range"));
    }
    for name in &o.players.names {
        let text = &name.0;
        if !text.contains(&0) && text.len() >= 0x20 {
            return Err(CodeError::Range("a player's name without its end"));
        }
        if text.iter().take_while(|&&b| b != 0).any(|&b| b < 0x20) {
            return Err(CodeError::Range("a player's name with a control character"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed() -> Options {
        let mut o = Options::default();
        o.sound.master_volume = 90;
        o.sound.music_volume = 33;
        o.players.names[0].0[..5].copy_from_slice(b"Ana\0\0");
        o.cars.first_numbers.0[0] = 7;
        o.pages.switches.0[3] = 9;
        o
    }

    #[test]
    fn the_difference_is_short_and_gives_the_options_back() {
        let o = changed();
        let text = text_of(&o).unwrap();
        assert!(text.len() < 600, "{} bytes: {text}", text.len());
        assert_eq!(options_of(&text, &Options::default()).unwrap(), o);
    }

    #[test]
    fn the_machines_parts_and_the_records_stay_with_the_machine() {
        let mut theirs = changed();
        theirs.display.menu_mode = 3;
        theirs.display.window_mode = Some("fullscreen".into());
        theirs.network.most_players = 7;
        theirs.records.tyre_change_time = 99;
        let text = text_of(&theirs).unwrap();
        let mut mine = Options::default();
        mine.display.menu_mode = 5;
        mine.network.most_players = 2;
        mine.records.tyre_change_time = 11;
        let got = options_of(&text, &mine).unwrap();
        assert_eq!(got.display, mine.display);
        assert_eq!(got.network, mine.network);
        assert_eq!(got.records, mine.records);
        assert_eq!(got.sound.master_volume, 90);
    }

    #[test]
    fn bad_settings_are_refused() {
        let cur = Options::default();
        assert!(options_of("sound = 5\n", &cur).is_err());
        assert!(options_of("[sound]\nmaster_volume = \"loud\"\n", &cur).is_err());
        assert!(options_of("[sound]\nmaster_volume = 99999\n", &cur).is_err());
        assert!(options_of("[sound]\nmaster_volume = -1\n", &cur).is_err());
        assert!(options_of("not toml [", &cur).is_err());
        assert!(options_of("[records]\ntyre_change_time = 1\n", &cur).is_err());
        assert!(options_of("[players.player_1]\nname = \"a\\u0001b\"\n", &cur).is_err());
    }
}
