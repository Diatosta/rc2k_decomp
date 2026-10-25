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

/// The three registry calls, so that a test can answer them.
pub trait Registry {
    unsafe fn open(&mut self, name: *const u8, key: *mut u32) -> u32;
    unsafe fn query(
        &mut self,
        key: u32,
        name: *const u8,
        kind: *mut u32,
        data: *mut u8,
        size: *mut u32,
    ) -> u32;
    unsafe fn close(&mut self, key: u32);
}

pub unsafe fn read_registry(registry: &mut dyn Registry) {
    unsafe {
        let key = INSTALLER_RUN.reg_key.addr() as *mut u32;
        if registry.open(INSTALLER_NAMES.key_name.addr() as *const u8, key) != 0 {
            return;
        }
        let (kind, size) = (
            INSTALLER_NAMES.reg_type.addr() as *mut u32,
            INSTALLER_NAMES.source_size.addr() as *mut u32,
        );
        let (status, source) = (
            INSTALLER_NAMES.status.addr() as *mut u8,
            INSTALLER_NAMES.source.addr() as *mut u8,
        );
        let key = key.read_unaligned();
        'values: {
            size.write_unaligned(1);
            let name = INSTALLER_NAMES.status_name.addr() as *const u8;
            if registry.query(key, name, kind, status, size) != 0 {
                break 'values;
            }
            if kind.read_unaligned() != REG_BINARY || size.read_unaligned() != 1 {
                status.write(0);
            }
            size.write_unaligned(SOURCE_ROOM);
            let name = INSTALLER_NAMES.source_name.addr() as *const u8;
            if registry.query(key, name, kind, source, size) != 0 {
                break 'values;
            }
            if kind.read_unaligned() != REG_SZ {
                source.write(0);
            }
        }
        registry.close(key);
    }
}

/// The registry, or in the standalone the portable record (`install::portable`).
pub unsafe fn read_registry_here() {
    unsafe { read_registry(&mut crate::install::portable::Portable) };
}
