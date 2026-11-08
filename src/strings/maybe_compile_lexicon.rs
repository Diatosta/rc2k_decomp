//! `strings::maybe_compile_lexicon` (0x403555): the string table made again from the language
//! file (`strings::compile_lexicon`) when `dword_4E0758` (signed) is 1 or more and
//! `byte_4F6B2A` (a key's state, it seems) is not 0: a developer's way to see a changed
//! language file at once (a reading).
//!
//! Callers: the network's list screens (`net::settings_loop`, `net::choose_connection`,
//! `net::choose_session`, `net::lobby_loop`), for a key they do not know, through `part`
//! (`net::lobby_calls`).

use crate::errors::Refused;
use crate::input::owner::input;
use crate::strings::compile_lexicon::compile_lexicon;

/// 1 or more (signed) to recompile on the key: a `var\\ral.cfg` setting, out of the image
/// since 2026-10-24 (phase 5; it was `dword_4E0758`, 0 there; set through
/// `crate::moved::place`).
pub static RECOMPILE_LEVEL: crate::session::Dword = crate::session::Dword::new(0);

pub fn maybe_compile_lexicon() -> Result<(), Refused> {
    let level = RECOMPILE_LEVEL.get();
    // byte_4F6B2A: `input`'s key states (key 0x4E).
    let key = input().devices.keys_down.get(0x4E);
    // Converted: `rc2k_core::strings::lexicon::recompile_wanted`.
    if rc2k_core::strings::lexicon::recompile_wanted(level, key) {
        compile_lexicon()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn setting_starts_as_the_games() {
        let d = crate::moved::image_bytes(0x4E0758, 4);
        assert_eq!(
            u32::from_le_bytes(d.try_into().unwrap()),
            super::RECOMPILE_LEVEL.get()
        );
    }
}
