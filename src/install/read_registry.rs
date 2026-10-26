//! `install::read_registry` (0x413D14): what the installer left in the registry.
//!
//! In:  nothing
//! Out: every register preserved
//! Under `HKEY_LOCAL_MACHINE\SOFTWARE\Magnetic Fields\RC99`: the value `status` into
//! `byte_51BB8C` (kept only if it is one byte of binary data, else 0) and the value
//! `source` into `byte_51BB8D` (up to 0xFF bytes; emptied unless it is a text). Without
//! the key nothing changes; if `status` cannot be read, `source` is not asked for. The
//! registry's own answers are left in `Type` (0x51BC94) and `dword_51BC90` (the size).
//! Three callers: the two installation checks and the installer.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_RUN;

// ---- game data ----

const SOURCE_ROOM: u32 = 0xFF;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

// ---- Rust version ----

/// The three registry calls, so that a test can answer them: the game's addresses of
/// the names, the key, the type, the data and its size.
pub trait Registry {
    fn open(&mut self, name: u32, key: u32) -> u32;
    fn query(&mut self, key: u32, name: u32, kind: u32, data: u32, size: u32) -> u32;
    fn close(&mut self, key: u32);
}

fn get(at: u32) -> u32 {
    crate::pointers::read_or_end(at, "install::read_registry's operand")
}

fn set<T: Copy>(at: u32, value: T) {
    crate::pointers::write_or_end(at, value, "install::read_registry's operand")
}

pub fn read_registry(registry: &mut dyn Registry) {
    let key = INSTALLER_RUN.reg_key.addr();
    if registry.open(INSTALLER_NAMES.key_name.addr(), key) != 0 {
        return;
    }
    let (kind, size) = (
        INSTALLER_NAMES.reg_type.addr(),
        INSTALLER_NAMES.source_size.addr(),
    );
    let (status, source) = (INSTALLER_NAMES.status.addr(), INSTALLER_NAMES.source.addr());
    let key = get(key);
    'values: {
        set(size, 1u32);
        let name = INSTALLER_NAMES.status_name.addr();
        if registry.query(key, name, kind, status, size) != 0 {
            break 'values;
        }
        if get(kind) != REG_BINARY || get(size) != 1 {
            set(status, 0u8);
        }
        set(size, SOURCE_ROOM);
        let name = INSTALLER_NAMES.source_name.addr();
        if registry.query(key, name, kind, source, size) != 0 {
            break 'values;
        }
        if get(kind) != REG_SZ {
            set(source, 0u8);
        }
    }
    registry.close(key);
}

/// The registry, or in the standalone the portable record (`install::portable`).
pub fn read_registry_here() {
    read_registry(&mut crate::install::portable::Portable);
}
