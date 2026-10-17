//! `race::sound_span` (0x40D71B): the two dwords at ebx made floats
//! (`race::whole_to_float`), the third their difference (`fld` / `fsub` / `fstp`).
//!
//! In:  ebx = the span
//! Out: eax = as the second `whole_to_float`, ebx = the second dword; the rest preserved
//!
//! See `race/sound_tables.rs`. Callers: `race::prepare_sound_tables`, `race::sound_row`.
//! It begins with its call: private copy, both calls relocated, manual patch. Shadow mode:
//! the Rust version, the span put back, the original's, compared. Self-test: spans of
//! whole numbers and floats, eight ways.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::sound_tables::sound_span;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::sound_span", rva(0x40D71B), thunk, &ORIGINAL)
    .private_copy(0x16)
    .relocate_calls(&[0x0, 0x8])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { sound_span(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(EBX, g.ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.ebx, 12)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = vec![0u32; 12 / 4];
    let at = buffer.as_mut_ptr() as u32;
    let values = [
        0u32,
        5,
        0xFFFF_FFFB,
        0x3F80_0000,
        0x4120_0000,
        0x0100_0000,
        100,
        0xFF00_0000,
    ];
    for k in 0..values.len() {
        check_state(
            &mut t,
            &format!("values from {k}"),
            call_original,
            Regs {
                ebx: at,
                ..Regs::SENTINEL
            },
            &|| {
                for n in 0..12 / 4 {
                    unsafe {
                        ((at + 4 * n as u32) as *mut u32).write(values[(n + k) % values.len()])
                    };
                }
            },
            &[(at, 12)],
            &rust,
        );
    }
    drop(buffer);
    t.finish()
}
