//! `settings::read_driver_names` (0x4162DB): other names for the game's drivers, from
//! `var\name.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `name000` .. `name179` (the key is made in place: the
//! number goes over the digits of `aName000`), each followed by up to three words as
//! `system::next_argument` cuts them: a name, of which at most 30 characters are kept in
//! the pool at `unk_95EFC8` (one after another, each with a NUL) and pointed at by
//! `dword_95ECF8[n]` (here the name's byte offset in `NAMES`, not its address); a number under 0x28 for the driver's record at `dword_73B688` and a
//! number under 0x22 for `dword_73B68C` (records of 0x94 bytes). A number that is too
//! large is left out; a missing third word ends the entry. Without the file nothing
//! changes. One caller, `sub_411239`. The owner's install has no such file.
//! Refuses names that together pass the pool's end at `unk_960000` (the original writes
//! on; 180 names of 30 characters would).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::settings::read_text::read_text;
use crate::strings::format_u32::format_u32;
use crate::strings::owner::strings;

// ---- game data ----

/// "var\name.cfg" (0x529D98 in the image), what `system::load_game_data` hands in.
static PATH: &[u8; 13] = b"var\\name.cfg\0";
const DRIVERS: usize = 180;
/// 0x95ECF8 .. 0x960000 in the image (bss): dword_95ECF8, a pointer a driver (`NAMES_LEN`
/// bytes), then the pool of names, unk_95EFC8, up to unk_960000: room for about 130 names of
/// the full length, not for 180. Nothing else reads either.
const NAMES_LEN: usize = DRIVERS * 4;
const POOL_ROOM: usize = 0x960000 - 0x95EFC8;
/// `NAMES`' length.
pub const NAMES_SIZE: usize = NAMES_LEN + POOL_ROOM;

/// The text of a driver's name from its pointer slot: the slot keeps the name's byte offset
/// in `NAMES` (the original kept its address), 0 for none. Bounded to `NAMES`.
pub fn name_at(slot: u32) -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&strings().drivers.names).offset(slot)
}

/// The path of `var\name.cfg` (without its NUL).
pub fn path() -> &'static [u8] {
    &PATH[..PATH.len() - 1]
}

// ---- Rust version ----

pub fn read_driver_names(menus: &Menus, path: &[u8]) -> Result<(), BadInput> {
    let strings = strings();
    let cars = &menus.cars;
    // Converted: `rc2k_core::files::settings::driver_names`; the pool's room checked before
    // anything is written. The key the original makes in place (`aName000`) is left as
    // its last one, "name179".
    {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let entries = rc2k_core::files::settings::driver_names(
            &text.bytes("the driver names' text")?,
            &crate::strings::same_text::case_table(),
        )
        .map_err(|e| BadInput(e.0))?;
        let room: usize = entries
            .iter()
            .flatten()
            .filter_map(|e| e.name.as_ref())
            .map(|n| n.len() + 1)
            .sum();
        if room > POOL_ROOM {
            return Err(BadInput("more driver names than their pool holds"));
        }
        let last = format_u32(DRIVERS as u32 - 1, 0x13);
        crate::pointers::GamePtr::to(&strings.drivers.key)
            .offset(4)
            .write_bytes(&last, "a driver name's key")?;
        let mut pool = name_at(NAMES_LEN as u32);
        let mut pool_used = 0u32;
        for (driver, entry) in entries.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            if let Some(name) = &entry.name {
                crate::pointers::GamePtr::to(&strings.drivers.names)
                    .offset((4 * driver) as u32)
                    .write(NAMES_LEN as u32 + pool_used, "a driver name's pointer")?;
                pool.write_bytes(name, "a driver name")?;
                pool.offset(name.len() as u32)
                    .write(0u8, "a driver name's end")?;
                pool = pool.offset(name.len() as u32 + 1);
                pool_used += name.len() as u32 + 1;
            }
            if let Some(n) = entry.first {
                cars.drivers.drivers[driver].nation.set(n);
            }
            if let Some(n) = entry.second {
                cars.drivers.drivers[driver].car.set(n);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::strings::owner::strings;
    #[test]
    fn the_path_is_the_text_not_the_pointer_to_it() {
        assert_eq!(super::path(), b"var\\name.cfg");
    }

    use crate::moved::image_bytes;

    #[test]
    fn a_name_slot_is_an_offset_inside_the_names_block() {
        // The first pool name's slot value is the pool's start, and the place it names is
        // bounded by the block (the longest read, 31 bytes, fits the pool's room).
        let first = super::NAMES_LEN as u32;
        assert!(first as usize + 31 <= super::NAMES_LEN + super::POOL_ROOM);
        assert!(super::name_at(first).bytes("a name").is_ok());
    }

    #[test]
    fn data_starts_as_the_games() {
        assert_eq!(&image_bytes(0x529D98, 13)[..], &super::PATH[..]);
        let key: Vec<u8> = (0..8).map(|i| strings().drivers.key.get(i)).collect();
        assert_eq!(&image_bytes(0x529DA5, 8)[..], &key[..]);
        assert!(
            image_bytes(0x95ECF8, super::NAMES_LEN + super::POOL_ROOM)
                .iter()
                .all(|&b| b == 0)
        );
    }
}
