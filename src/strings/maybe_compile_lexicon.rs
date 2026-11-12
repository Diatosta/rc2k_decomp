//! `strings::maybe_compile_lexicon` (0x403555): the string table made again from the language
//! file (`strings::compile_lexicon`) when `RECOMPILE_LEVEL` (signed) is 1 or more and
//! key 0x4E is down (the numpad plus in DirectInput's codes; the numpad plus in DirectInput's codes: the table
//! `input`'s `keys_down`) is not 0: a developer's way to see a changed language file at once
//! (the level is a `var\ral.cfg` setting, 0 unless set).
//!
//! Callers: the network's list screens (`net::settings_loop`, `net::choose_connection`,
//! `net::choose_session`, `net::lobby_loop`), for a key they do not know, through `part`
//! (`net::lobby_calls`).

use crate::errors::Refused;
use crate::strings::compile_lexicon::compile_lexicon;

/// 1 or more (signed) to recompile on the key: a `var\\ral.cfg` setting
/// (`dword_4E0758` in IDA, 0 unless set).
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
    // Converted: `rc2k_core::strings::lexicon::recompile_wanted`.
    if rc2k_core::strings::lexicon::recompile_wanted(level, key) {
        compile_lexicon(strings, system)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn setting_starts_as_the_games() {
        let d = crate::test_support::first_values::bytes(
            "strings.maybe_compile_lexicon.RECOMPILE_LEVEL/4",
        );
        assert_eq!(
            u32::from_le_bytes(d.try_into().unwrap()),
            super::RECOMPILE_LEVEL.get().get()
        );
    }
}
