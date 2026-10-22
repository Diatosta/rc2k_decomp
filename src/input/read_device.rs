//! `input::read_device` (0x408E38): a device's four readings from its bindings (see
//! `input/devices.rs`). The device polled (`sub_408869`: its state copied into +0x2C ..
//! +0x38); while it is read (+0x414 1):
//! - the bits: the hat's angle (`input::hat_reading`) in the top byte, its direction
//!   (`input::hat_direction`) in the second;
//! - steering left (binding +0x41C) and right (+0x42C), throttle (+0x43C), brake (+0x44C):
//!   each its key down (`g_keysDown`), or its axis (the record's dword at the binding's
//!   place times its sign, below 0 made 0) as it is, or, with a button bound too, all of it
//!   when that is above 0x40 and the button down, or the button alone: all of it is
//!   0xFFFFFF00 for left (whose axis is negated), 0x100 for the others, and sets bit 0x1000
//!   (steering), 0x4000 (throttle), 0x2000 (brake); the steering is left plus right, and
//!   with bit 0x1000 set held to 0xFFFFFF00 or 0x100 by its sign; on a digital device (+8
//!   1) bits 0x7000 and every reading not 0 made all of it;
//! - eight buttons (+0x45C .. +0x4CC) as bits 1 .. 0x80: the key down, or the axis's
//!   value above 0x40 (with a button bound, that button down as well), or the button;
//!
//! Into +0x4DC (steering), +0x4E0 (brake), +0x4E4 (throttle), +0x4E8 (the bits).
//!
//! In:  esi = the device's record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_408A56` (both devices, each tick of the input thread). The poll through
//! `part` (0; in the game through the game: DirectInput, section 15). Refuses (before
//! anything) a bound key past 0xFF and an axis's place past the record. Shadow mode runs
//! the original alone (it polls the device). Self-test: a copy whose poll goes to a
//! stand-in, against the Rust version, on records of the test's: keys, axes and buttons by
//! lot, each kind, read or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::input::devices::{KEYS_DOWN_RVA, RECORD_LEN, put, test_record};
use crate::input::devices::{joy_state, key_table, record_bytes};
use crate::race::mud::{copy, run_both_from, test_part};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::read_device", rva(0x408E38), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const POLL_RVA: usize = rva(0x408869); // sub_408869: the device polled (DirectInput)
const BINDINGS: u32 = 0x41C;
const BINDINGS_COUNT: u32 = 12;

/// A call by place: 0 the poll (esi the record).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
/// Converted: `rc2k_core::input::devices::read_device`; the bindings are checked before the poll,
/// the record read again after it.
pub unsafe fn read_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use rc2k_core::input::devices;
    let record = entry.esi;
    devices::check_bindings(&record_bytes(record))?;
    let mut poll = *entry;
    part(0, &mut poll)?;
    let Some(r) = devices::read_device(&record_bytes(record), &key_table(), &joy_state())? else {
        return Ok(*entry);
    };
    let readings = devices::READINGS as u32;
    put(record, readings, r.steering);
    put(record, readings + 4, r.brake);
    put(record, readings + 8, r.throttle);
    put(record, devices::BITS as u32, r.bits);
    Ok(*entry)
}

/// In the game: the poll through the game (DirectInput).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(POLL_RVA, r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { read_device(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// A record of the test's whose twelve bindings are made from `seed`: keys 0 .. 0xFF,
/// axes at +0x2C, +0x30, +0x34 or none, signs 1 or -1, buttons by bit or none; the axes
/// and buttons in the state copies by lot.
fn bound_record(seed: u32, kind: u32, read: u32) -> Vec<u8> {
    let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    };
    let mut r = test_record(seed);
    let mut put = |o: usize, v: u32| r[o..o + 4].copy_from_slice(&v.to_le_bytes());
    put(8, kind);
    put(0x414, read);
    put(0x2C, next() % 0x200);
    put(0x30, (next() % 0x200).wrapping_sub(0x100));
    put(0x34, next() % 0x100);
    put(0x38, next());
    for k in 0..BINDINGS_COUNT as usize {
        let g = BINDINGS as usize + 0x10 * k;
        put(g, next() & 0xFF);
        put(g + 4, [0u32, 0x2C, 0x30, 0x34][(next() % 4) as usize]);
        put(g + 8, if next() & 1 == 0 { 1 } else { u32::MAX });
        put(
            g + 0xC,
            if next() % 3 == 0 {
                0
            } else {
                1 << (next() % 32)
            },
        );
    }
    r
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call sub_408869
    let rel = (0x408869i64 - (0x408E39 + 5)) as i32;
    let b = rel.to_le_bytes();
    let copy = match copy(
        0x408E38,
        0x4D5,
        &[0x60, 0xE8, b[0], b[1], b[2], b[3]],
        &[1, 0x15, 0x1F],
        &[(0x408869, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let shared: Vec<Region> = crate::input::devices::shared_regions().to_vec();
    let saved = capture(&shared);
    for seed in 0..24u32 {
        let kind = [1u32, 2, 3][(seed % 3) as usize];
        let read = if seed % 8 == 7 { 0 } else { 1 };
        let mut record = bound_record(seed, kind, read);
        let start = record.clone();
        let p = record.as_mut_ptr() as u32;
        let regions: Vec<Region> = vec![(p, RECORD_LEN)];
        run_both_from(
            &mut t,
            &format!("seed {seed}, kind {kind}, read {read}"),
            &copy,
            &regions,
            &|| unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN);
                // Some keys down: those of some bindings, and others.
                let keys = game_addr(KEYS_DOWN_RVA) as *mut u8;
                for k in 0..0x100usize {
                    *keys.add(k) = if (k as u32).wrapping_mul(seed | 1).is_multiple_of(5) {
                        0x80
                    } else {
                        0
                    };
                }
            },
            Regs {
                esi: p,
                ..Regs::SENTINEL
            },
            &|r| unsafe { read_device(r, &mut test_part) },
        );
        drop(record);
    }
    restore(&shared, &saved);
    t.finish()
}
