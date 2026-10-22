//! `drive::tyre_screen_parts` (0x41C790): the tyre-change screen's fixed parts: ten quads
//! (`drive::tyre_quad`, place 2) from texture rectangles set in `dword_5C2A1C` and screen
//! rectangles in `dword_5C2A0C`, placed from the layout (`dword_5BEBD0` .., `dword_5BEAE0`
//! .., the car's half by `dword_5BEBEC`). All in `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use rc2k_core::drive::tyre_screen::fixed_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::tyre_screen_parts", rva(0x41C790), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tyre_screen_parts(r, &mut game_part) } {
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

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tyre_screen_parts(r, &mut game_part) }?;
        let esp = r.esp;
        *r = Regs {
            eflags: r.eflags,
            ..g
        };
        r.esp = esp;
    }
    Ok(())
}

const UV: usize = 0x5C2A1C;
const RECT: usize = 0x5C2A0C;

fn set(at: usize, v: [u32; 4]) {
    for (k, x) in v.iter().enumerate() {
        p(at + 4 * k, *x);
    }
}

/// The Rust version, the quads through `part`.
// Converted: `rc2k_core::drive::tyre_screen::fixed_parts`.
pub unsafe fn tyre_screen_parts(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut q = Regs {
        eax: a(UV),
        ebx: a(RECT),
        ecx: 0xFF_FFFF,
        ..*entry
    };
    let four = |at: usize| std::array::from_fn(|k| g(at + 4 * k));
    for (uv, rect) in fixed_parts(four(0x5BEBD0), four(0x5BEAE0), g(0x5BEBEC) != 0) {
        set(UV, uv);
        set(RECT, rect);
        part(QUAD, &mut q)?;
    }
    Ok(*entry)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x41C790,
        0x1D9,
        &[0x60, 0x8D, 0x05, 0x1C, 0x2A, 0x5C, 0x00],
        &[
            0x4C, 0xA2, 0xBC, 0x10D, 0x119, 0x125, 0x164, 0x170, 0x17C, 0x1D2,
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = test_regions();
    let saved = capture(&all);
    for car in [0u32, 1] {
        let fill = || {
            for k in 0..4usize {
                p(0x5BEBD0 + 4 * k, 100 + 7 * k as u32);
                p(0x5BEAE0 + 4 * k, 200 + 11 * k as u32);
            }
            p(0x5BEBEC, car);
        };
        both(
            &mut t,
            &format!("car {car}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            tyre_screen_parts,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
