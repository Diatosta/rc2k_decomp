//! `scene::fill_light_ramp` (0x44F169): a table of colours from one colour to another.
//!
//! In:  esi = two colours, three dwords each, edi = a table of 0x100 dwords
//! Out: eax = the last part worked out, ebx = 0x100, ecx = the last entry, edx = 0,
//!      ebp = 0xFF; esi and edi preserved
//!
//! Entry n (0 .. 0xFF) = for each of the three parts the first colour's plus
//! (the second's less the first's) times n over 255 (signed, towards zero), its low
//! byte; the first part in bits 16 .. 23, the third in the low byte, 0xFF on top.
//! Two call sites in `scene::fill_light_tables`.
//! Refuses a null pointer.
//! Self-test: 30 colours by lot into a table of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::scene::light_tables::{self, TABLE_LEN, test_colours};
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::fill_light_ramp", rva(0x44F169), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.esi == 0 || r.edi == 0 {
        return Err(BadInput("a null colour or table").into());
    }
    Ok(unsafe { light_tables::fill_light_ramp(r.esi as *const _, r.edi as *mut _) })
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: &[Region] = if r.esi == 0 || r.edi == 0 {
        &[]
    } else {
        &[(r.edi, TABLE_LEN)]
    };
    dispatch_state(&HOOK, r, call_original, regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let table: Cell<[u32; 0x100]> = Cell::new([0; 0x100]);
    let mut seed = 0x11640006u32;
    for case in 0..30 {
        let colours = test_colours(&mut seed);
        check_state(
            &mut t,
            &format!("case {case}: {colours:x?}"),
            call_original,
            Regs {
                esi: colours.as_ptr() as u32,
                edi: table.as_ptr() as u32,
                ..Regs::SENTINEL
            },
            &|| table.set([0xAAAA_AAAA; 0x100]),
            &[(table.as_ptr() as u32, TABLE_LEN)],
            &rust,
        );
    }
    t.finish()
}
