//! Other names for the game's drivers, from `var\name.cfg`: the keys `name000` .. `name179`, each
//! followed by up to three words: a name (at most 30 characters, kept in the names' pool and
//! pointed at by the driver's slot as a byte offset in `NAMES`), a number under 0x28 for the
//! driver's first record number and one under 0x22 for its second. A number too large is left
//! out; a missing word ends the entry.

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::settings::read_text::read_text;
use crate::strings::format_u32::format_u32;
use rc2k_core::files::settings::DriverNames;

// ---- game data ----

/// "var\name.cfg" (0x529D98 in the image), what `system::load_game_data` hands in.
static PATH: &[u8; 13] = b"var\\name.cfg\0";
const DRIVERS: usize = 180;
/// The image's block (0x95ECF8 .. 0x960000, bss): a pointer a driver (`NAMES_LEN`
/// bytes), then the pool of names, up to the block's end: room for about 130 names of
/// the full length, not for 180. Nothing else reads either.
const NAMES_LEN: usize = DRIVERS * 4;
const POOL_ROOM: usize = 0x960000 - 0x95EFC8;
/// `NAMES`' length.
pub const NAMES_SIZE: usize = NAMES_LEN + POOL_ROOM;

/// The path of `var\name.cfg` (without its NUL).
pub fn path() -> &'static [u8] {
    &PATH[..PATH.len() - 1]
}

/// The names file's entries, or `None` without the file. Refused: names that together pass the
/// pool's end.
pub fn read_driver_names(
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<Option<DriverNames>, BadInput> {
    let Some(text) = read_text(system, path)? else {
        return Ok(None);
    };
    let names = rc2k_core::files::settings::read_driver_names(
        &text,
        &crate::strings::same_text::case_table(),
    )
    .map_err(|e| BadInput(e.0))?;
    let room: usize = names
        .entries
        .iter()
        .flatten()
        .filter_map(|e| e.name.as_ref())
        .map(|n| n.len() + 1)
        .sum();
    if room > POOL_ROOM {
        return Err(BadInput("more driver names than their pool holds"));
    }
    Ok(Some(names))
}

/// Puts the names into the pool, each driver's slot at its name, and the drivers' two numbers
/// into their records. The key buffer is left as its last key, "name179".
pub fn apply_driver_names(
    menus: &Menus,
    strings: &crate::strings::owner::Strings,
    names: &DriverNames,
) -> Result<(), BadInput> {
    let cars = &menus.cars;
    let last = format_u32(DRIVERS as u32 - 1, 0x13);
    strings
        .drivers
        .key
        .write_at(4, &last, "a driver name's key")?;
    let pool = &strings.drivers.names;
    let mut pool_used = 0usize;
    for (driver, entry) in names.entries.iter().enumerate() {
        let Some(entry) = entry else {
            continue;
        };
        if let Some(name) = &entry.name {
            pool.set_u32(4 * driver, (NAMES_LEN + pool_used) as u32);
            let mut text = name.clone();
            text.push(0);
            pool.write_at(NAMES_LEN + pool_used, &text, "a driver name")?;
            pool_used += text.len();
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

#[cfg(test)]
mod tests {
    #[test]
    fn the_path_is_the_text_not_the_pointer_to_it() {
        assert_eq!(super::path(), b"var\\name.cfg");
    }

    #[test]
    fn a_name_slot_is_an_offset_inside_the_names_block() {
        // The first pool name's slot value is the pool's start, and the place it names is
        // bounded by the block (the longest read, 31 bytes, fits the pool's room).
        let first = super::NAMES_LEN as u32;
        assert!(first as usize + 31 <= super::NAMES_LEN + super::POOL_ROOM);
        let names: &dyn crate::cells::Store = &crate::test_support::own::strings().drivers.names;
        assert!((first as usize) < names.size());
    }
}
