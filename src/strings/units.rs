//! The game's unit system and the string ids of its unit labels.

use crate::errors::BadInput;
pub use rc2k_core::math::units::{Figure, Units};

/// 0 = imperial, non-zero = metric. The game stores Celsius and miles. Set by the language
/// (`load_language`) and player 1's choice (`menu::take_player1_units`, `menu::options_loop`).
pub fn units(strings: &crate::strings::owner::Strings) -> u32 {
    strings.language.units.get()
}

pub fn set_units(strings: &crate::strings::owner::Strings, units: u32) {
    strings.language.units.set(units);
}

/// The setting as the core's `Units`; refused past 1 (the label tables have two entries).
pub fn setting(units: u32) -> Result<Units, BadInput> {
    Units::from_setting(units).ok_or(BadInput("a unit setting with no label"))
}

/// A core `Figure` as the bytes the game appends: the text, then the label's string.
pub fn figure_bytes(
    o: crate::strings::append_string_by_id::StringOwners<'_>,
    figure: Figure,
) -> Result<Vec<u8>, BadInput> {
    let mut out = figure.text;
    if let Some(id) = figure.label {
        crate::strings::append_string_by_id::push_string(o, &mut out, id as u32)?;
    }
    Ok(out)
}
