//! What the game reads about its installation: the record `var\ral.zog`, the languages
//! file `lang.cfg`, the edition file `files\d\gfx\sku`.

use crate::framework::{GameHook, game_addr, rva};
use crate::redirect::RedirectedCopy;

pub mod check_at_start;
pub mod check_clio_disc;
pub mod check_test;
pub mod choose_language;
pub mod normalise_path;
pub mod outside;
pub mod read_languages;
pub mod read_record;
pub mod read_registry;
pub mod read_sku;
pub mod runs_from_source;
pub mod stand_ins;

pub static HOOKS: &[&dyn GameHook] = &[
    &read_record::HOOK,
    &choose_language::HOOK,
    &read_languages::HOOK,
    &read_sku::HOOK,
    &normalise_path::HOOK,
    &read_registry::HOOK,
    &runs_from_source::HOOK,
    &check_at_start::HOOK,
    &check_clio_disc::HOOK,
];

/// byte_5189A8: base directory A (`cdrom` in the record), and byte_518AA8: what a `%` in a
/// path stands for (the language's directory). 0x100 bytes each.
pub const BASE_DIRECTORY_RVA: usize = rva(0x5189A8);
pub const LOCALE_RVA: usize = rva(0x518AA8);
pub const PATH_ROOM: usize = 0x100;

/// For the self-tests of the three readers: a copy of the original that reads the file
/// at `path` and not its own. All three start `pusha; lea eax, <path>`.
pub unsafe fn copy_reading(
    original: usize,
    len: usize,
    own_path: usize,
    calls: &[usize],
    path: &'static [u8],
) -> Result<RedirectedCopy, &'static str> {
    unsafe {
        let named = (game_addr(own_path) as u32).to_le_bytes();
        let entry = [
            0x60, 0x8D, 0x05, named[0], named[1], named[2], named[3], 0xE8,
        ];
        let copy = RedirectedCopy::new(game_addr(original), len, &entry, calls, &|_| None)?;
        copy.patch_u32(3, path.as_ptr() as u32);
        Ok(copy)
    }
}
