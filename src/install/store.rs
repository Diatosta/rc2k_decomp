//! The installer's variables, in the old layout's order around the network's block:
//! - `InstallVars`: run from the CD, the Clio's car (file, script, name), and the credits'
//!   texts with their 64 pointers (nothing reads those);
//! - `DriveRows`: the drive page's rows;
//! - `LobbyKeys`: the registry key and value names a lobby registration writes.
//!
//! Structs of cells: the texts of `InstallVars` and `DriveRows` that the strings name are read
//! on past a field into the next, in the order the fields are declared
//! (`strings::string_table`). `InstallVars` keeps a one-byte lead so that every field has the
//! alignment it had.

use crate::memory::session::{PlainByte, PlainBytes, PlainDword, with_runs};

#[derive(rc2k_derive::Cells)]
pub struct InstallVars {
    /// Padding so that the dwords are aligned.
    pub _lead: PlainByte,
    /// A gap of 3 byte(s): nothing reads it.
    pub _gap_529dad: PlainBytes<3>,
    /// 1 when the game runs from the CD (`install::check_at_start`).
    pub running_from_cd: PlainDword,
    /// 1 when `files\b\obj0570.ro3` (the Clio's car) is there.
    pub has_clio: PlainDword,
    /// The Clio's car file, `files\b\obj0570.ro3`.
    pub clio_file: PlainBytes<20>,
    /// The installer's script for the Clio (`install::install_clio`).
    pub clio_script: PlainBytes<28>,
    /// 64 pointers to the texts of string ids 0x1000 .. 0x103F; the texts are found by
    /// `strings::string_table::made_text`, so the bytes are not kept.
    pub _gap_529de8: PlainBytes<256>,
    /// The credits' first texts.
    pub credits_a: PlainBytes<27>,
    /// "renault clio sport V6 (A7)" (`install::check_clio_disc` rewrites it).
    pub clio_car_name: PlainBytes<27>,
    /// The Clio's descriptions in the languages, and the credits' texts.
    pub credits_b: PlainBytes<837>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<1>,
}

impl InstallVars {
    pub const fn new() -> Self {
        InstallVars {
    _lead: PlainByte::new(0),
    _gap_529dad: PlainBytes::new([0; 3]),
    running_from_cd: PlainDword::new(0),
    has_clio: PlainDword::new(0),
    clio_file: PlainBytes::new(with_runs::<20>(&[(0, b"files\\b\\obj0570.ro3")])),
    clio_script: PlainBytes::new(with_runs::<28>(&[(0, b"\x03ral.exe\x03*.txt\x03*.pdf\x04files\x05")])),
    _gap_529de8: PlainBytes::new([0; 256]),
    credits_a: PlainBytes::new(with_runs::<27>(&[(0, b"\xe5\x00umlaut\x00\xa7\x00Craig Griffiths")])),
    clio_car_name: PlainBytes::new(with_runs::<27>(&[(0, b"renault clio sport V6 (A7)")])),
    credits_b: PlainBytes::new(with_runs::<837>(&[(0, b"11.2:1\x00solution F tuned V6\x0aRS 24 valves\x00rear wheel\x00\xb7 Renault Sport developed chassis and\x0a  engine rekindles the spirit of the\x0a  legendary mid-engined Renault 5 turbo\x0a\x0a\xb7 mid-engine layout for optimal weight\x0a  distribution and dynamic performance\x0a\x0a\xb7 the 3 litre V6 24V engine delivers\x0a  over 280bhp in race trim\x0a\x0a\xb7 developed for a one-make racing\x0a  series with a limited production\x0a  run of road-going models\x0a\x0a\x00V6 24 soupapes\x0a\x09de\x02v\x02eloppe par sol\x01ution F\x00traction arri\xe8re\x00\xb7 les ch\xe2ssis et moteur d\xe9velopp\xe9s par\x0a  Renault Sport reprennent l'esprit du\x0a  l\xe9gendaire moteur central de la\x0a  Renault 5 turbo\x0a\x0a\xb7 support moteur central pour une\x0a  parfaite r\xe9partition du poids et des\x0a  performances optimales\x0a\x0a\xb7 le moteur 3 litres, V6 24 soupapes\x0a  produit plus de 280 ch. en course\x0a\x0a\xb7 d\xe9velopp\xe9e pour des courses\x0a  exclusives et en serie limit\xe9e")])),
    _tail: PlainBytes::new([0; 1]),
        }
    }
}

#[derive(rc2k_derive::Cells)]
pub struct DriveRows {
    /// The installer's drive rows, 0x10 bytes each (letter, a byte, the text).
    pub drive_rows: PlainBytes<400>,
    /// A gap of 17 byte(s): nothing reads it.
    pub _gap_52a49f: PlainBytes<17>,
}

impl DriveRows {
    pub const fn new() -> Self {
        DriveRows {
            drive_rows: PlainBytes::new(with_runs::<400>(&[
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
            _gap_52a49f: PlainBytes::new(with_runs::<17>(&[(0, b"z:")])),
        }
    }
}

/// Plain immutable texts, each NUL-ended in its room (read with `text_of`).
pub struct LobbyKeys {
    /// "SOFTWARE\Microsoft\DirectPlay\Applications\RC" (`install::register_lobby`).
    pub registry_key: [u8; 74],
    /// "Guid".
    pub name_guid: [u8; 5],
    /// "File".
    pub name_file: [u8; 5],
    /// "CommandLine".
    pub name_command_line: [u8; 12],
    /// "Path".
    pub name_path: [u8; 5],
    /// "CurrentDirectory".
    pub name_current_directory: [u8; 17],
    /// A gap of 1 byte(s): nothing reads it.
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
