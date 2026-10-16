//! `memory::move_bytes` (0x46573F): eax bytes from esi to edi, `rep movsd` then `rep movsb`.
//!
//! In:  eax the count, esi from, edi to
//! Out: ecx 0, esi and edi past the bytes
//!
//! Callers: `sub_4BA388` (`marks::order_spline_points`, twice) and `sub_4BD1A1`, each inside
//! a `pusha`. Forwards, a dword at a time and then the last bytes: an edi a little past esi
//! repeats what was there, as the original. Self-test: the original and the Rust version on
//! buffers of the test's (0 .. 0x23 bytes, apart and overlapping).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("memory::move_bytes", rva(0x46573F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub unsafe fn move_bytes(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        for _ in 0..g.eax >> 2 {
            (g.edi as *mut u32).write_unaligned((g.esi as *const u32).read_unaligned());
            g.esi = g.esi.wrapping_add(4);
            g.edi = g.edi.wrapping_add(4);
        }
        for _ in 0..g.eax & 3 {
            *(g.edi as *mut u8) = *(g.esi as *const u8);
            g.esi = g.esi.wrapping_add(1);
            g.edi = g.edi.wrapping_add(1);
        }
    }
    g.ecx = 0;
    g
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { move_bytes(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(r.edi, r.eax as usize)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = [0u8; 0x80];
    let len = buffer.len();
    let b = buffer.as_mut_ptr() as u32;
    for count in [0u32, 1, 3, 4, 7, 8, 0x23] {
        for (from, to) in [(0u32, 0x40u32), (0x40, 1), (0, 2), (0, 5), (3, 0)] {
            check_state(
                &mut t,
                &format!("{count:#x} bytes, {from:#x} to {to:#x}"),
                call_original,
                Regs {
                    eax: count,
                    esi: b + from,
                    edi: b + to,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    for n in 0..len {
                        *(b as *mut u8).add(n) = (n as u8).wrapping_mul(37).wrapping_add(5);
                    }
                },
                &[(b, len)],
                &rust,
            );
        }
    }
    t.finish()
}
