//! Makes the string table again from the language file (`strings::compile_lexicon`) when
//! `RECOMPILE_LEVEL` (signed, a `var\ral.cfg` setting, 0 unless set) is 1 or more and the numpad
//! plus key (`input`'s `keys_down`) is down: a developer's way to see a changed language file at
//! once.

use crate::errors::Refused;
use crate::strings::compile_lexicon::compile_lexicon;

/// 1 or more (signed) to recompile on the key: a `var\ral.cfg` setting, 0 unless set.
pub static RECOMPILE_LEVEL: crate::session::GameThread<crate::session::PlainDword> =
    crate::session::GameThread::new(crate::session::PlainDword::new(0));

pub fn maybe_compile_lexicon(
    input: &crate::input::owner::Input,
    strings: &crate::strings::owner::Strings,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let level = RECOMPILE_LEVEL.get().get();
    // `input`'s key states (key 0x4E).
    let key = input.devices.keys_down.get(0x4E);
    if rc2k_core::strings::lexicon::recompile_wanted(level, key) {
        compile_lexicon(strings, system)?;
    }
    Ok(())
}
