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

// ---- game data ----

const SOURCE_ROOM: u32 = 0xFF;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

// ---- Rust version ----

/// What `RegQueryValueExA` gave: its result, the value's type and size when the value is
/// there, its bytes when they fitted.
pub struct Query {
    pub result: u32,
    pub kind: Option<u32>,
    pub size: Option<u32>,
    pub data: Option<Vec<u8>>,
}

/// The registry calls the installer makes (under `HKEY_LOCAL_MACHINE`), so that a test can
/// answer them; in the standalone the portable record (`install::portable`).
pub trait Registry {
    /// `RegOpenKeyExA`: the key, or the error.
    fn open(&mut self, name: &[u8]) -> Result<u32, u32>;
    /// `RegQueryValueExA` with `room` bytes for the data.
    fn query(&mut self, key: u32, name: &[u8], room: u32) -> Query;
    /// `RegCloseKey`.
    fn close(&mut self, key: u32);
    /// `RegCreateKeyA`: the key, or the error.
    fn create(&mut self, name: &[u8]) -> Result<u32, u32>;
    /// `RegSetValueExA`: its result.
    fn set_value(&mut self, key: u32, name: &[u8], kind: u32, data: &[u8]) -> u32;
    /// `RegDeleteKeyA`: its result.
    fn delete(&mut self, name: &[u8]) -> u32;
}

/// A query's answers into the type and the size, and its data to `put`, as the registry
/// writes them.
fn take(q: &Query, put: impl FnOnce(&[u8])) {
    if let Some(kind) = q.kind {
        crate::install::owner::install().names.reg_type.set(kind);
    }
    if let Some(size) = q.size {
        crate::install::owner::install().names.source_size.set(size);
    }
    if let Some(bytes) = &q.data {
        put(bytes);
    }
}

pub fn read_registry(g: &'static crate::install::owner::Install, registry: &mut dyn Registry) {
    let Ok(key) = registry.open(&g.names.key_name.text()) else {
        return;
    };
    g.run.reg_key.set(key);
    let names = &g.names;
    'values: {
        names.source_size.set(1);
        let q = registry.query(key, &names.status_name.text(), 1);
        take(&q, |d| {
            if let Some(&first) = d.first() {
                names.status.set(first);
            }
        });
        if q.result != 0 {
            break 'values;
        }
        if names.reg_type.get() != REG_BINARY || names.source_size.get() != 1 {
            names.status.set(0);
        }
        names.source_size.set(SOURCE_ROOM);
        let q = registry.query(key, &names.source_name.text(), SOURCE_ROOM);
        take(&q, |d| {
            for (k, &b) in d.iter().enumerate().take(SOURCE_ROOM as usize) {
                names.source.set(k, b);
            }
        });
        if q.result != 0 {
            break 'values;
        }
        if names.reg_type.get() != REG_SZ {
            names.source.set(0, 0);
        }
    }
    registry.close(key);
}

/// The registry, or in the standalone the portable record (`install::portable`).
pub fn read_registry_here(g: &'static crate::install::owner::Install) {
    read_registry(g, &mut crate::install::portable::Portable);
}
