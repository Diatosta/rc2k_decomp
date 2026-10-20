//! `gfx::set_component_format` (0x405DB4): describes the format of the game's own
//! component pixels.
//!
//! In:  nothing
//! Out: every register preserved
//! The format at `unk_4F3FAC` gets the masks 0xFF, 0xFF00, 0xFF0000 and 0xFF000000 and
//! is described by `gfx::describe_format`. One caller, `start`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_ptr, hook_asm, rva};
use crate::gfx::describe_format::{FORMAT_LEN, describe_format};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::set_component_format", rva(0x405DB4), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const FORMAT_RVA: usize = rva(0x4F3FAC); // unk_4F3FAC
/// Converted: `rc2k_core::pixels::COMPONENT_MASKS`.
const MASKS: [u32; 4] = rc2k_core::pixels::COMPONENT_MASKS;

// ---- Rust version ----

pub unsafe fn set_component_format() -> Result<(), Refused> {
    unsafe {
        let format = game_ptr::<u32>(FORMAT_RVA);
        for (n, mask) in MASKS.into_iter().enumerate() {
            format.add(1 + n).write_unaligned(mask);
        }
        describe_format(format)?;
    }
    Ok(())
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_ptr::<u8>(FORMAT_RVA) as u32, FORMAT_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_component_format() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "the format",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &vec![0xAA; saved.len()]),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
