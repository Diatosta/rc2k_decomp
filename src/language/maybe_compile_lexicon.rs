//! Makes the string table again from the language file (`language::compile_lexicon`) when
//! `Lexicon::recompile_level` (signed, a `var\ral.cfg` setting, 0 unless set) is 1 or more and the numpad
//! plus key (`input`'s `keys_down`, given by the caller) is down: a developer's way to see a changed language file at
//! once.

use crate::errors::Refused;
use crate::language::compile_lexicon::compile_lexicon;

/// `plus_key` is the numpad plus key's state (key 0x4E of the input devices' key states).
pub fn maybe_compile_lexicon(
    plus_key: u8,
    strings: &crate::strings::owner::Strings,
    scratch: &crate::files::scratch::Scratch,
) -> Result<(), Refused> {
    let level = strings.lexicon.recompile_level.get();
    if rc2k_core::strings::lexicon::recompile_wanted(level, plus_key) {
        compile_lexicon(strings, scratch)?;
    }
    Ok(())
}
