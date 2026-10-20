//! `strings::append_axis_name` (0x465F9E): appends " (", a sign ('+' when al has 0x10, which
//! is taken off, else '-'), an 'r' for al 3 and up (3 taken off), al + 'x', and ")": an
//! axis's name on the controller page ("(-x)", "(+ry)"). Values past 5 give letters past 'z',
//! as the original.
//!
//! In:  al = the axis, ebx = cursor
//! Out: ebx = new cursor; everything else preserved (eax pushed and popped)
//!
//! Callers: `screens::draw_control_bindings` (0x46A8A4, 0x46A954). The first 5 bytes end inside
//! a `call`: private copy of 0x2E bytes, calls at +1, +6, +0x27 relocated, jump written by
//! hand. Shadow mode: both write, bytes and ebx compared. Self-test: every al 0 .. 255.

use crate::framework::{EBX, RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::strings::utils::bytes_str;
use std::ptr::copy_nonoverlapping;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::append_axis_name", rva(0x465F9E), thunk, &ORIGINAL)
        .private_copy(0x2E)
        .relocate_calls(&[0x01, 0x06, 0x27])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The bytes appended for axis `al` (converted: `rc2k_core::strings::format::axis_name`).
pub use rc2k_core::strings::format::axis_name as append_axis_name;

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (al, cursor) = (r.eax as u8, r.ebx as *mut u8);
    let mine = append_axis_name(al);
    if !shadow() {
        unsafe { copy_nonoverlapping(mine.as_ptr(), cursor, mine.len()) };
        r.ebx = (cursor as usize + mine.len()) as u32;
        return;
    }
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let len = (o.ebx as usize).wrapping_sub(cursor as usize);
    if len != mine.len() || unsafe { from_raw_parts(cursor, len) } != &mine[..] {
        HOOK.mismatch(|| {
            format!(
                "al={al:#x}: original wrote {len} bytes, rust {}",
                bytes_str(&mine)
            )
        });
    }
    r.ebx = o.ebx;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for v in 0..=255u32 {
        let mut buf = [0xAAu8; 16];
        let before = Regs {
            eax: 0x1234_5600 | v,
            ebx: buf.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let len = (ro.ebx as usize).wrapping_sub(buf.as_ptr() as usize);
        let mine = append_axis_name(v as u8);
        let preserved = ro.preserved_except(&before, EBX);
        let mut want = [0xAAu8; 16];
        want[..mine.len()].copy_from_slice(&mine);
        t.check(len == mine.len() && buf == want && preserved, || {
            format!(
                "al={v:#x} original {} rust {} preserved={preserved}",
                bytes_str(buf.get(..len.min(16)).unwrap_or(&[])),
                bytes_str(&mine)
            )
        });
    }
    t.finish()
}
