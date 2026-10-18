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

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::input::devices::{KEYS_DOWN_RVA, RECORD_LEN, at, key_down, put, test_record};
use crate::input::hat_direction::hat_direction;
use crate::input::hat_reading::hat_reading;
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
const FULL_LEFT: u32 = 0xFFFF_FF00;
const FULL: u32 = 0x100;

/// A call by place: 0 the poll (esi the record).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The bindings' keys and places, or the refusal.
fn check_bindings(record: u32) -> Result<(), Refused> {
    for k in 0..BINDINGS_COUNT {
        let g = BINDINGS + 0x10 * k;
        if at(record, g) > 0xFF {
            return Err(BadInput("a bound key past 0xFF").into());
        }
        if at(record, g + 4) > RECORD_LEN as u32 - 4 {
            return Err(BadInput("a bound axis's place past the record").into());
        }
    }
    Ok(())
}

/// An axis's value: the record's dword at the binding's place times its sign (`imul`).
fn axis_value(record: u32, g: u32) -> i32 {
    (at(record, at(record, g + 4)) as i32).wrapping_mul(at(record, g + 8) as i32)
}

/// One of the four readings (binding at `g`): (the value, all of it pressed).
fn reading(record: u32, g: u32, full: u32, negate_axis: bool) -> (u32, bool) {
    let buttons = at(record, 0x38);
    if key_down(at(record, g)) {
        return (full, true);
    }
    let button = at(record, g + 0xC);
    if at(record, g + 4) != 0 {
        let v = axis_value(record, g).max(0);
        if button == 0 {
            let v = v as u32;
            return (if negate_axis { v.wrapping_neg() } else { v }, false);
        }
        if v > 0x40 && buttons & button != 0 {
            return (full, true);
        }
        return (0, false);
    }
    if button != 0 && buttons & button != 0 {
        return (full, true);
    }
    (0, false)
}

/// One of the eight buttons (binding at `g`).
fn pressed(record: u32, g: u32) -> bool {
    if key_down(at(record, g)) {
        return true;
    }
    let button = at(record, g + 0xC);
    if at(record, g + 4) != 0 {
        if axis_value(record, g) <= 0x40 {
            return false;
        }
        return button == 0 || at(record, 0x38) & button != 0;
    }
    button != 0 && at(record, 0x38) & button != 0
}

/// `test ecx, ecx` then `mov ecx, 0FFFFFF00h; js` .. `mov ecx, 100h`: not 0 made all of it.
fn whole_steering(v: u32) -> u32 {
    match v {
        0 => 0,
        v if (v as i32) < 0 => FULL_LEFT,
        _ => FULL,
    }
}

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn read_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let record = entry.esi;
    check_bindings(record)?;
    let mut poll = *entry;
    part(0, &mut poll)?;
    if at(record, 0x414) != 1 {
        return Ok(*entry);
    }
    let mut bits = (hat_reading(record).0 << 24) | hat_direction(record);
    let mut steering = {
        let (left, l) = reading(record, 0x41C, FULL_LEFT, true);
        let (right, r) = reading(record, 0x42C, FULL, false);
        if l || r {
            bits |= 0x1000;
        }
        left.wrapping_add(right)
    };
    if bits & 0x1000 != 0 {
        steering = whole_steering(steering);
    }
    let (mut throttle, t) = reading(record, 0x43C, FULL, false);
    if t {
        bits |= 0x4000;
    }
    let (mut brake, b) = reading(record, 0x44C, FULL, false);
    if b {
        bits |= 0x2000;
    }
    if at(record, 8) == 1 {
        bits |= 0x7000;
        steering = whole_steering(steering);
        if brake != 0 {
            brake = FULL;
        }
        if throttle != 0 {
            throttle = FULL;
        }
    }
    for k in 0..8 {
        if pressed(record, 0x45C + 0x10 * k) {
            bits |= 1 << k;
        }
    }
    put(record, 0x4DC, steering);
    put(record, 0x4E0, brake);
    put(record, 0x4E4, throttle);
    put(record, 0x4E8, bits);
    Ok(*entry)
}

/// In the game: the poll through the game (DirectInput).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { call_game(r, game_addr(POLL_RVA)) };
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
