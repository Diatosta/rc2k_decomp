//! The installer's variables that stand outside the run's own state:
//! - `InstallVars`: run from the CD, the Clio's car found, and the credits' texts;
//! - the drive page's rows (`DriveRows`).
//!
//! The texts the strings name are read on past a field's end into the next byte, so the
//! credits and the drive rows are each one buffer that holds the bytes after its last text
//! too (`strings::string_table`).

use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::with_runs;
use std::cell::Cell;

/// The Clio's car file, `files\b\obj0570.ro3`.
pub const CLIO_FILE: &[u8] = b"files\\b\\obj0570.ro3";

/// The bytes of the credits' texts: the first texts, the Clio's name, its descriptions in the
/// languages and the rest; one NUL past the last text, which has none of its own.
const CREDITS_LEN: usize = 27 + 27 + 837 + 1;

pub struct InstallVars {
    /// 1 when the game runs from the CD (`install::check_at_start`).
    pub running_from_cd: Cell<u32>,
    /// 1 when the Clio's car file is there.
    pub has_clio: Cell<u32>,
    /// The credits' texts, from the first (a letter, `umlaut`, the writer's name), the car's
    /// name "renault clio sport V6 (A7)" and the Clio's descriptions on.
    pub credits: PlainBuffer<CREDITS_LEN>,
}

impl InstallVars {
    pub const fn new() -> Self {
        InstallVars {
            running_from_cd: Cell::new(0),
            has_clio: Cell::new(0),
            credits: PlainBuffer::new(with_runs::<CREDITS_LEN>(&[
                (0, b"\xe5\x00umlaut\x00\xa7\x00Craig Griffiths"),
                (27, b"renault clio sport V6 (A7)"),
                (54, CREDITS_B),
            ])),
        }
    }
}

const CREDITS_B: &[u8] = b"11.2:1\x00solution F tuned V6\x0aRS 24 valves\x00rear wheel\x00\xb7 Renault Sport developed chassis and\x0a  engine rekindles the spirit of the\x0a  legendary mid-engined Renault 5 turbo\x0a\x0a\xb7 mid-engine layout for optimal weight\x0a  distribution and dynamic performance\x0a\x0a\xb7 the 3 litre V6 24V engine delivers\x0a  over 280bhp in race trim\x0a\x0a\xb7 developed for a one-make racing\x0a  series with a limited production\x0a  run of road-going models\x0a\x0a\x00V6 24 soupapes\x0a\x09de\x02v\x02eloppe par sol\x01ution F\x00traction arri\xe8re\x00\xb7 les ch\xe2ssis et moteur d\xe9velopp\xe9s par\x0a  Renault Sport reprennent l'esprit du\x0a  l\xe9gendaire moteur central de la\x0a  Renault 5 turbo\x0a\x0a\xb7 support moteur central pour une\x0a  parfaite r\xe9partition du poids et des\x0a  performances optimales\x0a\x0a\xb7 le moteur 3 litres, V6 24 soupapes\x0a  produit plus de 280 ch. en course\x0a\x0a\xb7 d\xe9velopp\xe9e pour des courses\x0a  exclusives et en serie limit\xe9e";

/// The drive page's rows, 0x10 bytes each (letter, colon, the text); the 26th row's letter
/// stands in the bytes after the 25 the page holds, which the text of a row runs on into.
pub const DRIVE_ROWS_LEN: usize = 417;

pub const fn drive_rows() -> PlainBuffer<DRIVE_ROWS_LEN> {
    PlainBuffer::new(with_runs::<DRIVE_ROWS_LEN>(&[
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
        (400, b"z:"),
    ]))
}
