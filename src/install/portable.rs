//! The standalone program's installation record in place of the registry: the standalone is
//! portable, a folder that needs nothing outside it. `status` (1 installed) is kept in
//! `var\install.cfg`; `source` is not kept: read, it is always the folder the program runs in,
//! so that a copied or moved folder is still its own installation; written, it says only where
//! the record goes. The DirectPlay lobby's key is
//! not written. Started from somewhere that is no installation (a CD), there is no record to
//! find: the installer offers a new installation.

use crate::install::read_registry::{Query, Registry};

/// The record, relative to an installation's folder.
pub const FILE: &str = "var\\install.cfg";

/// Whether the standalone runs in the game's download folder (the CD's files, as it is
/// bundled), not an installation: the game's data (`files\d\gfx\jade.bun`) but neither
/// `var\ral.zog` nor `var\install.cfg`. The main loop then enters the installer (owner:
/// the folder in place of a CD).
pub fn is_source_folder() -> bool {
    let here = |p: &str| crate::files::host_path(p.as_bytes()).exists();
    here("files\\d\\gfx\\jade.bun") && !here("var\\ral.zog") && !here(FILE)
}
/// The key that is kept (the lobby's is not).
const KEY: &[u8] = b"SOFTWARE\\Magnetic Fields\\RC99";
/// The handles handed out for the kept key and for any other.
const KEPT: u32 = 0x5243_3939; // "RC99"
const OTHER: u32 = 0x4F54_4852; // "OTHR"

const ERROR_FILE_NOT_FOUND: u32 = 2;
const ERROR_MORE_DATA: u32 = 234;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

/// The status in a record's text (`status=N`), if it is there.
pub fn parse_status(record: &str) -> Option<u8> {
    record.lines().find_map(|line| {
        let line = line.trim();
        let value = line
            .strip_prefix("status")?
            .trim_start()
            .strip_prefix('=')?;
        value.trim().parse::<u8>().ok()
    })
}

pub fn record_text(status: u8) -> String {
    format!(
        "; Rally Championship\r\n; INSTALLATION STATUS (the standalone's, in place of the registry)\r\n\r\nstatus={status}\r\n"
    )
}

/// The current directory with a `\` after it.
fn here() -> Option<Vec<u8>> {
    let mut d = crate::install::normalise_path::current_directory().ok()??;
    if !matches!(d.last(), Some(b'\\' | b'/')) {
        d.push(b'\\');
    }
    Some(d)
}

/// The registry's answers, from the record in the current folder; its writes, gathered
/// for the kept key and written as the record when it is closed (the lobby's key and a
/// key's removal done without effect, each answering success as the registry would).
#[derive(Default)]
pub struct Portable {
    /// What the kept key is given until it is closed: its status and the folder it goes to.
    pending: Pending,
}

/// A status and the folder of the record it is written to.
type Pending = (Option<u8>, Option<Vec<u8>>);

impl Registry for Portable {
    fn open(&mut self, name: &[u8]) -> Result<u32, u32> {
        if name.eq_ignore_ascii_case(KEY) && crate::files::host_path(FILE.as_bytes()).exists() {
            Ok(KEPT)
        } else {
            Err(ERROR_FILE_NOT_FOUND)
        }
    }

    fn query(&mut self, key: u32, name: &[u8], room: u32) -> Query {
        let missing = Query {
            result: ERROR_FILE_NOT_FOUND,
            kind: None,
            size: None,
            data: None,
        };
        if key != KEPT {
            return missing;
        }
        let (value, kind) = match name {
            b"status" => {
                let record = std::fs::read_to_string(crate::files::host_path(FILE.as_bytes()))
                    .unwrap_or_default();
                match parse_status(&record) {
                    Some(status) => (vec![status], REG_BINARY),
                    None => return missing,
                }
            }
            b"source" => match here() {
                Some(mut folder) => {
                    folder.push(0);
                    (folder, REG_SZ)
                }
                None => return missing,
            },
            _ => return missing,
        };
        let fits = value.len() <= room as usize;
        Query {
            result: if fits { 0 } else { ERROR_MORE_DATA },
            kind: Some(kind),
            size: Some(value.len() as u32),
            data: fits.then_some(value),
        }
    }

    fn close(&mut self, key: u32) {
        if key == KEPT
            && let (Some(status), folder) = std::mem::take(&mut self.pending)
        {
            write_record(status, folder);
        }
    }

    fn create(&mut self, name: &[u8]) -> Result<u32, u32> {
        let kept = name.eq_ignore_ascii_case(KEY);
        if kept {
            self.pending = (None, None);
        }
        Ok(if kept { KEPT } else { OTHER })
    }

    fn set_value(&mut self, key: u32, name: &[u8], _kind: u32, data: &[u8]) -> u32 {
        if key == KEPT {
            let pending = &mut self.pending;
            match name {
                b"status" => pending.0 = data.first().copied(),
                b"source" => {
                    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                    pending.1 = Some(data[..end].to_vec());
                }
                _ => {}
            }
        }
        0
    }

    fn delete(&mut self, _name: &[u8]) -> u32 {
        0
    }
}

/// The record written for the status given, in the folder given (else the current one), if
/// that folder has a `var\` (an installation's).
fn write_record(status: u8, folder: Option<Vec<u8>>) {
    let folder = folder.filter(|f| !f.is_empty()).unwrap_or_default();
    let mut var = folder.clone();
    if !var.is_empty() && !var.ends_with(b"\\") {
        var.push(b'\\');
    }
    var.extend(b"var");
    let var = crate::files::host_path(&var);
    let folder = crate::files::ansi_text(&folder);
    let what = if var.is_dir() {
        match std::fs::write(var.join("install.cfg"), record_text(status)) {
            Ok(()) => "written".to_string(),
            Err(e) => format!("not written: {e}"),
        }
    } else {
        "not written: no var folder there".to_string()
    };
    crate::logfile::program_log::log(&format!(
        "install: the installation's record (status {status}) in \"{folder}\": {what}"
    ));
}
