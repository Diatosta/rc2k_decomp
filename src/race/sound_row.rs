//! `race::sound_row` (0x40D731): a row of a sound record at edx: +4, +8, +0xC made
//! floats (`race::whole_to_float`), spans at +0x10 and +0x1C (`race::sound_span`).
//!
//! In:  edx = the row
//! Out: eax = as the last `whole_to_float`, ebx = edx + 0x20; the rest preserved
//!
//! See `race/sound_tables.rs`. One caller, `race::prepare_sound_tables`. A call in its
//! first bytes: private copy, its five calls relocated, manual patch. Shadow mode: the
//! Rust version, the row put back, the original's, compared. Self-test: rows of whole
//! numbers and floats, eight ways.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::sound_tables::sound_row;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::sound_row", rva(0x40D731), thunk, &ORIGINAL)
    .private_copy(0x29)
    .relocate_calls(&[0x3, 0xB, 0x13, 0x1B, 0x23])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { sound_row(r) };
    Ok(Left::nothing().with(EAX, g.eax).with(EBX, g.ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[(r.edx, 0x28)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut buffer = vec![0u32; 0x28 / 4];
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
                edx: at,
                ..Regs::SENTINEL
            },
            &|| {
                for n in 0..0x28 / 4 {
                    unsafe {
                        ((at + 4 * n as u32) as *mut u32).write(values[(n + k) % values.len()])
                    };
                }
            },
            &[(at, 0x28)],
            &rust,
        );
    }
    drop(buffer);
    t.finish()
}
