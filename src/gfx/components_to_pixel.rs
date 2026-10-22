//! `gfx::components_to_pixel` (0x405C8A): puts four 8-bit components, one per byte of eax,
//! together into a pixel. The reverse of `pixel_to_components`.
//!
//! In:  eax = components, ebx = pixel format
//! Out: eax = the pixel; everything else preserved
//! Components of 0 give 0 without looking at the format. With a format that has rotation
//! counts, the pixel is the four components rotated left by their counts and masked. With
//! the other kind the pixel is built one bit at a time from the format's top bit down: each
//! bit takes the next bit, from the top, of the first component whose mask has it; the
//! components are shifted out of four scratch bytes (`dword_4F3FA8`), which are left as they
//! end up.
//! 30 call sites, among them the per-pixel loops that scale and convert pictures. Manual patch with a private copy (0x70 bytes, nothing to relocate): a
//! conditional jump sits in the first bytes.
//! Refuses a null format.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow,
};
use crate::gfx::pixel_format::{
    PixelFormat, SCRATCH_RVA, TEST_FORMATS, TEST_VALUES, surface_format,
};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::components_to_pixel", rva(0x405C8A), thunk, &ORIGINAL)
        .private_copy(0x70)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The pixel, and what is left in the scratch bytes if they are used (converted:
/// `rc2k_core::gfx::pixels::pack`).
pub use rc2k_core::gfx::pixels::pack as packed;

pub unsafe fn components_to_pixel(format: &PixelFormat, components: u32) -> u32 {
    let (pixel, scratch) = packed(format, components);
    if let Some(scratch) = scratch {
        unsafe { game_write(SCRATCH_RVA, scratch) };
    }
    pixel
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 {
        return Err(BadInput("no pixel format").into());
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    Ok(Left::nothing().with(EAX, unsafe { components_to_pixel(&format, r.eax) }))
}

// Written out instead of going through `dispatch_state`: the game converts pictures one pixel
// at a time through here (19 million calls while it loads), and that harness allocates.
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if r.ebx == 0 {
        unsafe { call_original(r) };
        return;
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    if !shadow() {
        r.eax = unsafe { components_to_pixel(&format, r.eax) };
        return;
    }
    let (pixel, scratch) = packed(&format, r.eax);
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let scratch_o: [u8; 4] = unsafe { game_read(SCRATCH_RVA) };
    if o.eax != pixel || scratch.is_some_and(|s| s != scratch_o) || !o.preserved_except(r, EAX) {
        HOOK.mismatch(|| {
            format!(
                "{format:x?} components={:#x}: original {:#x} scratch {scratch_o:x?}; rust {pixel:#x} {scratch:x?}",
                r.eax, o.eax
            )
        });
    }
    r.eax = o.eax;
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if r.ebx == 0 {
        unsafe { crate::framework::call_game(r, crate::framework::game_addr(rva(0x405C8A))) };
        return Ok(());
    }
    let format = unsafe { (r.ebx as *const PixelFormat).read_unaligned() };
    r.eax = unsafe { components_to_pixel(&format, r.eax) };
    Ok(())
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(SCRATCH_RVA)];
    let saved = capture(&regions);
    for format in TEST_FORMATS.iter().chain([&surface_format()]) {
        for components in TEST_VALUES {
            let before = Regs {
                eax: components,
                ebx: format as *const PixelFormat as u32,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("{format:x?} components={components:#x}"),
                call_original,
                before,
                &|| unsafe { game_write(SCRATCH_RVA, 0x5EED_0001u32) },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
