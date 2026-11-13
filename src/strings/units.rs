//! The game's unit system and the string ids of its unit labels.

use crate::errors::BadInput;
pub use rc2k_core::math::units::{Figure, Units};

/// 0 = imperial (Fahrenheit, miles) and non-zero = metric (`g_isImperialUnits`, 0x51DE2C, 0 at the start). Set by the
/// language (`load_language`) and player 1's choice (`menu::take_player1_units`,
/// `menu::options_loop`).
pub static UNITS: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

/// 0 = imperial, non-zero = metric. The game stores Celsius and miles.
pub fn units() -> u32 {
    UNITS.get().get()
}

pub fn set_units(units: u32) {
    UNITS.get().set(units);
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
