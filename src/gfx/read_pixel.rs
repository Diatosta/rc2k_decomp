//! `gfx::read_pixel` (0x406584): a pixel of `g_surface` taken apart.
//!
//! In:  ecx = x, edx = y
//! Out: eax = the components; the rest preserved
//!
//! See `race/mud.rs` (a 16-bit pixel keeps the row address's upper half in eax). Two
//! callers (`race::mud_pixel` and one more). Self-test: surfaces of the test's (16 and 32
//! bits), places inside them.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::mud::{TestSurface, has_format, read_pixel};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::read_pixel", rva(0x406584), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { read_pixel(r.ecx, r.edx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        for (x, y) in [(0u32, 0u32), (5, 7), (63, 47), (31, 20)] {
            let before = Regs {
                ecx: x,
                edx: y,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_original(&mut ro) };
            let mine = unsafe { read_pixel(x, y) };
            let preserved = ro.preserved_except(&before, EAX);
            let (row, raw) = unsafe { crate::race::mud::raw_pixel(x, y) };
            let format = unsafe {
                (crate::framework::game_read::<u32>(rva(0x4F49B8))
                    as *const crate::gfx::pixel_format::PixelFormat)
                    .read_unaligned()
            };
            let direct = crate::gfx::pixel_to_components::pixel_to_components(&format, raw);
            t.check(preserved && ro.eax == mine, || {
                format!(
                    "{bits} bits, ({x}, {y}): original eax {:#x} preserved {preserved}; rust {mine:#x}; row {row:#x}, what the conversion got {raw:#x}; the format {format:x?}, its conversion of that {direct:#x}",
                    ro.eax
                )
            });
        }
        drop(surface);
    }
    t.finish()
}
