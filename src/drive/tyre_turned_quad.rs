//! `drive::tyre_turned_quad` (0x41C969): the tyre-change screen's turning quad (the
//! steering wheel): the device's turn (esi +0x4EC, x `flt_5BEAC8` x 2 pi, `fsincos` into
//! `flt_5BEBF4` cos, `flt_5BEBF0` sin) turning the half-sizes of the rectangle at ebx
//! (floats: x, y, w, h) round its place plus `dword_5BEAE0` / `5BEAE4`, the colour ecx, the
//! texture rectangle at eax (`quad_tail`), drawn (place 5). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the rectangle, ecx the colour, esi the input device
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, base, p, x87_run};
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::tyre_turned_quad", rva(0x41C969), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tyre_turned_quad(r, &mut game_part) } {
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

/// The Rust version, the draw through `part`.
pub unsafe fn tyre_turned_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (uv, rect, colour, device) = (entry.eax, entry.ebx, entry.ecx, entry.esi);
    p(0x5183CC, 0x40C9_0FDB);
    x87_run!(
        "fild dword ptr [{e} + 0x4EC]",
        "fmul dword ptr [{b} + 0x1BEAC8]",
        "fmul dword ptr [{b} + 0x1183CC]",
        ".byte 0xD9, 0xFB", // fsincos
        "fstp dword ptr [{b} + 0x1BEBF4]",
        "fstp dword ptr [{b} + 0x1BEBF0]",
        "fld dword ptr [{d} + 0xC]",
        "fmul dword ptr [{b} + 0x1BEBF0]",
        "fstp dword ptr [{b} + 0x1BEBF8]",
        "fld dword ptr [{d} + 0xC]",
        "fmul dword ptr [{b} + 0x1BEBF4]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{b} + 0x1BEBFC]",
        "fld dword ptr [{d} + 0x8]",
        "fmul dword ptr [{b} + 0x1BEBF4]",
        "fstp dword ptr [{b} + 0x1BEC00]",
        "fld dword ptr [{d} + 0x8]",
        "fmul dword ptr [{b} + 0x1BEBF0]",
        "fstp dword ptr [{b} + 0x1BEC04]",
        "fld dword ptr [{b} + 0x1BEC00]",
        ".byte 0xD9, 0xE0",
        "fadd dword ptr [{d}]",
        "fiadd dword ptr [{b} + 0x1BEAE0]",
        "fst dword ptr [{v} + 0x40]",
        "fadd dword ptr [{b} + 0x1BEBF8]",
        "fstp dword ptr [{v}]",
        "fld dword ptr [{b} + 0x1BEC00]",
        "fadd dword ptr [{d}]",
        "fiadd dword ptr [{b} + 0x1BEAE0]",
        "fst dword ptr [{v} + 0x60]",
        "fadd dword ptr [{b} + 0x1BEBF8]",
        "fstp dword ptr [{v} + 0x20]",
        "fld dword ptr [{b} + 0x1BEC04]",
        ".byte 0xD9, 0xE0",
        "fadd dword ptr [{d} + 0x4]",
        "fiadd dword ptr [{b} + 0x1BEAE4]",
        "fst dword ptr [{v} + 0x44]",
        "fadd dword ptr [{b} + 0x1BEBFC]",
        "fstp dword ptr [{v} + 0x4]",
        "fld dword ptr [{b} + 0x1BEC04]",
        "fadd dword ptr [{d} + 0x4]",
        "fiadd dword ptr [{b} + 0x1BEAE4]",
        "fst dword ptr [{v} + 0x64]",
        "fadd dword ptr [{b} + 0x1BEBFC]",
        "fstp dword ptr [{v} + 0x24]";
        b = base(), e = device, d = rect, v = a(VERTS)
    );
    quad_words(colour);
    quad_tail(uv);
    draw(
        &Regs {
            esi: uv,
            edi: rect,
            ..*entry
        },
        part,
    )?;
    Ok(*entry)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x41C969,
        0x1E7,
        &[0x60, 0xDB, 0x86, 0xEC, 0x04, 0x00, 0x00],
        &[0x1E0],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = test_regions();
    let saved = capture(&all);
    let uv = [0x10u32, 0x20, 0x80, 0x19];
    let rect = [
        12.5f32.to_bits(),
        3.0f32.to_bits(),
        40.0f32.to_bits(),
        9.0f32.to_bits(),
    ];
    let mut device = vec![0u32; 0x500 / 4];
    let dv = device.as_mut_ptr() as u32;
    for turn in [0i32, 0x40, -0x123, 0x7FFF] {
        let fill = || {
            unsafe { ((dv + 0x4EC) as *mut i32).write(turn) };
            p(0x5BEAE0, 300);
            p(0x5BEAE4, 200);
        };
        both(
            &mut t,
            &format!("turn {turn}"),
            &copy,
            Regs {
                eax: uv.as_ptr() as u32,
                ebx: rect.as_ptr() as u32,
                ecx: 0xFF_FFFF,
                esi: dv,
                ..Regs::SENTINEL
            },
            &fill,
            tyre_turned_quad,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
