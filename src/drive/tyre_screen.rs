//! `drive::tyre_screen` (0x41C5FA): the tyre-change screen for the mode eax (0: one table;
//! 2: two, for car esi, the wheel's place 0xAA further left; else two more) and car esi:
//! its layout's 0x20 bytes (`unk_5BEB30` .. `unk_5BEBB0`) into `dword_5BEAD0` (16) and
//! `dword_5BEBD0` (16), the wheel's place (`dword_5BEAE0`, `5BEAE4`: the panel's middle
//! less the wheel's half), the car into `dword_5BEBEC`; the texture (`dword_5BEBE4`,
//! place 0) and eleven render states (place 1); the panel and wheel quads (place 2), the
//! wheel turned by the car's input device (place 3), the fixed parts (place 4). All in
//! `pusha` / `popa`.
//!
//! In:  eax the mode, esi the car (0, 1)
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_change_screen`.
//! Refuses a car past 7 (`g_inputDevices`).
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::tyre_screen", rva(0x41C5FA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { tyre_screen(r, &mut game_part) } {
        Ok(g) => {
            let esp = r.esp;
            *r = Regs {
                eflags: r.eflags,
                ..g
            };
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

const STATES: [(u32, u32); 11] = [
    (2, 0x15),
    (0, 0x1C),
    (5, 0x13),
    (6, 0x14),
    (1, 0x1B),
    (3, 3),
    (2, 0x11),
    (2, 0x12),
    (0, 7),
    (0, 0xE),
    (0, 0xF),
];

/// The Rust version, the calls through `part`.
pub unsafe fn tyre_screen(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (mode, car) = (entry.eax, entry.esi);
    if car > 7 {
        return Err(crate::errors::BadInput("a car past 7 (g_inputDevices)").into());
    }
    let (table, shift) = match mode {
        0 => (0x5BEB30, 0),
        2 => (if car != 0 { 0x5BEB70 } else { 0x5BEB50 }, 0xAA),
        _ => (if car != 0 { 0x5BEBB0 } else { 0x5BEB90 }, 0),
    };
    unsafe {
        std::ptr::copy(a(table) as *const u8, a(0x5BEAD0) as *mut u8, 16);
        std::ptr::copy((a(table) + 16) as *const u8, a(0x5BEBD0) as *mut u8, 16);
    }
    let half = |x: u32, y: u32| ((x.wrapping_sub(y) as i32) >> 1) as u32;
    p(
        0x5BEAE0,
        half(g(0x5BEAD8), g(0x5BEAE8))
            .wrapping_add(g(0x5BEAD0))
            .wrapping_sub(shift),
    );
    p(
        0x5BEAE4,
        half(g(0x5BEADC), g(0x5BEAEC)).wrapping_add(g(0x5BEAD4)),
    );
    p(0x5BEBEC, car);
    let device = g(0x4F6EE8 + 4 * car as usize);
    // ebx the table (the copy's source), ecx and edx the wheel's place (the sums).
    let mut r = Regs {
        ebx: a(table),
        ecx: g(0x5BEAE0),
        edx: g(0x5BEAE4),
        esi: device,
        edi: mode,
        ..*entry
    };
    r.eax = g(0x5BEBE4);
    part(TEXTURE, &mut r)?;
    for (value, state) in STATES {
        r.eax = value;
        r.ebx = state;
        part(STATE, &mut r)?;
    }
    for (uv, rect, place) in [
        (0x5BEB00, 0x5BEAD0, QUAD),
        (0x5BEB10, 0x5BEAE0, QUAD),
        (0x5BEB20, 0x5BEAF0, TURNED),
    ] {
        r.eax = a(uv);
        r.ebx = a(rect);
        r.ecx = 0xFF_FFFF;
        part(place, &mut r)?;
    }
    part(PARTS, &mut r)?;
    Ok(*entry)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls = [
        0xA3, 0xB2, 0xC1, 0xD0, 0xDF, 0xEE, 0xFD, 0x10C, 0x11B, 0x12A, 0x139, 0x148, 0x15E, 0x174,
        0x18A, 0x18F,
    ];
    let copy = match copy_of(0x41C5FA, 0x196, &[0x60, 0x33, 0xD2, 0x85, 0xC0], &calls) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = test_regions();
    let saved = capture(&all);
    for (mode, car) in [(0u32, 0u32), (2, 0), (2, 1), (1, 0), (1, 1)] {
        both(
            &mut t,
            &format!("mode {mode}, car {car}"),
            &copy,
            Regs {
                eax: mode,
                esi: car,
                ..Regs::SENTINEL
            },
            &|| {},
            tyre_screen,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
