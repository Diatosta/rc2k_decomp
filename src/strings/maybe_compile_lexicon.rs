//! `strings::maybe_compile_lexicon` (0x403555): the string table made again from the language
//! file (`strings::compile_lexicon`) when `dword_4E0758` (signed) is 1 or more and
//! `byte_4F6B2A` (a key's state, it seems) is not 0: a developer's way to see a changed
//! language file at once (a reading).
//!
//! Callers: the network's list screens (`net::settings_loop`, `net::choose_connection`,
//! `net::choose_session`, `net::lobby_loop`), for a key they do not know, through `part`
//! (`net::lobby_calls`).
//! Hook removed 2026-10-21 (conversion phase 4); the original keeps every register.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::strings::compile_lexicon::compile_lexicon;

const LEVEL_RVA: usize = rva(0x4E0758); // dword_4E0758
const KEY_RVA: usize = rva(0x4F6B2A); // byte_4F6B2A

pub unsafe fn maybe_compile_lexicon() -> Result<(), Refused> {
    unsafe {
        let (level, key) = (game_read::<u32>(LEVEL_RVA), game_read::<u8>(KEY_RVA));
        // Converted: `rc2k_core::strings::lexicon::recompile_wanted`.
        if rc2k_core::strings::lexicon::recompile_wanted(level, key) {
            compile_lexicon()?;
        }
    }
    Ok(())
}
