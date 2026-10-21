//! `input::set_up_device` (0x408BCD): a device's record made from a player's controls
//! (edi: `dword_75AE28` player 1, `dword_75AF94` player 2): +0x414 .. +0x4DB 0
//! (`system::zero_memory`, 0xC8 bytes); with a controller chosen ([edi] n, not 0): +0 n - 1,
//! +0x418 [edi+0x10], +0xC [edi+0x1C], +0x10 [edi+0x18], +0x14 [edi+0x14], +0x18 [edi+0xC],
//! +0x20 [edi+8] and +8 the same or 2 for 0, and the controller acquired (`input::acquire_device`);
//! then the twelve bindings: each its key (a byte of [edi+0x54 ..]) and the place, sign
//! and button of its word (`input::binding_place` of a dword of [edi+0x24 ..]), in the order
//! of `BINDINGS`.
//!
//! In:  esi = the device's record, edi = the player's controls
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::device_command` (the race's start, both players). The acquiring through
//! `part` (0, esi and edi as here, eax [edi+8] or 2; in the game
//! `input::acquire_device`'s Rust version). Shadow mode runs the original alone (it acquires the device).
//! Self-test: a copy whose acquiring goes to a stand-in (which says the device acquired or
//! not), against the Rust version, on records and controls of the test's.

use crate::errors::Refused;
use crate::field;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::input::devices::{RECORD_LEN, at, put, test_record};
use crate::input::types::InputDevice;
use crate::race::mud::{copy, run_both_from};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::set_up_device", rva(0x408BCD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CONTROLS_LEN: usize = 0x60;

/// The bindings: (the key's byte in the controls, the word's dword, the record's binding).
const BINDINGS: [(u32, u32, u32); 12] = [
    (0x54, 0x2C, 0x41C),
    (0x55, 0x30, 0x42C),
    (0x56, 0x24, 0x43C),
    (0x57, 0x28, 0x44C),
    (0x59, 0x34, 0x45C),
    (0x5A, 0x38, 0x46C),
    (0x58, 0x3C, 0x47C),
    (0x5B, 0x4C, 0x48C),
    (0x5C, 0x48, 0x49C),
    (0x5D, 0x40, 0x4AC),
    (0x5E, 0x44, 0x4BC),
    (0x5F, 0x50, 0x4CC),
];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves (`popa`'s: the entry's).
pub unsafe fn set_up_device(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (record, controls) = (entry.esi, entry.edi);
    unsafe {
        crate::system::zero_memory::zero_memory((record + 0x414) as *mut u8, 0xC8)?;
    }
    let n = at(controls, 0);
    if n != 0 {
        put(record, field!(InputDevice, controller), n - 1);
        for (to, from) in [
            (0x418, 0x10),
            (0xC, 0x1C),
            (0x10, 0x18),
            (0x14, 0x14),
            (0x18, 0xC),
        ] {
            put(record, to, at(controls, from));
        }
        // Converted: `rc2k_core::input::devices::device_kind`.
        let kind = at(controls, 8);
        put(record, field!(InputDevice, sensitivity), kind);
        let kind = rc2k_core::input::devices::device_kind(kind);
        put(record, field!(InputDevice, kind), kind);
        let mut r = Regs {
            eax: kind,
            ..*entry
        };
        part(0, &mut r)?;
    }
    // Converted: `rc2k_core::input::devices::binding` over `DEVICE_BINDINGS`.
    for (key, word, to) in rc2k_core::input::devices::DEVICE_BINDINGS {
        let key = unsafe { ((controls + key as u32) as *const u8).read() };
        let b = rc2k_core::input::devices::binding(
            key,
            at(controls, word as u32),
            at(record, 0x24) == 1,
        );
        for (n, v) in b.into_iter().enumerate() {
            put(record, to as u32 + 4 * n as u32, v);
        }
    }
    Ok(*entry)
}

/// In the game: the acquiring through the game.
/// In the game: `input::acquire_device`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::input::acquire_device::acquire_device(r.esi, &mut crate::input::make_effect::game_part)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { set_up_device(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// Whether the test's acquiring says the device is acquired (+0x24).
static ACQUIRES: AtomicU32 = AtomicU32::new(0);

fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    put(r.esi, 0x24, ACQUIRES.load(Ordering::SeqCst));
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push ebx; push ecx; lea ebx, [esi+414h]
    let copy = match copy(
        0x408BCD,
        0x209,
        &[0x60, 0x53, 0x51, 0x8D, 0x9E, 0x14, 0x04, 0x00, 0x00],
        &[
            0xE, 0x52, 0x64, 0x88, 0xAC, 0xD0, 0xF4, 0x118, 0x13C, 0x160, 0x184, 0x1A8, 0x1CC,
            0x1F0,
        ],
        &[(0x408473, 0)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    for case in 0..12u32 {
        let mut record = test_record(case);
        let start = record.clone();
        let mut controls: Vec<u8> = (0..CONTROLS_LEN as u32)
            .map(|n| (n.wrapping_mul(0x5B) ^ case.wrapping_mul(0x77)) as u8)
            .collect();
        let chosen = [0u32, 1, 3][(case % 3) as usize];
        controls[0..4].copy_from_slice(&chosen.to_le_bytes());
        controls[8..12].copy_from_slice(&[0u32, 1, 3, 2][(case % 4) as usize].to_le_bytes());
        // Each word one of the kinds `binding_place` tells apart, and a button.
        for (k, (_, word, _)) in BINDINGS.into_iter().enumerate() {
            let w = [0x400u32, 0x800, 0x200, 0x100, 0][(k + case as usize) % 5]
                | ((case + k as u32) % 0x22);
            controls[word as usize..word as usize + 4].copy_from_slice(&w.to_le_bytes());
        }
        let acquires = (case / 3) % 2;
        let p = record.as_mut_ptr() as u32;
        let c = controls.as_mut_ptr() as u32;
        run_both_from(
            &mut t,
            &format!("case {case}: controller {chosen}, acquired {acquires}"),
            &copy,
            &[(p, RECORD_LEN)],
            &|| unsafe {
                std::ptr::copy_nonoverlapping(start.as_ptr(), p as *mut u8, RECORD_LEN);
                // Not acquired before the set-up (else `binding_place` sees the old +0x24).
                put(p, field!(InputDevice, acquired), 0);
                ACQUIRES.store(acquires, Ordering::SeqCst);
            },
            Regs {
                esi: p,
                edi: c,
                ..Regs::SENTINEL
            },
            &|r| unsafe { set_up_device(r, &mut test_part) },
        );
        drop((record, controls));
    }
    ACQUIRES.store(0, Ordering::SeqCst);
    t.finish()
}
