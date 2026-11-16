//! The new options file, `var/rc2k.toml`: `version = 1` first, then a table for each area
//! (`display`, `sound`, `network`, `pages`, `cars`, `players`, `records`, `kept`).
//!
//! A key that is not there keeps the value it has in `base` (the game's own at start); a key
//! this program does not know is ignored on reading and kept in the file when it is written
//! over.

use super::{Options, OptionsError};
use toml::{Table, Value};

/// The version this program writes and reads.
pub const VERSION: i64 = 1;

fn text_error(e: impl std::fmt::Display) -> OptionsError {
    OptionsError::Text(e.to_string())
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

fn table_of(options: &Options) -> Result<Table, OptionsError> {
    let mut table = Table::new();
    table.insert("version".into(), Value::Integer(VERSION));
    merge(&mut table, Table::try_from(options).map_err(text_error)?);
    Ok(table)
}

/// The file's text for `options`; `existing` is the file written over (its unknown keys stay).
pub fn write(options: &Options, existing: Option<&str>) -> Result<String, OptionsError> {
    let mut table = existing
        .and_then(|text| text.parse::<Table>().ok())
        .unwrap_or_default();
    merge(&mut table, table_of(options)?);
    toml::to_string(&table).map_err(text_error)
}

/// The options in `text`, the values it has not got taken from `base`.
pub fn read(text: &str, base: &Options) -> Result<Options, OptionsError> {
    let given = text.parse::<Table>().map_err(text_error)?;
    match given.get("version") {
        None => {}
        Some(Value::Integer(v)) if *v > VERSION => return Err(OptionsError::TooNew(*v)),
        Some(Value::Integer(_)) => {}
        Some(_) => return Err(OptionsError::Text("version is not a number".into())),
    }
    let mut table = table_of(base)?;
    merge(&mut table, given);
    table.try_into::<Options>().map_err(text_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Options {
        let mut o = Options::default();
        o.display.menu_mode = 3;
        o.sound.master_volume = 90;
        o.network.machine_name.0[..4].copy_from_slice(b"Zeus");
        o.pages.switches.0[10] = 5;
        o.records.events[11].0[0x47B] = 0xEE;
        o
    }

    #[test]
    fn written_and_read_it_is_the_same() {
        let o = sample();
        let text = write(&o, None).unwrap();
        assert!(text.starts_with("version = 1\n"));
        assert_eq!(read(&text, &Options::default()).unwrap(), o);
    }

    #[test]
    fn missing_keys_keep_the_base_and_unknown_keys_are_ignored() {
        let mut base = sample();
        base.sound.master_volume = 11;
        let text =
            "version = 1\nfuture = 3\n[display]\nrace_mode = 9\nfuture_key = 'x'\n[later]\na = 1\n";
        let o = read(text, &base).unwrap();
        assert_eq!(o.display.race_mode, 9);
        assert_eq!(o.display.menu_mode, 3);
        assert_eq!(o.sound.master_volume, 11);
        assert_eq!(read("", &base).unwrap(), base);
    }

    #[test]
    fn unknown_keys_stay_when_the_file_is_written_over() {
        let old = "version = 1\nfuture = 3\n[later]\na = 1\n";
        let text = write(&sample(), Some(old)).unwrap();
        assert!(text.contains("future = 3") && text.contains("[later]"));
        assert_eq!(read(&text, &Options::default()).unwrap(), sample());
    }

    #[test]
    fn bad_values_and_newer_versions_are_errors() {
        let base = Options::default();
        assert!(matches!(
            read("version = 2", &base),
            Err(OptionsError::TooNew(2))
        ));
        assert!(matches!(
            read("[display]\nmenu_mode = 'a'", &base),
            Err(OptionsError::Text(_))
        ));
        assert!(matches!(
            read("[network]\nmachine_name = 'ab'", &base),
            Err(OptionsError::Text(_))
        ));
        assert!(matches!(
            read("[pages]\nswitches = [1, 2]", &base),
            Err(OptionsError::Text(_))
        ));
        assert!(matches!(read("[[", &base), Err(OptionsError::Text(_))));
    }
}
