//! `install::read_sku` (0x411696): reads the edition file `files\d\gfx\sku`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the key `sku`. Its value may start with the marks `oem:` and
//! `dvd:`, in any order and number, each setting a switch to 1 (`dword_5190F8`,
//! `dword_5190FC`); then a number (`dword_5190F4`), after the next `:` a second
//! (`byte_519101`) and after the next `.` within 254 characters a third (`byte_519100`).
//! What is not there keeps its value. One caller, `sub_411239`.

use crate::errors::BadInput;
use crate::session::{Byte, Bytes, Dword, with_runs};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

// ---- Rust version ----

pub fn read_sku(g: &'static crate::install::owner::Install, path: &[u8]) -> Result<(), BadInput> {
    let Some((text, _)) = read_text(path)? else {
        return Ok(());
    };
    // Converted: `rc2k_core::install::config::read_sku` (the key as its constant).
    let Some(sku) = config::read_sku(&text.bytes("the install record")?, &case_table())? else {
        return Ok(());
    };
    if sku.oem {
        g.sku.oem.set(1u32);
    }
    if sku.dvd {
        g.sku.dvd.set(1u32);
    }
    g.sku.number.set(sku.number);
    if let Some(second) = sku.second {
        g.sku.second.set(second);
    }
    if let Some(third) = sku.third {
        g.sku.third.set(third);
    }
    Ok(())
}

/// The edition's markers (`install::read_sku`) (IDA 0x5190F4 .. 0x519118, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Sku {
    /// 0x5190F4: `dword_5190F4`: the edition's number.
    pub number: Dword,
    /// 0x5190F8: `dword_5190F8`: 1 for an OEM edition.
    pub oem: Dword,
    /// 0x5190FC: `dword_5190FC`: 1 for a DVD edition.
    pub dvd: Dword,
    /// 0x519100: `byte_519100`: the sku file's third byte.
    pub third: Byte,
    /// 0x519101: `byte_519101`: its second byte.
    pub second: Byte,
    /// 0x519102: `aFilesDGfxSku`: "files\d\gfx\sku", the file read.
    pub path: Bytes<16>,
    /// 0x519112: Not named: 6 bytes (a gap, or data nothing reads here).
    pub _gap_519112: Bytes<6>,
}

impl Sku {
    pub const fn new() -> Self {
        Sku {
            number: Dword::new(0xffffffff),
            oem: Dword::new(0x0),
            dvd: Dword::new(0x0),
            third: Byte::new(0x0),
            second: Byte::new(0x0),
            path: Bytes::new(with_runs::<16>(&[(0, b"files\\d\\gfx\\sku")])),
            _gap_519112: Bytes::new(with_runs::<6>(&[(0, b"sku")])),
        }
    }
}

impl Sku {
    pub const START: usize = 0x5190F4;
    pub const LEN: usize = 0x24;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::install::owner::INSTALL.sku) as usize
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
