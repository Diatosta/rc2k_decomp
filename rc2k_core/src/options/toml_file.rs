//! The new options file, `var/rc2k.toml`: `version = 2` first, then a table for each area
//! (`display`, `sound`, `network`, `pages`, `cars`, `players`, `records`, `kept`).
//!
//! A key that is not there keeps the value it has in `base` (the game's own at start); a key
//! this program does not know is ignored on reading and kept in the file when it is written
//! over. The values are the readable ones of `document`, not the old file's bytes. A file of an
//! older version (the first, with hexadecimal bytes) is not read and is not kept when written
//! over: the options then come from where they come from when there is no file.

use super::document::Document;
use super::{Options, OptionsError};
use toml::{Table, Value};

/// The version this program writes and reads.
pub const VERSION: i64 = 2;

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
    let document = Document::from(options);
    merge(&mut table, Table::try_from(&document).map_err(text_error)?);
    Ok(table)
}

/// The `version` a table names: `None` when it has none (the current one is meant).
fn version_of(table: &Table) -> Result<Option<i64>, OptionsError> {
    match table.get("version") {
        None => Ok(None),
        Some(Value::Integer(v)) => Ok(Some(*v)),
        Some(_) => Err(OptionsError::Text("version is not a number".into())),
    }
}

/// The file's text for `options`; `existing` is the file written over (its unknown keys stay,
/// unless it is of another version).
pub fn write(options: &Options, existing: Option<&str>) -> Result<String, OptionsError> {
    let mut table = existing
        .and_then(|text| text.parse::<Table>().ok())
        .filter(|table| matches!(version_of(table), Ok(None) | Ok(Some(VERSION))))
        .unwrap_or_default();
    merge(&mut table, table_of(options)?);
    toml::to_string(&table).map_err(text_error)
}

/// The options in `text`, the values it has not got taken from `base`. `Ok(None)`: the file is
/// of an older version, which is not read.
pub fn read(text: &str, base: &Options) -> Result<Option<Options>, OptionsError> {
    let given = text.parse::<Table>().map_err(text_error)?;
    match version_of(&given)? {
        Some(v) if v > VERSION => return Err(OptionsError::TooNew(v)),
        Some(v) if v < VERSION => return Ok(None),
        _ => {}
    }
    let mut table = table_of(base)?;
    merge(&mut table, given);
    let document = table.try_into::<Document>().map_err(text_error)?;
    document.into_options().map(Some)
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
        o.records.events[11].0[0x47B] = 0;
        o.records.events[11].0[0x7C] = b'k';
        o
    }

    /// Options whose every byte differs from its neighbours', then made what the file can hold
    /// (texts end at a NUL, floats are numbers).
    fn full_sample(salt: usize) -> Options {
        let lengths = Options::piece_lengths();
        let bytes: Vec<u8> = (0..crate::options::model::PIECES_LEN)
            .map(|n| ((n + salt) * 7 + (n + salt) / 256) as u8)
            .collect();
        let mut pieces = Vec::new();
        let mut at = 0;
        for len in lengths {
            pieces.push(&bytes[at..at + len]);
            at += len;
        }
        let mut raw = Options::default();
        assert!(raw.take_pieces(&pieces));
        Document::from(&raw).into_options().unwrap()
    }

    #[test]
    fn written_and_read_it_is_the_same() {
        let o = sample();
        let text = write(&o, None).unwrap();
        assert!(text.starts_with("version = 2\n"));
        assert_eq!(read(&text, &Options::default()).unwrap(), Some(o));
    }

    #[test]
    fn every_value_comes_back() {
        for salt in [0, 3, 1000] {
            let o = full_sample(salt);
            let text = write(&o, None).unwrap();
            assert_eq!(read(&text, &Options::default()).unwrap(), Some(o.clone()));
            // The old file's bytes are the same from the model read back.
            let back = read(&text, &Options::default()).unwrap().unwrap();
            assert_eq!(back.to_pieces(), o.to_pieces());
        }
    }

    #[test]
    fn texts_guids_and_nones_are_readable() {
        let mut o = Options::default();
        o.network.machine_name.0[..6].copy_from_slice(&[b'J', 0xE9, 0x80, b' ', b'X', b'1']);
        o.network.saved_connection.0 = vec![
            0xe0, 0x5e, 0xe9, 0x36, 0x77, 0x85, 0xcf, 0x11, 0x96, 0x0c, 0x00, 0x80, 0xc7, 0x53,
            0x4e, 0x82,
        ];
        o.cars.first_numbers.0 = vec![u32::MAX; 40];
        o.cars.first_numbers.0[..3].copy_from_slice(&[13, 21, 18]);
        o.cars.second_numbers.0 = vec![5; 20];
        o.players.names[2].0[..4].copy_from_slice(b"Ana\0");
        o.players.names[3].0 = vec![b'z'; 0x20];
        o.players.records[0].0[0x04] = 7;
        o.players.records[1].0[0x1C..0x28]
            .copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        o.pages.arcade_levels_won.0 = vec![1, 0, 1, 0, 0, 0];
        o.records.tyre_change_holder.0[..4].copy_from_slice(b"Karl");
        let text = write(&o, None).unwrap();
        for wanted in [
            "machine_name = \"Jé€ X1\"",
            "saved_connection = \"36E95EE0-8577-11CF-960C-0080C7534E82\"",
            "first_numbers = [13, 21, 18]",
            "name = \"Ana\"",
            "tyre_change_holder = \"Karl\"",
            "arcade_levels_won = [1, 0, 1, 0, 0, 0]",
            "key_scan_codes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]",
            "[players.player_1]",
            "nationality = 7",
        ] {
            assert!(text.contains(wanted), "{wanted} is not in\n{text}");
        }
        let back = read(&text, &Options::default()).unwrap().unwrap();
        assert_eq!(back, o);
        assert_eq!(back.cars.first_numbers.0[3], u32::MAX);
    }

    #[test]
    fn missing_keys_keep_the_base_and_unknown_keys_are_ignored() {
        let mut base = sample();
        base.sound.master_volume = 11;
        base.players.records[0].0[4] = 9;
        let text = "version = 2\nfuture = 3\n[display]\nrace_mode = 9\nfuture_key = 'x'\n[later]\na = 1\n\
                    [players.player_2]\nnationality = 5\n";
        let o = read(text, &base).unwrap().unwrap();
        assert_eq!(o.display.race_mode, 9);
        assert_eq!(o.display.menu_mode, 3);
        assert_eq!(o.sound.master_volume, 11);
        assert_eq!(o.players.records[0].0[4], 9);
        assert_eq!(o.players.records[1].0[4], 5);
        assert_eq!(read("", &base).unwrap(), Some(base));
    }

    #[test]
    fn unknown_keys_stay_when_the_file_is_written_over() {
        let old = "version = 2\nfuture = 3\n[later]\na = 1\n";
        let text = write(&sample(), Some(old)).unwrap();
        assert!(text.contains("future = 3") && text.contains("[later]"));
        assert_eq!(read(&text, &Options::default()).unwrap(), Some(sample()));
    }

    #[test]
    fn a_file_of_the_first_version_is_not_read_nor_kept() {
        let first = "version = 1\nfuture = 3\n[display]\nmenu_mode = 7\n[players]\n\
                     names = [\"41\", \"42\", \"43\", \"44\"]\n";
        assert_eq!(read(first, &Options::default()).unwrap(), None);
        let text = write(&sample(), Some(first)).unwrap();
        assert!(text.starts_with("version = 2\n"));
        assert!(!text.contains("future") && !text.contains("names ="));
        assert_eq!(read(&text, &Options::default()).unwrap(), Some(sample()));
    }

    #[test]
    fn bad_values_and_newer_versions_are_errors() {
        let base = Options::default();
        assert!(matches!(
            read("version = 3", &base),
            Err(OptionsError::TooNew(3))
        ));
        assert!(matches!(
            read("version = 'x'", &base),
            Err(OptionsError::Text(_))
        ));
        for bad in [
            "[display]\nmenu_mode = 'a'",
            "[network]\nmachine_name = 'a very long name, past the sixty-four characters that its room holds'",
            "[network]\nsaved_connection = 'not a guid'",
            "[pages]\nswitches = [1, 2]",
            "[cars]\nfirst_numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41]",
            "[players.player_1]\nkey_scan_codes = [1, 2]",
            "[records]\nchampionship = []",
            "[[",
        ] {
            assert!(
                matches!(read(bad, &base), Err(OptionsError::Text(_))),
                "{bad}"
            );
        }
    }
}
