//! Reads the edition file `files\d\gfx\sku`: a settings text with the key `sku`. Its value may
//! start with the marks `oem:` and `dvd:`, in any order and number, each setting a switch to 1;
//! then a number, after the next `:` a second byte and after the next `.` within 254 characters
//! a third byte. What is not there keeps its value. One caller, `system::load_game_data`.

use crate::errors::BadInput;
use crate::session::{PlainByte, PlainBytes, PlainDword, with_runs};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

pub fn read_sku(
    install: &crate::install::owner::Install,
    system: &crate::system::owner::System,
    path: &[u8],
) -> Result<(), BadInput> {
    let Some(text) = read_text(system, path)? else {
        return Ok(());
    };
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

/// The edition's markers (`install::read_sku`).
#[derive(rc2k_derive::Cells)]
pub struct Sku {
    /// The edition's number.
    pub number: PlainDword,
    /// 1 for an OEM edition.
    pub oem: PlainDword,
    /// 1 for a DVD edition.
    pub dvd: PlainDword,
    /// The sku file's third byte.
    pub third: PlainByte,
    /// Its second byte.
    pub second: PlainByte,
    /// "files\d\gfx\sku", the file read.
    pub path: PlainBytes<16>,
    /// A gap of 6 byte(s): nothing reads it.
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
