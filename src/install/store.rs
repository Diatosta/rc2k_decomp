//! The installer's variables, kept with the image's layout, around
//! the network's block (the build notice between them is `menu::build_notice`'s):
//! - `InstallVars` (IDA 0x529DAD .. 0x52A263): run from the CD, the Clio's car (file, script,
//!   name), and the credits' texts with their 64 pointers (nothing ported reads those);
//! - `DriveRows` (0x52A30F .. 0x52A4B0): the drive page's rows;
//! - `LobbyKeys` (0x5AB419 .. 0x5AB490): the registry key and value names a lobby
//!   registration writes (`install::register_lobby`).
//!
//! `#[repr(C)]` structs of atomic cells with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at them (`home`): the credits'
//! pointers name these texts. `InstallVars` starts on an odd address in the image
//! (0x529DAD): a one-byte lead keeps every image address's alignment.

use crate::session::{Byte, Bytes, PlainDword, with_runs};

/// IDA 0x529DAD .. 0x52A263.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct InstallVars {
    /// Not in the image: one byte so that the dwords are aligned (the block starts on an odd address).
    pub _lead: Byte,
    /// 0x529DAD: Not named: 3 bytes (a gap, or text or data nothing reads here).
    pub _gap_529dad: Bytes<3>,
    /// 0x529DB0: `g_runningFromCd`: 1 when the game runs from the CD (`install::check_at_start`).
    pub running_from_cd: PlainDword,
    /// 0x529DB4: `g_hasClio`: 1 when `files\b\obj0570.ro3` (the Clio's car) is there.
    pub has_clio: PlainDword,
    /// 0x529DB8: `aFilesBObj0570R`: the Clio's car file, `files\b\obj0570.ro3`.
    pub clio_file: Bytes<20>,
    /// 0x529DCC: `byte_529DCC`: the installer's script for the Clio (`install::install_clio`).
    pub clio_script: Bytes<28>,
    /// 0x529DE8: the texts of string ids 0x1000 .. 0x103F (`dword_525DE8[id]`, `strings::string_table::ptr_high`): the IDA address of each, a name.
    pub text_pointers: [PlainDword; 64],
    /// 0x529EE8: The credits' first texts (`aUmlaut`, `aCraigGriffiths`).
    pub credits_a: Bytes<27>,
    /// 0x529F03: `aRenaultClioSpo`: "renault clio sport V6 (A7)" (`install::check_clio_disc` rewrites it).
    pub clio_car_name: Bytes<27>,
    /// 0x529F1E: The Clio's descriptions (`a1121`, `aSolutionFTuned`, `aRearWheel` ..) in the languages, and the credits' texts.
    pub credits_b: Bytes<837>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: Bytes<1>,
}

impl InstallVars {
    pub const fn new() -> Self {
        InstallVars {
    _lead: Byte::new(0),
    _gap_529dad: Bytes::new([0; 3]),
    running_from_cd: PlainDword::new(0),
    has_clio: PlainDword::new(0),
    clio_file: Bytes::new(with_runs::<20>(&[(0, b"files\\b\\obj0570.ro3")])),
    clio_script: Bytes::new(with_runs::<28>(&[(0, b"\x03ral.exe\x03*.txt\x03*.pdf\x04files\x05")])),
    text_pointers: [PlainDword::new(0x52A263), PlainDword::new(0x52A2B1), PlainDword::new(0x52A2E7), PlainDword::new(0xA8BD54), PlainDword::new(0xA8BE54), PlainDword::new(0xA8BF54), PlainDword::new(0xA8C054), PlainDword::new(0xA8C158), PlainDword::new(0xA8C15C), PlainDword::new(0xA8C160), PlainDword::new(0xA8C164), PlainDword::new(0xA8C168), PlainDword::new(0xA8C16C), PlainDword::new(0xA8C170), PlainDword::new(0xA8C174), PlainDword::new(0xA8C178), PlainDword::new(0xA8C17C), PlainDword::new(0x52A30F), PlainDword::new(0x52A31F), PlainDword::new(0x52A32F), PlainDword::new(0x52A33F), PlainDword::new(0x52A34F), PlainDword::new(0x52A35F), PlainDword::new(0x52A36F), PlainDword::new(0x52A37F), PlainDword::new(0x52A38F), PlainDword::new(0x52A39F), PlainDword::new(0x52A3AF), PlainDword::new(0x52A3BF), PlainDword::new(0x52A3CF), PlainDword::new(0x52A3DF), PlainDword::new(0x52A3EF), PlainDword::new(0x52A3FF), PlainDword::new(0x52A40F), PlainDword::new(0x52A41F), PlainDword::new(0x52A42F), PlainDword::new(0x52A43F), PlainDword::new(0x52A44F), PlainDword::new(0x52A45F), PlainDword::new(0x52A46F), PlainDword::new(0x52A47F), PlainDword::new(0x52A48F), PlainDword::new(0x52A49F), PlainDword::new(0x51BEEE), PlainDword::new(0x51BF6E), PlainDword::new(0x51BFEE), PlainDword::new(0x51C06E), PlainDword::new(0x51C0EE), PlainDword::new(0x51C16E), PlainDword::new(0x51C1EE), PlainDword::new(0x51C26E), PlainDword::new(0x51C2EE), PlainDword::new(0x529EE8), PlainDword::new(0x529EEA), PlainDword::new(0x529EF1), PlainDword::new(0x529EF3), PlainDword::new(0x529F03), PlainDword::new(0x529F25), PlainDword::new(0x52A0B7), PlainDword::new(0x529F1E), PlainDword::new(0x529F46), PlainDword::new(0x52A0E3), PlainDword::new(0x529F51), PlainDword::new(0x52A0F4)],
    credits_a: Bytes::new(with_runs::<27>(&[(0, b"\xe5\x00umlaut\x00\xa7\x00Craig Griffiths")])),
    clio_car_name: Bytes::new(with_runs::<27>(&[(0, b"renault clio sport V6 (A7)")])),
    credits_b: Bytes::new(with_runs::<837>(&[(0, b"11.2:1\x00solution F tuned V6\x0aRS 24 valves\x00rear wheel\x00\xb7 Renault Sport developed chassis and\x0a  engine rekindles the spirit of the\x0a  legendary mid-engined Renault 5 turbo\x0a\x0a\xb7 mid-engine layout for optimal weight\x0a  distribution and dynamic performance\x0a\x0a\xb7 the 3 litre V6 24V engine delivers\x0a  over 280bhp in race trim\x0a\x0a\xb7 developed for a one-make racing\x0a  series with a limited production\x0a  run of road-going models\x0a\x0a\x00V6 24 soupapes\x0a\x09de\x02v\x02eloppe par sol\x01ution F\x00traction arri\xe8re\x00\xb7 les ch\xe2ssis et moteur d\xe9velopp\xe9s par\x0a  Renault Sport reprennent l'esprit du\x0a  l\xe9gendaire moteur central de la\x0a  Renault 5 turbo\x0a\x0a\xb7 support moteur central pour une\x0a  parfaite r\xe9partition du poids et des\x0a  performances optimales\x0a\x0a\xb7 le moteur 3 litres, V6 24 soupapes\x0a  produit plus de 280 ch. en course\x0a\x0a\xb7 d\xe9velopp\xe9e pour des courses\x0a  exclusives et en serie limit\xe9e")])),
    _tail: Bytes::new([0; 1]),
        }
    }
}

impl InstallVars {
    pub const START: usize = 0x529DAD;
    pub const LEN: usize = 0x4B6;
}

/// IDA 0x52A30F .. 0x52A4B0.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DriveRows {
    /// 0x52A30F: `byte_52A30F`: the installer's drive rows, 0x10 bytes each (letter, a byte, the text).
    pub drive_rows: Bytes<400>,
    /// 0x52A49F: Not named: 17 bytes (a gap, or text or data nothing reads here).
    pub _gap_52a49f: Bytes<17>,
}

impl DriveRows {
    pub const fn new() -> Self {
        DriveRows {
            drive_rows: Bytes::new(with_runs::<400>(&[
                (0, b"a:"),
                (16, b"b:"),
                (32, b"c:"),
                (48, b"d:"),
                (64, b"e:"),
                (80, b"f:"),
                (96, b"g:"),
                (112, b"h:"),
                (128, b"i:"),
                (144, b"j:"),
                (160, b"k:"),
                (176, b"l:"),
                (192, b"m:"),
                (208, b"n:"),
                (224, b"o:"),
                (240, b"p:"),
                (256, b"q:"),
                (272, b"r:"),
                (288, b"s:"),
                (304, b"t:"),
                (320, b"u:"),
                (336, b"v:"),
                (352, b"w:"),
                (368, b"x:"),
                (384, b"y:"),
            ])),
            _gap_52a49f: Bytes::new(with_runs::<17>(&[(0, b"z:")])),
        }
    }
}

impl DriveRows {
    pub const START: usize = 0x52A30F;
    pub const LEN: usize = 0x1A1;
}

/// IDA 0x5AB419 .. 0x5AB490: plain immutable texts, each NUL-ended in its room (read with
/// `text_of`).
pub struct LobbyKeys {
    /// 0x5AB419: `aSoftwareMicros`: "SOFTWARE\Microsoft\DirectPlay\Applications\RC" (`install::register_lobby`).
    pub registry_key: [u8; 74],
    /// 0x5AB463: `aGuid`.
    pub name_guid: [u8; 5],
    /// 0x5AB468: `aFile`.
    pub name_file: [u8; 5],
    /// 0x5AB46D: `aCommandline`.
    pub name_command_line: [u8; 12],
    /// 0x5AB479: `aPath`.
    pub name_path: [u8; 5],
    /// 0x5AB47E: `aCurrentdirecto`.
    pub name_current_directory: [u8; 17],
    /// 0x5AB48F: Not named: 1 bytes (a gap, or text or data nothing reads here).
    pub _gap_5ab48f: [u8; 1],
}

impl LobbyKeys {
    pub const fn new() -> Self {
        LobbyKeys {
            registry_key: with_runs::<74>(&[(
                0,
                b"SOFTWARE\\Microsoft\\DirectPlay\\Applications\\RC",
            )]),
            name_guid: with_runs::<5>(&[(0, b"Guid")]),
            name_file: with_runs::<5>(&[(0, b"File")]),
            name_command_line: with_runs::<12>(&[(0, b"CommandLine")]),
            name_path: with_runs::<5>(&[(0, b"Path")]),
            name_current_directory: with_runs::<17>(&[(0, b"CurrentDirectory")]),
            _gap_5ab48f: [0; 1],
        }
    }
}

/// The bytes up to the first NUL (or all of them): a text kept in a field of `LobbyKeys`.
pub fn text_of(field: &[u8]) -> &[u8] {
    &field[..field.iter().position(|&c| c == 0).unwrap_or(field.len())]
}

#[cfg(test)]
impl LobbyKeys {
    pub const START: usize = 0x5AB419;
    pub const LEN: usize = 0x77;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    /// The static's bytes (after `lead`) are the image's, its size the range's (with the
    /// tail padding up to the struct's alignment: each range ends at a string's end, so no
    /// read runs on into the next).
    fn check(p: *const u8, size: usize, lead: usize, start: usize, len: usize) {
        let end = lead + len;
        assert!(
            end <= size && size <= end.next_multiple_of(4),
            "{start:#x}: size"
        );
        let image = crate::moved::image_bytes(start, len);
        for (i, &b) in image.iter().enumerate() {
            let ours = unsafe { p.add(lead + i).read() };
            assert_eq!(ours, b, "differs first at {:#x}", start + i);
        }
    }

    #[test]
    fn start_as_the_games_image_and_keep_its_layout() {
        check(
            (&raw const crate::install::owner::INSTALL.get().vars).cast::<u8>(),
            size_of::<InstallVars>(),
            1,
            InstallVars::START,
            InstallVars::LEN,
        );
        assert_eq!(
            offset_of!(InstallVars, running_from_cd),
            1 + 0x529DB0 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, has_clio),
            1 + 0x529DB4 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, clio_file),
            1 + 0x529DB8 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, clio_script),
            1 + 0x529DCC - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, text_pointers),
            1 + 0x529DE8 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, credits_a),
            1 + 0x529EE8 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, clio_car_name),
            1 + 0x529F03 - InstallVars::START
        );
        assert_eq!(
            offset_of!(InstallVars, credits_b),
            1 + 0x529F1E - InstallVars::START
        );
        check(
            (&raw const crate::install::owner::INSTALL.get().drive_rows).cast::<u8>(),
            size_of::<DriveRows>(),
            0,
            DriveRows::START,
            DriveRows::LEN,
        );
        assert_eq!(
            offset_of!(DriveRows, drive_rows),
            0x52A30F - DriveRows::START
        );
        let keys = &crate::install::owner::INSTALL.get().lobby_keys;
        let mine = [
            &keys.registry_key[..],
            &keys.name_guid,
            &keys.name_file,
            &keys.name_command_line,
            &keys.name_path,
            &keys.name_current_directory,
            &keys._gap_5ab48f,
        ]
        .concat();
        assert_eq!(
            mine,
            crate::moved::image_bytes(LobbyKeys::START, LobbyKeys::LEN)
        );
    }
}
