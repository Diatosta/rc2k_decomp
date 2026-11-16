//! The texts of the pages this game adds to the original's (the controller axes page and the
//! controller buttons page's new rows). The language file holds none of them, so they are kept
//! here, in English and in Portuguese, and chosen by the language the game loaded.
//!
//! Their string ids are past the language file's (`FIRST ..= LAST`); a menu item names one as it
//! names a text of the file.

/// The first id of an own text.
pub const FIRST: u32 = 0x2000;
/// The last id of an own text.
pub const LAST: u32 = 0x20FF;

/// The language an own text is given in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Portuguese,
}

impl Language {
    /// The language of a `.loc` file's 0x30 bytes: its name is the text from byte 0x10, "Portugu.s"
    /// for the Portuguese build; any other is English.
    pub fn of_locale(file: &[u8; 0x30]) -> Language {
        if file[0x10..0x12].eq_ignore_ascii_case(b"po") {
            Language::Portuguese
        } else {
            Language::English
        }
    }
}

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
}

/// (id, English, Portuguese); the Portuguese bytes are Windows-1252 as the game's texts are.
const TEXTS: &[(u32, &[u8], &[u8])] = &[
    (id::BLANK, b"", b""),
    (id::AXES, b"axes", b"eixos"),
    (id::MORE_CONTROLS, b"more controls", b"mais controlos"),
    (
        id::FIRST_CONTROLS,
        b"first controls",
        b"primeiros controlos",
    ),
    (id::JOYSTICK_AXIS, b"axis x", b"eixo x"),
    (id::JOYSTICK_AXIS + 1, b"axis y", b"eixo y"),
    (id::JOYSTICK_AXIS + 2, b"axis z", b"eixo z"),
    (id::JOYSTICK_AXIS + 3, b"axis rx", b"eixo rx"),
    (id::JOYSTICK_AXIS + 4, b"axis ry", b"eixo ry"),
    (id::JOYSTICK_AXIS + 5, b"axis rz", b"eixo rz"),
    (id::PAD_AXIS, b"left stick x", b"anal\xF3gico esq. x"),
    (id::PAD_AXIS + 1, b"left stick y", b"anal\xF3gico esq. y"),
    (id::PAD_AXIS + 2, b"right stick x", b"anal\xF3gico dir. x"),
    (id::PAD_AXIS + 3, b"right stick y", b"anal\xF3gico dir. y"),
    (id::PAD_AXIS + 4, b"left trigger", b"gatilho esq."),
    (id::PAD_AXIS + 5, b"right trigger", b"gatilho dir."),
    (id::WHEEL, b"wheel", b"volante"),
    (id::FULL_ROTATION, b"full rotation", b"rota\xE7\xE3o total"),
    (id::SETTINGS, b"axis", b"eixo"),
    (id::SETTINGS + 1, b"dead zone", b"zona morta"),
    (id::SETTINGS + 2, b"sensitivity", b"sensibilidade"),
    (id::SETTINGS + 3, b"range", b"alcance"),
    (id::SETTINGS + 4, b"invert", b"inverter"),
    (id::SETTINGS + 5, b"position", b"posi\xE7\xE3o"),
    (id::LINEAR, b"linear", b"linear"),
    (id::GENTLE, b"gentle", b"suave"),
    (id::QUICK, b"quick", b"r\xE1pido"),
    (id::YES, b"yes", b"sim"),
    (id::NO, b"no", b"n\xE3o"),
    (id::ON, b"on", b"sim"),
    (id::OFF, b"off", b"n\xE3o"),
    (
        id::CONTROL_ROWS,
        b"steering (axis)",
        b"dire\xE7\xE3o (eixo)",
    ),
    (
        id::CONTROL_ROWS + 1,
        b"accelerate (axis)",
        b"acelerar (eixo)",
    ),
    (id::CONTROL_ROWS + 2, b"brake (axis)", b"travar (eixo)"),
    (
        id::CONTROL_ROWS + 3,
        b"steer left (button)",
        b"virar \xE0 esquerda (bot\xE3o)",
    ),
    (
        id::CONTROL_ROWS + 4,
        b"steer right (button)",
        b"virar \xE0 direita (bot\xE3o)",
    ),
    (
        id::CONTROL_ROWS + 5,
        b"accelerate (button)",
        b"acelerar (bot\xE3o)",
    ),
    (
        id::CONTROL_ROWS + 6,
        b"brake (button)",
        b"travar (bot\xE3o)",
    ),
    (id::CONTROL_ROWS + 7, b"gear up", b"mudan\xE7a superior"),
    (id::CONTROL_ROWS + 8, b"gear down", b"mudan\xE7a inferior"),
    (id::CONTROL_ROWS + 9, b"handbrake", b"trav\xE3o de m\xE3o"),
    (id::CONTROL_ROWS + 10, b"camera", b"vistas de c\xE2mara"),
    (id::CONTROL_ROWS + 11, b"pause", b"pausa"),
    (id::CONTROL_ROWS + 12, b"lights", b"far\xF3is"),
    (id::CONTROL_ROWS + 13, b"wipers", b"limpa-p\xE1ra-brisas"),
    (id::CONTROL_ROWS + 14, b"reset car", b"repor carro"),
    (id::PROMPT_STEERING, b"turn right", b"vire \xE0 direita"),
    (id::PROMPT_AXIS, b"press the pedal", b"carregue no pedal"),
    (
        id::PROMPT_BUTTON,
        b"press a button",
        b"carregue num bot\xE3o",
    ),
    (id::DEGREES, b"degrees", b"graus"),
];

/// The text of own id `id` in `language`; none for an id that is none's.
pub fn text(id: u32, language: Language) -> Option<&'static [u8]> {
    TEXTS.iter().find(|(known, _, _)| *known == id).map(
        |&(_, english, portuguese)| match language {
            Language::English => english,
            Language::Portuguese => portuguese,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_inside_the_range() {
        for (n, (id, _, _)) in TEXTS.iter().enumerate() {
            assert!(is_own(*id));
            assert!(TEXTS[n + 1..].iter().all(|(other, _, _)| other != id));
        }
    }

    #[test]
    fn the_language_follows_the_locale_file() {
        let mut file = [0u8; 0x30];
        file[0x10..0x19].copy_from_slice(b"Portugu\xEAs");
        assert_eq!(Language::of_locale(&file), Language::Portuguese);
        file[0x10..0x17].copy_from_slice(b"English");
        assert_eq!(Language::of_locale(&file), Language::English);
        assert_eq!(text(id::AXES, Language::English), Some(&b"axes"[..]));
        assert_eq!(text(id::AXES, Language::Portuguese), Some(&b"eixos"[..]));
        assert_eq!(text(FIRST + 0xF0, Language::English), None);
    }

    #[test]
    fn every_listed_row_has_a_text() {
        for n in 0..15 {
            assert!(text(id::CONTROL_ROWS + n, Language::English).is_some());
        }
    }
}
