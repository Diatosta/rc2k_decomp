//! `drive::tyre_quad` (0x41DD3B): a screen quad of the tyre-change screen: the screen
//! rectangle at ebx (x, y, w, h, whole numbers) as the four vertices' places
//! (`g_polygonVertices`), z 0, rhw 1, the colour ecx, the texture rectangle at eax
//! (`drive::tyre_parts`' `quad_tail`, scaled to the screen), drawn (place 5, the device's
//! DrawPrimitive: a call into the game). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the screen rectangle, ecx the colour
//! Out: every register kept; no flags read
//!
//! Callers: `drive::tyre_screen` (two), `drive::tyre_screen_parts` (ten).
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::p;
use crate::race::car_starts::Part;
use crate::service::harness::{capture, restore};
use rc2k_core::drive::tyre_screen::quad_corners;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::tyre_quad", rva(0x41DD3B), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tyre_quad(r, &mut game_part) } {
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
// Converted: `rc2k_core::drive::tyre_screen::quad_corners` (the shared tail stays x87).
pub unsafe fn tyre_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let (uv, rect, colour) = (entry.eax, entry.ebx, entry.ecx);
    let at = |o: u32| unsafe { ((rect + o) as *const u32).read_unaligned() };
    let corners = quad_corners([at(0), at(4), at(8), at(0xC)]);
    for (k, [x, y]) in corners.into_iter().enumerate() {
        p(VERTS + 0x20 * k, x.to_bits());
        p(VERTS + 0x20 * k + 4, y.to_bits());
    }
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
    let copy = match copy_of(0x41DD3B, 0x134, &[0x60, 0x8B, 0xF0, 0x8B, 0xFB], &[0x12D]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let all = test_regions();
    let saved = capture(&all);
    let uv = [0x10u32, 0x20, 0x80, 0x19];
    let rect = [100u32, 50, 0x100, 0x19];
    for colour in [0xFF_FFFFu32, 0x1234] {
        let fill = || {};
        both(
            &mut t,
            &format!("colour {colour:#x}"),
            &copy,
            Regs {
                eax: uv.as_ptr() as u32,
                ebx: rect.as_ptr() as u32,
                ecx: colour,
                ..Regs::SENTINEL
            },
            &fill,
            tyre_quad,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
