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
//! Shadow mode asks twice and compares. Self-test: a redirected copy whose three registry
//! calls go to a registry of the test's, as does the Rust version's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore, state_entry};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_LOCAL_MACHINE, RegCloseKey, RegOpenKeyA, RegQueryValueExA,
};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::read_registry", rva(0x413D14), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEY_RVA: usize = rva(0x51BB5C); // phkResult
const KEY_NAME_RVA: usize = rva(0x51BB60); // "SOFTWARE\Magnetic Fields\RC99"
const STATUS_NAME_RVA: usize = rva(0x51BB7E); // "status"
const SOURCE_NAME_RVA: usize = rva(0x51BB85); // "source"
/// byte_51BB8C: 1 = the game is installed, and byte_51BB8D: where from (0xFF bytes).
pub const STATUS_RVA: usize = rva(0x51BB8C);
pub const SOURCE_RVA: usize = rva(0x51BB8D);
const SOURCE_ROOM: u32 = 0xFF;
const SIZE_RVA: usize = rva(0x51BC90); // dword_51BC90
const TYPE_RVA: usize = rva(0x51BC94); // Type
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

pub struct Windows;

impl Registry for Windows {
    unsafe fn open(&mut self, name: *const u8, key: *mut u32) -> u32 {
        unsafe { RegOpenKeyA(HKEY_LOCAL_MACHINE, name, key as *mut HKEY) }
    }
    unsafe fn query(
        &mut self,
        key: u32,
        name: *const u8,
        kind: *mut u32,
        data: *mut u8,
        size: *mut u32,
    ) -> u32 {
        unsafe { RegQueryValueExA(key as HKEY, name, std::ptr::null(), kind, data, size) }
    }
    unsafe fn close(&mut self, key: u32) {
        unsafe { RegCloseKey(key as HKEY) };
    }
}

pub unsafe fn read_registry(registry: &mut dyn Registry) {
    unsafe {
        let key = game_addr(KEY_RVA) as *mut u32;
        if registry.open(game_addr(KEY_NAME_RVA) as *const u8, key) != 0 {
            return;
        }
        let (kind, size) = (
            game_addr(TYPE_RVA) as *mut u32,
            game_addr(SIZE_RVA) as *mut u32,
        );
        let (status, source) = (
            game_addr(STATUS_RVA) as *mut u8,
            game_addr(SOURCE_RVA) as *mut u8,
        );
        let key = key.read_unaligned();
        'values: {
            size.write_unaligned(1);
            let name = game_addr(STATUS_NAME_RVA) as *const u8;
            if registry.query(key, name, kind, status, size) != 0 {
                break 'values;
            }
            if kind.read_unaligned() != REG_BINARY || size.read_unaligned() != 1 {
                status.write(0);
            }
            size.write_unaligned(SOURCE_ROOM);
            let name = game_addr(SOURCE_NAME_RVA) as *const u8;
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

// ---- hook ----

/// What it writes, but for the key's handle, which is another each time.
fn regions() -> [Region; 3] {
    [
        (game_addr(STATUS_RVA) as u32, 1 + SOURCE_ROOM as usize),
        at(SIZE_RVA),
        at(TYPE_RVA),
    ]
}

/// The registry, or in the standalone the portable record (`install::portable`).
pub unsafe fn read_registry_here() {
    if crate::framework::standalone() {
        unsafe { read_registry(&mut crate::install::portable::Portable) };
    } else {
        unsafe { read_registry(&mut Windows) };
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_registry_here() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test: a registry of the test's ----

/// A value: its kind and its data.
type Value = Option<(u32, Vec<u8>)>;

struct Fake {
    has_key: bool,
    status: Value,
    source: Value,
    /// (opened, closed)
    calls: (u32, u32),
}

static FAKE: Mutex<Fake> = Mutex::new(Fake {
    has_key: false,
    status: None,
    source: None,
    calls: (0, 0),
});
const FAKE_KEY: u32 = 0x1234_5678;

extern "system" fn fake_open(_root: u32, _name: *const u8, key: *mut u32) -> u32 {
    let mut fake = FAKE.lock().unwrap();
    if !fake.has_key {
        return 2;
    }
    fake.calls.0 += 1;
    unsafe { key.write_unaligned(FAKE_KEY) };
    0
}

extern "system" fn fake_query(
    key: u32,
    name: *const u8,
    _reserved: *const u32,
    kind: *mut u32,
    data: *mut u8,
    size: *mut u32,
) -> u32 {
    let fake = FAKE.lock().unwrap();
    let name = unsafe { std::ffi::CStr::from_ptr(name as *const _) }.to_bytes();
    let value = match name {
        b"status" => &fake.status,
        b"source" => &fake.source,
        _ => &None,
    };
    let Some((value_kind, bytes)) = value else {
        return 2;
    };
    if key != FAKE_KEY {
        return 6;
    }
    unsafe {
        kind.write_unaligned(*value_kind);
        let room = size.read_unaligned() as usize;
        size.write_unaligned(bytes.len() as u32);
        if bytes.len() > room {
            return 234;
        }
        std::ptr::copy(bytes.as_ptr(), data, bytes.len());
    }
    0
}

extern "system" fn fake_close(_key: u32) -> u32 {
    FAKE.lock().unwrap().calls.1 += 1;
    0
}

struct FakeRegistry;

impl Registry for FakeRegistry {
    unsafe fn open(&mut self, name: *const u8, key: *mut u32) -> u32 {
        fake_open(0x8000_0002, name, key)
    }
    unsafe fn query(
        &mut self,
        key: u32,
        name: *const u8,
        kind: *mut u32,
        data: *mut u8,
        size: *mut u32,
    ) -> u32 {
        fake_query(key, name, std::ptr::null(), kind, data, size)
    }
    unsafe fn close(&mut self, key: u32) {
        fake_close(key);
    }
}

const LEN: usize = 0xB1;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x68, 0x5C, 0xBB, 0x51, 0x00, 0x68, 0x60];
/// `RegOpenKeyA`, `RegQueryValueExA` twice, `RegCloseKey`.
const CALLS: [usize; 4] = [0x10, 0x43, 0x8B, 0xAA];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Which import each call goes to, read from the original.
    let target = |call: usize| unsafe {
        let at = game_addr(rva(0x413D14)) + call;
        let rel = ((at + 1) as *const i32).read_unaligned();
        (at + 5)
            .wrapping_add(rel as usize)
            .wrapping_sub(game_addr(0))
    };
    let (open, query, close) = (target(CALLS[0]), target(CALLS[1]), target(CALLS[3]));
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x413D14)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == open {
                    Some(fake_open as *const () as usize)
                } else if callee == query {
                    Some(fake_query as *const () as usize)
                } else if callee == close {
                    Some(fake_close as *const () as usize)
                } else {
                    None
                }
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut regions = regions().to_vec();
    regions.push(at(KEY_RVA));
    let saved = capture(&regions);
    let text = |text: &[u8]| Some((REG_SZ, text.to_vec()));
    let cases: Vec<(bool, Value, Value)> = vec![
        (false, None, None),
        (true, None, None),
        (true, Some((REG_BINARY, vec![1])), text(b"D:\\\0")),
        (true, Some((REG_BINARY, vec![0])), text(b"\0")),
        (true, Some((REG_BINARY, vec![1])), None),
        (true, Some((4, vec![1])), text(b"E:\\rally\0")),
        (true, Some((REG_BINARY, vec![1, 2])), text(b"E:\\rally\0")),
        (true, Some((REG_BINARY, vec![])), text(b"E:\\rally\0")),
        (
            true,
            Some((REG_BINARY, vec![1])),
            Some((REG_BINARY, b"F:\\\0".to_vec())),
        ),
        (
            true,
            Some((REG_BINARY, vec![1])),
            Some((REG_SZ, vec![b'x'; 0xFF])),
        ),
        (
            true,
            Some((REG_BINARY, vec![1])),
            Some((REG_SZ, vec![b'x'; 0x100])),
        ),
        (true, None, text(b"G:\\\0")),
    ];
    for (n, (has_key, status, source)) in cases.into_iter().enumerate() {
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            *FAKE.lock().unwrap() = Fake {
                has_key,
                status: status.clone(),
                source: source.clone(),
                calls: (0, 0),
            };
            restore(&regions, &vec![0xAA; saved.len()]);
            game_write(STATUS_RVA, 0x55u8);
            let done = if rust {
                read_registry(&mut FakeRegistry);
                true
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (done, FAKE.lock().unwrap().calls, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.1.0 == theirs.1.1, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {n} (key={has_key}): original done={} opened/closed={:?}; rust done={} {:?}; first different byte {first:?} (status, 0xFF source, size, type, key)",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&regions, &saved);
    t.finish()
}
