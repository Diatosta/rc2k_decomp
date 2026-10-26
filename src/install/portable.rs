//! The standalone program's installation record in place of the registry (owner, 2026-10-19:
//! the standalone is portable, a folder that needs nothing outside it; a decided difference
//! from the original).
//!
//! What the original keeps under `HKEY_LOCAL_MACHINE\SOFTWARE\Magnetic Fields\RC99`
//! (`install::write_registry`, `install::read_registry`):
//! - `status` (one byte: 1 installed) is kept in `var\install.cfg` of the installation's
//!   folder, a settings text as `var\ral.zog` (`status=1`);
//! - `source` (the installation's folder) is not kept: read, it is always the folder the
//!   program runs in (the current directory with a `\`, as `files::current_directory`
//!   gives it), so that a copied or moved folder is still its own installation. Written, it
//!   says only where the record goes (the folder being installed to).
//!
//! The DirectPlay lobby's key (`install::register_lobby`, `unregister_lobby`) is not
//! written: the standalone cannot be started by a lobby (its DirectPlay is not lobbied).
//! Started from somewhere that is no installation (a CD), there is no record to find: the
//! installer offers a new installation where the original offered an update.

use crate::install::installer::{REG_CLOSE_KEY, REG_CREATE_KEY, REG_DELETE_KEY, REG_SET_VALUE};
use crate::install::read_registry::Registry;
use std::sync::Mutex;

/// The record, relative to an installation's folder.
pub const FILE: &str = "var\\install.cfg";

/// Whether the standalone runs in the game's download folder (the CD's files, as it is
/// bundled), not an installation: the game's data (`files\d\gfx\jade.bun`) but neither
/// `var\ral.zog` nor `var\install.cfg`. The main loop then enters the installer (owner,
/// 2026-10-20: the folder in place of a CD).
pub fn is_source_folder() -> bool {
    let here = |p: &str| std::path::Path::new(p).exists();
    here("files\\d\\gfx\\jade.bun") && !here("var\\ral.zog") && !here(FILE)
}
/// The original's key, the one kept (the lobby's is not).
const KEY: &[u8] = b"SOFTWARE\\Magnetic Fields\\RC99";
/// The handles handed out for the kept key and for any other.
const KEPT: u32 = 0x5243_3939; // "RC99"
const OTHER: u32 = 0x4F54_4852; // "OTHR"

const ERROR_FILE_NOT_FOUND: u32 = 2;
const ERROR_MORE_DATA: u32 = 234;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

unsafe fn text(at: *const u8) -> Vec<u8> {
    if at.is_null() {
        return Vec::new();
    }
    unsafe { std::ffi::CStr::from_ptr(at as *const _) }
        .to_bytes()
        .to_vec()
}

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

/// A record's text for `status`.
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

/// The registry's reading, from the record in the current folder.
pub struct Portable;

impl Registry for Portable {
    unsafe fn open(&mut self, name: *const u8, key: *mut u32) -> u32 {
        if unsafe { text(name) }.eq_ignore_ascii_case(KEY) && std::path::Path::new(FILE).exists() {
            unsafe { key.write_unaligned(KEPT) };
            0
        } else {
            ERROR_FILE_NOT_FOUND
        }
    }

    unsafe fn query(
        &mut self,
        key: u32,
        name: *const u8,
        kind: *mut u32,
        data: *mut u8,
        size: *mut u32,
    ) -> u32 {
        if key != KEPT {
            return ERROR_FILE_NOT_FOUND;
        }
        let (value, value_kind) = match unsafe { text(name) }.as_slice() {
            b"status" => {
                let record = std::fs::read_to_string(FILE).unwrap_or_default();
                match parse_status(&record) {
                    Some(status) => (vec![status], REG_BINARY),
                    None => return ERROR_FILE_NOT_FOUND,
                }
            }
            b"source" => match here() {
                Some(mut folder) => {
                    folder.push(0);
                    (folder, REG_SZ)
                }
                None => return ERROR_FILE_NOT_FOUND,
            },
            _ => return ERROR_FILE_NOT_FOUND,
        };
        unsafe {
            let room = size.read_unaligned() as usize;
            kind.write_unaligned(value_kind);
            size.write_unaligned(value.len() as u32);
            if room < value.len() {
                return ERROR_MORE_DATA;
            }
            std::ptr::copy_nonoverlapping(value.as_ptr(), data, value.len());
        }
        0
    }

    unsafe fn close(&mut self, _key: u32) {}
}

/// What the kept key is given until it is closed: its status and the folder it goes to.
static PENDING: Mutex<(Option<u8>, Option<Vec<u8>>)> = Mutex::new((None, None));

/// The record written for the status given, in the folder given (else the current one), if
/// that folder has a `var\` (an installation's).
fn write_record(status: u8, folder: Option<Vec<u8>>) {
    let folder = match folder.filter(|f| !f.is_empty()) {
        Some(f) => String::from_utf8_lossy(&f).into_owned(),
        None => String::new(),
    };
    let base = std::path::Path::new(&folder);
    let var = base.join("var");
    let what = if var.is_dir() {
        match std::fs::write(var.join("install.cfg"), record_text(status)) {
            Ok(()) => "written".to_string(),
            Err(e) => format!("not written: {e}"),
        }
    } else {
        "not written: no var folder there".to_string()
    };
    crate::framework::log(&format!(
        "install: the installation's record (status {status}) in \"{folder}\": {what}"
    ));
}

/// The installer's registry calls in the standalone (see the header): the kept key's
/// values gathered and written as the record when it is closed; the lobby's key and a
/// key's removal done without effect. Each answers success, as the registry would.
pub fn api(place: u32, x: &[u32]) -> Option<u32> {
    let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    unsafe {
        match place {
            REG_CREATE_KEY => {
                let kept = text(x[1] as *const u8).eq_ignore_ascii_case(KEY);
                if kept {
                    *pending = (None, None);
                }
                (x[2] as *mut u32).write_unaligned(if kept { KEPT } else { OTHER });
                Some(0)
            }
            REG_SET_VALUE => {
                if x[0] == KEPT {
                    let data = std::slice::from_raw_parts(x[4] as *const u8, x[5] as usize);
                    match text(x[1] as *const u8).as_slice() {
                        b"status" => pending.0 = data.first().copied(),
                        b"source" => {
                            let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                            pending.1 = Some(data[..end].to_vec());
                        }
                        _ => {}
                    }
                }
                Some(0)
            }
            REG_CLOSE_KEY => {
                if x[0] == KEPT
                    && let (Some(status), folder) = std::mem::take(&mut *pending)
                {
                    write_record(status, folder);
                }
                Some(0)
            }
            REG_DELETE_KEY => Some(0),
            _ => None,
        }
    }
}
