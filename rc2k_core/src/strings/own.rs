//! The texts of the pages this game adds to the original's (the controller axes page, the
//! controller buttons page's new rows and the H shifter's switch). The original's language file
//! holds none of them, so they are kept in one file per language, `<name>.rc2k.toml` beside the
//! language's own files (`files/locale`, named after the language's `.LOC` file), and English is
//! built into the program (`assets/locale/english.rc2k.toml`): the text of a language file that
//! lacks a text, or a missing file, is shown in English.
//!
//! A file is TOML, UTF-8, one `key = "text"` for each text, the key's dots standing for tables
//! (`axis_page.dead_zone` is `dead_zone` in `[axis_page]`). `KEYS` names every key and the id a
//! menu item gives for it. The texts are converted to Windows-1252 as the game's own are.
//! A translator adds a language by writing a file, no code.
//!
//! Their string ids are past the language file's (`FIRST ..= LAST`); a menu item names one as it
//! names a text of the file.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The first id of an own text.
pub const FIRST: u32 = 0x2000;
/// The last id of an own text.
pub const LAST: u32 = 0x20FF;

/// Whether `id` is an own text's.
pub fn is_own(id: u32) -> bool {
    (FIRST..=LAST).contains(&id)
}

/// A page's text ids.
pub mod id {
    use super::FIRST;

    /// An empty text.
    pub const BLANK: u32 = FIRST;
    /// The controller page's item that opens the axes page.
    pub const AXES: u32 = FIRST + 1;
    /// The buttons page's item for its second page, and for its first.
    pub const MORE_CONTROLS: u32 = FIRST + 2;
    pub const FIRST_CONTROLS: u32 = FIRST + 3;
    /// A joystick's axis `n` (0 ..= 5), a gamepad's axis `n`.
    pub const JOYSTICK_AXIS: u32 = FIRST + 0x10;
    pub const PAD_AXIS: u32 = FIRST + 0x18;
    /// The axes page's wheel row and its full rotation row.
    pub const WHEEL: u32 = FIRST + 0x20;
    pub const FULL_ROTATION: u32 = FIRST + 0x21;
    /// "yes", "no", "on", "off".
    pub const YES: u32 = FIRST + 0x28;
    pub const NO: u32 = FIRST + 0x29;
    pub const ON: u32 = FIRST + 0x2A;
    pub const OFF: u32 = FIRST + 0x2B;
    /// The row `n` of `Row::LISTED` on the buttons page.
    pub const CONTROL_ROWS: u32 = FIRST + 0x30;
    /// The prompts while a control is learnt: steering axis, pedal or axis, button.
    pub const PROMPT_STEERING: u32 = FIRST + 0x40;
    pub const PROMPT_AXIS: u32 = FIRST + 0x41;
    pub const PROMPT_BUTTON: u32 = FIRST + 0x42;
    /// The axis page's rows: the axis, dead zone, sensitivity, range, invert and the live
    /// position.
    pub const SETTINGS: u32 = FIRST + 0x50;
    /// What a sensitivity does: nothing (linear), finer near the rest, quicker near the rest.
    pub const LINEAR: u32 = FIRST + 0x58;
    pub const GENTLE: u32 = FIRST + 0x59;
    pub const QUICK: u32 = FIRST + 0x5A;
    /// The wheel's unit.
    pub const DEGREES: u32 = FIRST + 0x46;
    /// The buttons page's rows for the H shifter's gears 1 .. 7 and reverse (8 ids).
    pub const GEAR_ROWS: u32 = FIRST + 0x60;
    /// The buttons page's rows for the menu controls (8 ids).
    pub const MENU_ROWS: u32 = FIRST + 0x70;
    /// The controller page's switch for the H shifter.
    pub const H_SHIFTER: u32 = FIRST + 0x68;
}

/// Each text's file key and id.
pub const KEYS: &[(&str, u32)] = &[
    ("blank", id::BLANK),
    ("axes", id::AXES),
    ("more_controls", id::MORE_CONTROLS),
    ("first_controls", id::FIRST_CONTROLS),
    ("wheel", id::WHEEL),
    ("full_rotation", id::FULL_ROTATION),
    ("degrees", id::DEGREES),
    ("yes", id::YES),
    ("no", id::NO),
    ("on", id::ON),
    ("off", id::OFF),
    ("h_shifter", id::H_SHIFTER),
    ("joystick_axis.x", id::JOYSTICK_AXIS),
    ("joystick_axis.y", id::JOYSTICK_AXIS + 1),
    ("joystick_axis.z", id::JOYSTICK_AXIS + 2),
    ("joystick_axis.rx", id::JOYSTICK_AXIS + 3),
    ("joystick_axis.ry", id::JOYSTICK_AXIS + 4),
    ("joystick_axis.rz", id::JOYSTICK_AXIS + 5),
    ("pad_axis.left_stick_x", id::PAD_AXIS),
    ("pad_axis.left_stick_y", id::PAD_AXIS + 1),
    ("pad_axis.right_stick_x", id::PAD_AXIS + 2),
    ("pad_axis.right_stick_y", id::PAD_AXIS + 3),
    ("pad_axis.left_trigger", id::PAD_AXIS + 4),
    ("pad_axis.right_trigger", id::PAD_AXIS + 5),
    ("axis_page.axis", id::SETTINGS),
    ("axis_page.dead_zone", id::SETTINGS + 1),
    ("axis_page.sensitivity", id::SETTINGS + 2),
    ("axis_page.range", id::SETTINGS + 3),
    ("axis_page.invert", id::SETTINGS + 4),
    ("axis_page.position", id::SETTINGS + 5),
    ("sensitivity.linear", id::LINEAR),
    ("sensitivity.gentle", id::GENTLE),
    ("sensitivity.quick", id::QUICK),
    ("control_rows.steering_axis", id::CONTROL_ROWS),
    ("control_rows.accelerate_axis", id::CONTROL_ROWS + 1),
    ("control_rows.brake_axis", id::CONTROL_ROWS + 2),
    ("control_rows.steer_left_button", id::CONTROL_ROWS + 3),
    ("control_rows.steer_right_button", id::CONTROL_ROWS + 4),
    ("control_rows.accelerate_button", id::CONTROL_ROWS + 5),
    ("control_rows.brake_button", id::CONTROL_ROWS + 6),
    ("control_rows.gear_up", id::CONTROL_ROWS + 7),
    ("control_rows.gear_down", id::CONTROL_ROWS + 8),
    ("control_rows.handbrake", id::CONTROL_ROWS + 9),
    ("control_rows.camera", id::CONTROL_ROWS + 10),
    ("control_rows.pause", id::CONTROL_ROWS + 11),
    ("control_rows.lights", id::CONTROL_ROWS + 12),
    ("control_rows.reset_car", id::CONTROL_ROWS + 13),
    ("gear_rows.gear_1", id::GEAR_ROWS),
    ("gear_rows.gear_2", id::GEAR_ROWS + 1),
    ("gear_rows.gear_3", id::GEAR_ROWS + 2),
    ("gear_rows.gear_4", id::GEAR_ROWS + 3),
    ("gear_rows.gear_5", id::GEAR_ROWS + 4),
    ("gear_rows.gear_6", id::GEAR_ROWS + 5),
    ("gear_rows.gear_7", id::GEAR_ROWS + 6),
    ("gear_rows.reverse", id::GEAR_ROWS + 7),
    ("menu_rows.enter", id::MENU_ROWS),
    ("menu_rows.back", id::MENU_ROWS + 1),
    ("menu_rows.up", id::MENU_ROWS + 2),
    ("menu_rows.down", id::MENU_ROWS + 3),
    ("menu_rows.left", id::MENU_ROWS + 4),
    ("menu_rows.right", id::MENU_ROWS + 5),
    ("menu_rows.third_answer", id::MENU_ROWS + 6),
    ("menu_rows.tab", id::MENU_ROWS + 7),
    ("prompt.steering", id::PROMPT_STEERING),
    ("prompt.axis", id::PROMPT_AXIS),
    ("prompt.button", id::PROMPT_BUTTON),
];

/// The English file, built in.
pub const ENGLISH: &str = include_str!("../../../assets/locale/english.rc2k.toml");

/// The texts of a language file, as Windows-1252 bytes by id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct File {
    texts: HashMap<u32, Vec<u8>>,
    /// The keys the file has that `KEYS` does not.
    pub unknown: Vec<String>,
}

/// The keys of `table` and their texts, each key with the tables above it joined by dots.
fn flatten(
    table: &toml::Table,
    prefix: &str,
    into: &mut Vec<(String, String)>,
) -> Result<(), String> {
    for (name, value) in table {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        match value {
            toml::Value::String(text) => into.push((key, text.clone())),
            toml::Value::Table(inner) => flatten(inner, &key, into)?,
            _ => return Err(format!("{key} is not a text")),
        }
    }
    Ok(())
}

impl File {
    /// A language file's texts. Refused: text that is not TOML, a value that is not a text.
    pub fn parse(text: &str) -> Result<File, String> {
        let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
        let mut pairs = Vec::new();
        flatten(&table, "", &mut pairs)?;
        let mut file = File::default();
        for (key, text) in pairs {
            match KEYS.iter().find(|(known, _)| *known == key) {
                Some(&(_, id)) => {
                    file.texts.insert(id, crate::files::ansi::ansi_bytes(&text));
                }
                None => file.unknown.push(key),
            }
        }
        Ok(file)
    }

    /// The text of own id `id` if the file has it.
    pub fn get(&self, id: u32) -> Option<&[u8]> {
        self.texts.get(&id).map(Vec::as_slice)
    }
}

/// The built-in English texts.
pub fn english() -> &'static File {
    static ENGLISH_TEXTS: OnceLock<File> = OnceLock::new();
    ENGLISH_TEXTS.get_or_init(|| {
        // The file is part of the program: a test keeps it parsing.
        File::parse(ENGLISH).unwrap_or_default()
    })
}

/// The text of own id `id`: the language's file's, else English; none for an id that is none's.
pub fn text(id: u32, language: Option<&File>) -> Option<&[u8]> {
    language
        .and_then(|file| file.get(id))
        .or_else(|| english().get(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every language file the repository ships.
    const SHIPPED: &[(&str, &str)] = &[
        ("english", ENGLISH),
        (
            "portuges",
            include_str!("../../../assets/locale/portuges.rc2k.toml"),
        ),
    ];

    #[test]
    fn ids_and_keys_are_unique_and_inside_the_range() {
        for (n, (key, id)) in KEYS.iter().enumerate() {
            assert!(is_own(*id), "{key}");
            assert!(KEYS[n + 1..].iter().all(|(k, i)| k != key && i != id));
        }
    }

    #[test]
    fn the_english_file_has_every_text() {
        let file = File::parse(ENGLISH).unwrap();
        assert!(file.unknown.is_empty(), "{:?}", file.unknown);
        for (key, id) in KEYS {
            assert!(file.get(*id).is_some(), "english lacks {key}");
        }
        assert_eq!(english(), &file);
    }

    #[test]
    fn every_shipped_file_has_known_keys_and_texts() {
        for (name, text) in SHIPPED {
            let file = File::parse(text).unwrap();
            assert!(file.unknown.is_empty(), "{name}: {:?}", file.unknown);
            for (key, id) in KEYS {
                assert!(file.get(*id).is_some(), "{name} lacks {key}");
            }
        }
    }

    #[test]
    fn texts_are_windows_1252_and_fall_back_to_english() {
        let portuguese =
            File::parse("axes = \"eix\u{f5}s\"\nbogus = \"x\"\n[prompt]\nbutton = \"b\"").unwrap();
        assert_eq!(portuguese.unknown, vec!["bogus".to_string()]);
        assert_eq!(text(id::AXES, Some(&portuguese)), Some(&b"eix\xF5s"[..]));
        assert_eq!(text(id::PROMPT_BUTTON, Some(&portuguese)), Some(&b"b"[..]));
        // Not in the file: English.
        assert_eq!(text(id::WHEEL, Some(&portuguese)), Some(&b"wheel"[..]));
        assert_eq!(text(id::WHEEL, None), Some(&b"wheel"[..]));
        assert_eq!(text(FIRST + 0xF0, None), None);
    }

    #[test]
    fn a_value_that_is_not_a_text_is_refused() {
        assert!(File::parse("axes = 3").is_err());
        assert!(File::parse("axes = ").is_err());
    }
}
