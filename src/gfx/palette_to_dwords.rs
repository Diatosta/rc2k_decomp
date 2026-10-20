//! `gfx::palette_to_dwords` (0x405DDE): turns a palette of 256 entries of three bytes into
//! 256 dwords.
//!
//! In:  esi = the three-byte palette, edi = the dwords
//! Out: every register preserved
//! A dword is the entry's first byte shifted up 16, its second shifted up 8, and its third;
//! the top byte is 0. Entry 255 is done first. Four callers, the picture readers.
//! Refuses null pointers. The name is a reading.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::palette_to_dwords", rva(0x405DDE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ENTRIES: usize = 256;

// ---- Rust version ----

pub unsafe fn palette_to_dwords(from: *const u8, to: *mut u32) -> Result<(), BadInput> {
    if from.is_null() || to.is_null() {
        return Err(BadInput("no palette"));
    }
    // From the last entry down, as the original goes: it matters if the two overlap.
    for entry in (0..ENTRIES).rev() {
        unsafe {
            // Converted: `rc2k_core::pixels::palette_entry`.
            let bytes = (from.add(entry * 3) as *const [u8; 3]).read();
            to.add(entry)
                .write_unaligned(rc2k_core::pixels::palette_entry(bytes));
        }
    }
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { palette_to_dwords(r.esi as *const u8, r.edi as *mut u32) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = if r.edi == 0 {
        Vec::new()
    } else {
        vec![(r.edi, ENTRIES * 4)]
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for seed in [1u32, 0x9E37_79B9, 0] {
        let from: Vec<u8> = (0..ENTRIES as u32 * 3)
            .map(|n| (n.wrapping_mul(seed) >> 5) as u8 ^ n as u8)
            .collect();
        let to = std::cell::RefCell::new(vec![0xAAAA_AAAAu32; ENTRIES + 2]);
        let before = Regs {
            esi: from.as_ptr() as u32,
            edi: to.borrow_mut().as_mut_ptr() as u32 + 4,
            ..Regs::SENTINEL
        };
        let whole = [(to.borrow().as_ptr() as u32, (ENTRIES + 2) * 4)];
        check_state(
            &mut t,
            &format!("seed={seed:#x}"),
            call_original,
            before,
            &|| to.borrow_mut().fill(0xAAAA_AAAA),
            &whole,
            &rust,
        );
    }
    t.finish()
}
