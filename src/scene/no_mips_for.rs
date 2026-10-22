//! `scene::no_mips_for` (0x46512F): ebp (the mip levels wanted) cleared for the texture
//! sizes that get none: 0x80 .. 0xFF and 0x140 on (esi, unsigned).
//!
//! In:  esi the size, ebp the mip levels
//! Out: ebp 0 for those sizes, else as it came; everything else preserved
//! One caller, `gfx::allocate_textures`'s caller 0x42263E (`scene::try_allocate_textures`,
//! whose Rust version inlines it). Shadow mode: both, compared.
//! Self-test: the original against the Rust version on sizes around the bounds.

use crate::errors::Refused;
use crate::framework::{EBP, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::no_mips_for", rva(0x46512F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// True when the size gets no mip levels.
pub fn no_mips_for(size: u32) -> bool {
    // Converted: `rc2k_core::scene::textures::no_mips_for`.
    rc2k_core::scene::textures::no_mips_for(size)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(if no_mips_for(r.esi) {
        Left::nothing().with(EBP, 0)
    } else {
        Left::nothing()
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    for esi in [
        0u32,
        0x40,
        0x7F,
        0x80,
        0xFF,
        0x100,
        0x13F,
        0x140,
        0x200,
        0xFFFF_FFFF,
    ] {
        check_state(
            &mut t,
            &format!("size {esi:#x}"),
            call_original,
            Regs {
                esi,
                ebp: 5,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
