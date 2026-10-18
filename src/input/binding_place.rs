//! `input::binding_place` (0x408DD6): a binding's word as where the record reads it: for an
//! acquired device (+0x24 1), the axis (bit 0x400: +0x2C negated, 0x800: +0x2C, 0x200:
//! +0x34, 0x100: +0x30, else none) and the button (the low byte n: bit n - 1; 0 none);
//! else none of them.
//!
//! In:  eax = the binding's word, esi = the device's record
//! Out: eax = the axis's place in the record (0 none), edx = its sign (1, -1; 0 when not
//!      acquired), ebx = the button's bit (0 none), ecx = the low byte (less one when not
//!      0) when acquired; no flags read
//!
//! Caller: `sub_408BCD` (twelve times, a binding each). Self-test: the original and the
//! Rust version on words of each kind and buttons 0 .. 0x20 and past, acquired or not.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::devices::{at, test_record};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::binding_place", rva(0x408DD6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub fn binding_place(entry: &Regs) -> Regs {
    let mut r = *entry;
    if at(r.esi, 0x24) != 1 {
        r.edx = 0;
        r.eax = 0;
        r.ebx = 0;
        return r;
    }
    r.edx = 1;
    r.ecx = r.eax;
    r.eax = if r.eax & 0x400 != 0 {
        r.edx = u32::MAX;
        0x2C
    } else if r.eax & 0x800 != 0 {
        0x2C
    } else if r.eax & 0x200 != 0 {
        0x34
    } else if r.eax & 0x100 != 0 {
        0x30
    } else {
        0
    };
    r.ecx &= 0xFF;
    r.ebx = if r.ecx == 0 {
        0
    } else {
        r.ecx -= 1;
        1u32.wrapping_shl(r.ecx)
    };
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let o = binding_place(r);
    Ok(Left::nothing()
        .with(EAX, o.eax)
        .with(EBX, o.ebx)
        .with(ECX, o.ecx)
        .with(EDX, o.edx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for acquired in [1u32, 0, 2] {
        let mut record = test_record(acquired);
        record[0x24..0x28].copy_from_slice(&acquired.to_le_bytes());
        let p = record.as_mut_ptr() as u32;
        for word in [
            0u32,
            0x400,
            0x800,
            0x200,
            0x100,
            0x401,
            0x820,
            0x21,
            0x1F,
            0x0C00,
            0x300,
            0xFFFF_FFFF,
            0x1_0005,
            0xFF,
        ] {
            check_state(
                &mut t,
                &format!("acquired {acquired}, word {word:#x}"),
                call_original,
                Regs {
                    eax: word,
                    esi: p,
                    ..Regs::SENTINEL
                },
                &|| {},
                &[],
                &rust,
            );
        }
        drop(record);
    }
    t.finish()
}
