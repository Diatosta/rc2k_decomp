//! `settings::read_driver_names` (0x4162DB): other names for the game's drivers, from
//! `var\name.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `name000` .. `name179` (the key is made in place: the
//! number goes over the digits of `aName000`), each followed by up to three words as
//! `system::next_argument` cuts them: a name, of which at most 30 characters are kept in
//! the pool at `unk_95EFC8` (one after another, each with a NUL) and pointed at by
//! `dword_95ECF8[n]`; a number under 0x28 for the driver's record at `dword_73B688` and a
//! number under 0x22 for `dword_73B68C` (records of 0x94 bytes). A number that is too
//! large is left out; a missing third word ends the entry. Without the file nothing
//! changes. One caller, `sub_411239`. The owner's install has no such file.
//! Refuses names that together pass the pool's end at `unk_960000` (the original writes
//! on; 180 names of 30 characters would).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::moved::Place;
use crate::pointers::GamePtr;
use crate::settings::read_text::read_text;
use crate::strings::format_u32::format_u32;

// ---- game data ----

/// "var\name.cfg" (0x529D98 in the image), what `system::load_game_data` hands in.
static PATH: &[u8; 13] = b"var\\name.cfg\0";
/// aName000 (0x529DA5 in the image, 8 bytes): the key, made in place.
pub static KEY: Place<[u8; 8]> = Place::new(*b"name000\0");
const DRIVERS: usize = 180;
/// 0x95ECF8 .. 0x960000 in the image (bss): dword_95ECF8, a pointer a driver (`NAMES_LEN`
/// bytes), then the pool of names, unk_95EFC8, up to unk_960000: room for about 130 names of
/// the full length, not for 180. Nothing else reads either.
const NAMES_LEN: usize = DRIVERS * 4;
const POOL_ROOM: usize = 0x960000 - 0x95EFC8;
pub static NAMES: Place<[u8; NAMES_LEN + POOL_ROOM]> = Place::new([0; NAMES_LEN + POOL_ROOM]);

/// The path of `var\name.cfg`.
pub fn path() -> crate::pointers::GamePtr {
    crate::pointers::GamePtr::to(&PATH)
}

// ---- Rust version ----

pub fn read_driver_names(path: crate::pointers::GamePtr) -> Result<(), BadInput> {
    // Converted: `rc2k_core::files::settings::driver_names`; the pool's room checked before
    // anything is written. The key the original makes in place (`aName000`) is left as
    // its last one, "name179".
    {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let entries = rc2k_core::files::settings::driver_names(
            &GamePtr::new(crate::pointers::game_address(text)).bytes("the driver names' text")?,
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
        crate::pointers::GamePtr::to(&KEY)
            .offset(4)
            .write_bytes(&last, "a driver name's key")?;
        let mut pool = crate::pointers::GamePtr::to(&NAMES).offset(NAMES_LEN as u32);
        for (driver, entry) in entries.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            if let Some(name) = &entry.name {
                crate::pointers::GamePtr::to(&NAMES)
                    .offset((4 * driver) as u32)
                    .write(pool.addr(), "a driver name's pointer")?;
                pool.write_bytes(name, "a driver name")?;
                pool.offset(name.len() as u32)
                    .write(0u8, "a driver name's end")?;
                pool = pool.offset(name.len() as u32 + 1);
            }
            if let Some(n) = entry.first {
                crate::menu::drivers_store::DRIVERS.drivers[driver]
                    .nation
                    .set(n);
            }
            if let Some(n) = entry.second {
                crate::menu::drivers_store::DRIVERS.drivers[driver]
                    .car
                    .set(n);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn data_starts_as_the_games() {
        assert_eq!(&image_bytes(0x529D98, 13)[..], &super::PATH[..]);
        let key = unsafe { *super::KEY.ptr() };
        assert_eq!(&image_bytes(0x529DA5, 8)[..], &key[..]);
        assert!(
            image_bytes(0x95ECF8, super::NAMES_LEN + super::POOL_ROOM)
                .iter()
                .all(|&b| b == 0)
        );
    }
}
