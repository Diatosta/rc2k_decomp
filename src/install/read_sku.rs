//! `install::read_sku` (0x411696): reads the edition file `files\d\gfx\sku`.
//!
//! A settings text with the key `sku`. Its value may start with the marks `oem:` and
//! `dvd:`, in any order and number, each setting a switch to 1 (the OEM and DVD
//! switches); then a number, after the next `:` a second byte and after the next `.`
//! within 254 characters a third byte.
//! What is not there keeps its value. One caller, `system::load_game_data`.

use crate::errors::BadInput;
use crate::session::{PlainByte, PlainBytes, PlainDword, with_runs};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

pub fn read_sku(
    install: &'static crate::install::owner::Install,
    path: &[u8],
) -> Result<(), BadInput> {
    let Some(text) = read_text(path)? else {
        return Ok(());
    };
    // Converted: `rc2k_core::install::config::read_sku` (the key as its constant).
    let Some(sku) = config::read_sku(&text, &case_table())? else {
        return Ok(());
    };
    if sku.oem {
        install.sku.oem.set(1u32);
    }
    if sku.dvd {
        install.sku.dvd.set(1u32);
    }
    install.sku.number.set(sku.number);
    if let Some(second) = sku.second {
        install.sku.second.set(second);
    }
    if let Some(third) = sku.third {
        install.sku.third.set(third);
    }
    Ok(())
}

/// The edition's markers (`install::read_sku`) (IDA 0x5190F4 .. 0x519118, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Sku {
    /// 0x5190F4: `dword_5190F4`: the edition's number.
    pub number: PlainDword,
    /// 0x5190F8: `dword_5190F8`: 1 for an OEM edition.
    pub oem: PlainDword,
    /// 0x5190FC: `dword_5190FC`: 1 for a DVD edition.
    pub dvd: PlainDword,
    /// 0x519100: `byte_519100`: the sku file's third byte.
    pub third: PlainByte,
    /// 0x519101: `byte_519101`: its second byte.
    pub second: PlainByte,
    /// 0x519102: `aFilesDGfxSku`: "files\d\gfx\sku", the file read.
    pub path: PlainBytes<16>,
    /// 0x519112: Not named: 6 bytes (a gap, or data nothing reads here).
    pub _gap_519112: PlainBytes<6>,
}

impl Sku {
    pub const fn new() -> Self {
        Sku {
            number: PlainDword::new(0xffffffff),
            oem: PlainDword::new(0x0),
            dvd: PlainDword::new(0x0),
            third: PlainByte::new(0x0),
            second: PlainByte::new(0x0),
            path: PlainBytes::new(with_runs::<16>(&[(0, b"files\\d\\gfx\\sku")])),
            _gap_519112: PlainBytes::new(with_runs::<6>(&[(0, b"sku")])),
        }
    }
}

impl Sku {
    pub const START: usize = 0x5190F4;
    pub const LEN: usize = 0x24;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().sku) as usize
    }
}

#[cfg(test)]
mod sku_tests {
    use super::*;

    #[test]
    fn sku_starts_as_the_games() {
        let size = size_of::<Sku>();
        assert!(Sku::LEN <= size && size <= (Sku::LEN).next_multiple_of(4));
        let image = crate::moved::image_bytes(Sku::START, Sku::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(crate::pointers::host_at(Sku::home()).cast_const(), Sku::LEN)
        };
        assert_eq!(mine, &image[..]);
    }
}
