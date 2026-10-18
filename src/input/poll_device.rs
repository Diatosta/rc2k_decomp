//! `input::poll_device` (0x408869): an acquired device's state copied into its record.
//! With its DirectInput device (+4) acquired (+0x24 1): `Poll`; `GetDeviceState` (0x50
//! bytes into `g_deviceState`); when that fails `Acquire` and (sic) the record's first 0x50
//! bytes copied over the state (`rep movsd` where zeros were meant, which also leaves esi
//! 0x50 further on, so that the rest reads and writes the record from there: section 16).
//! Then the
//! axes: for kinds up to 2, x the state dword `+0x1D & 7` (negated with bit 0x10) and y
//! `+0x1C & 7` (likewise), a y below 0 being the third axis (made positive) and the second
//! 0; for kinds above 2, x the state's first dword and the others by `+0x1A` / `+0x1B` (each
//! plus 0x100 with bit 0x10); the second and third made positive; into +0x2C, +0x30, +0x34;
//! the 32 buttons (state +0x30 .., bit 7) as bits into +0x38.
//!
//! In:  esi = the device's record (`input/devices.rs`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::read_device`. Its DirectInput calls through `part`: 0 `Poll`, 1
//! `GetDeviceState` (eax its result), 2 `Acquire`; in the game the device's own methods
//! (platform, section 15). Shadow mode runs the original alone (it polls the device).
//! Self-test: the original and the Rust version on records of the test's with a device of
//! the test's (its three methods noting, its state by lot, failing or not): each kind, the
//! axis choices by lot, acquired or not, with and without a device.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, STATE_LEN, STATE_RVA, at, com_call, put, test_record};
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::poll_device", rva(0x408869), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place (see the header): eax the device, and for 1 the result back in eax.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

fn state(n: u32) -> u32 {
    unsafe { game_read(STATE_RVA + 4 * n as usize) }
}

/// The state dword a selector byte names (`& 7`), and whether its bit 0x10 is set.
fn selected(record: u32, at_byte: u32) -> (u32, bool) {
    let b = unsafe { ((record + at_byte) as *const u8).read() };
    (state(u32::from(b & 7)), b & 0x10 != 0)
}

/// `cmp v, 0; jns; neg v`.
fn positive(v: u32) -> u32 {
    if (v as i32) < 0 { v.wrapping_neg() } else { v }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn poll_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let record = entry.esi;
    let device = at(record, 4);
    if at(record, 0x24) != 1 || device == 0 {
        return Ok(*entry);
    }
    let mut r = Regs {
        eax: device,
        ..*entry
    };
    part(0, &mut r)?;
    r.eax = device;
    part(1, &mut r)?;
    // The `rep movsd` of a failed read moves esi on by 0x50 as well: what follows reads
    // and writes the record from there (sic) until the `popa`.
    let mut record = record;
    if r.eax != 0 {
        r.eax = device;
        part(2, &mut r)?;
        unsafe {
            std::ptr::copy(
                record as *const u8,
                game_addr(STATE_RVA) as *mut u8,
                STATE_LEN,
            );
        }
        record += STATE_LEN as u32;
    }
    let (x, mut y, mut z);
    if at(record, 8) <= 2 {
        let (v, n) = selected(record, 0x1D);
        x = if n { v.wrapping_neg() } else { v };
        let (v, n) = selected(record, 0x1C);
        y = if n { v.wrapping_neg() } else { v };
        z = 0;
        if (y as i32) < 0 {
            z = 0u32.wrapping_sub(y);
            y = 0;
        }
    } else {
        x = state(0);
        let (v, n) = selected(record, 0x1A);
        y = if n { v.wrapping_add(0x100) } else { v };
        let (v, n) = selected(record, 0x1B);
        z = if n { v.wrapping_add(0x100) } else { v };
    }
    let (y, z) = (positive(y), positive(z));
    put(record, 0x2C, x);
    put(record, 0x30, y);
    put(record, 0x34, z);
    let mut buttons = 0u32;
    for k in 0..32usize {
        if unsafe { game_read::<u8>(STATE_RVA + 0x30 + k) } & 0x80 != 0 {
            buttons |= 1 << k;
        }
    }
    put(record, 0x38, buttons);
    Ok(*entry)
}

/// In the game: the device's own methods (DirectInput).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => r.eax = com_call(r.eax, 0x64, &[]),
            1 => {
                r.eax = com_call(
                    r.eax,
                    0x24,
                    &[STATE_LEN as u32, game_addr(STATE_RVA) as u32],
                )
            }
            _ => r.eax = com_call(r.eax, 0x1C, &[]),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { poll_device(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test: a DirectInput device of the test's ----

/// What the test's device's `GetDeviceState` gives back, and the state it writes.
static FAILS: AtomicU32 = AtomicU32::new(0);
static TEST_STATE: Mutex<[u8; STATE_LEN]> = Mutex::new([0; STATE_LEN]);
/// The test device's methods called, by slot.
pub static NOTED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

extern "system" fn test_poll(_: u32) -> u32 {
    NOTED.lock().unwrap().push(0x64);
    0
}

extern "system" fn test_get_state(_: u32, len: u32, to: u32) -> u32 {
    NOTED.lock().unwrap().push(0x24);
    let s = TEST_STATE.lock().unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), to as *mut u8, (len as usize).min(STATE_LEN))
    };
    FAILS.load(Ordering::SeqCst)
}

extern "system" fn test_acquire(_: u32) -> u32 {
    NOTED.lock().unwrap().push(0x1C);
    0
}

extern "system" fn test_release(_: u32) -> u32 {
    NOTED.lock().unwrap().push(0x08);
    0
}

extern "system" fn test_unacquire(_: u32) -> u32 {
    NOTED.lock().unwrap().push(0x20);
    0
}

/// A device object of the test's: its table's slots 0x08 (`Release`), 0x1C (`Acquire`),
/// 0x20 (`Unacquire`), 0x24 (`GetDeviceState`) and 0x64 (`Poll`) the ones above.
pub struct TestDevice {
    table: Vec<u32>,
    object: Box<u32>,
}

impl TestDevice {
    pub fn new() -> Self {
        let mut table = vec![0u32; 0x70 / 4];
        table[0x08 / 4] = test_release as *const () as u32;
        table[0x1C / 4] = test_acquire as *const () as u32;
        table[0x20 / 4] = test_unacquire as *const () as u32;
        table[0x24 / 4] = test_get_state as *const () as u32;
        table[0x64 / 4] = test_poll as *const () as u32;
        let object = Box::new(table.as_ptr() as u32);
        Self { table, object }
    }

    pub fn address(&self) -> u32 {
        let _ = &self.table;
        &*self.object as *const u32 as u32
    }
}

impl Default for TestDevice {
    fn default() -> Self {
        Self::new()
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept: [Region; 1] = [(game_addr(STATE_RVA) as u32, STATE_LEN)];
    let saved = capture(&kept);
    let device = TestDevice::new();
    let mut x = 0x2468_ACE1u32;
    for case in 0..24u32 {
        let mut next = || {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            x
        };
        let kind = [1u32, 2, 3, 0][(case % 4) as usize];
        let acquired = if case % 7 == 6 { 0 } else { 1 };
        let has_device = case % 9 != 8;
        let fails = if case % 5 == 4 { 0x8007_000C } else { 0 };
        let mut record = test_record(case);
        let mut put_r = |o: usize, v: u32| record[o..o + 4].copy_from_slice(&v.to_le_bytes());
        put_r(4, if has_device { device.address() } else { 0 });
        put_r(8, kind);
        put_r(0x24, acquired);
        let selectors = next();
        record[0x1A..0x1E].copy_from_slice(&selectors.to_le_bytes());
        let start = record.clone();
        let mut state = [0u8; STATE_LEN];
        for (k, b) in state.iter_mut().enumerate() {
            *b = (next() >> (k % 24)) as u8;
        }
        *TEST_STATE.lock().unwrap() = state;
        FAILS.store(fails, Ordering::SeqCst);
        let p = record.as_mut_ptr() as u32;
        let run = |original: bool| {
            unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN);
                std::ptr::write_bytes(game_addr(STATE_RVA) as *mut u8, 0xAA, STATE_LEN);
            }
            NOTED.lock().unwrap().clear();
            let before = Regs {
                esi: p,
                ..Regs::SENTINEL
            };
            let regs = if original {
                let mut r = before;
                unsafe { call_original(&mut r) };
                Some(r.preserved_except(&before, 0))
            } else {
                unsafe { poll_device(&before, &mut game_part) }
                    .ok()
                    .map(|r| r.preserved_except(&before, 0))
            };
            (
                regs,
                std::mem::take(&mut *NOTED.lock().unwrap()),
                capture(&[(p, RECORD_LEN), kept[0]]),
            )
        };
        let (theirs, mine) = (run(true), run(false));
        t.check(theirs == mine && theirs.0 == Some(true), || {
            format!(
                "case {case}: kind {kind}, acquired {acquired}, device {has_device}, fails {fails:#x}, selectors {selectors:#x}: original {:?} {:x?}; rust {:?} {:x?}; state same {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
        drop(record);
    }
    drop(device);
    FAILS.store(0, Ordering::SeqCst);
    restore(&kept, &saved);
    t.finish()
}
