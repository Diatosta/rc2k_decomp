//! `race::put_mud_texel` (0x44A5A9): a texel of the locked mud texture (`g_surface`) set
//! by its place on a 256 x 256 square, the place halved as `race::mud_texel`'s; skipped
//! when an odd place is halved, or outside the surface; else `gfx::put_surface_pixel`.
//!
//! In:  eax = the components, ecx = x, edx = y (0 .. 0xFF)
//! Out: eax = the pixel put, or as it came; the rest kept; no flags read
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`. Refuses what
//! `put_surface_pixel` refuses (no format, no surface), before anything. Self-test: the
//! original and the Rust version on test surfaces (16 and 32 bits), each halving, places
//! even, odd and outside; the surface compared.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::gfx::pixel_format::SCRATCH_RVA;
use crate::gfx::put_surface_pixel::put_surface_pixel;
use crate::race::mud::{TestSurface, has_format};
use crate::race::mud_texel::{HALVINGS_RVA, TEST_PLACES, texel_place};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::put_mud_texel", rva(0x44A5A9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const HEIGHT_RVA: usize = rva(0x4F49A4); // g_surface.height

/// The Rust version (see the header): eax as it leaves it.
pub unsafe fn put_mud_texel(entry: &Regs) -> Result<u32, Refused> {
    let Some((x, y)) = texel_place(entry.ecx, entry.edx) else {
        return Ok(entry.eax);
    };
    unsafe {
        if x >= game_read::<u32>(WIDTH_RVA) || y >= game_read::<u32>(HEIGHT_RVA) {
            return Ok(entry.eax);
        }
        let put = Regs {
            ecx: x,
            edx: y,
            ..*entry
        };
        Ok(put_surface_pixel(&put)?.eax)
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { put_mud_texel(r) }?))
}

/// The pixel the call may write (its place on the surface) and the format's scratch.
fn regions(r: &Regs) -> Vec<Region> {
    let mut v = vec![at(SCRATCH_RVA)];
    if let Some((x, y)) = texel_place(r.ecx, r.edx) {
        unsafe {
            if x < game_read::<u32>(WIDTH_RVA) && y < game_read::<u32>(HEIGHT_RVA) {
                let (at, wide) = crate::gfx::put_surface_pixel::place(x, y);
                v.push((at, wide as usize));
            }
        }
    }
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    let kept = [at(HALVINGS_RVA), at(SCRATCH_RVA)];
    let saved = capture(&kept);
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        let pixels: Region = (surface.pixels.as_ptr() as u32, surface.pixels.len());
        let start = capture(&[pixels]);
        for halvings in [0u32, 1, 2, 3] {
            for (k, (x, y)) in TEST_PLACES.into_iter().enumerate() {
                let colour = [0xFF12_3456u32, 0, 0x80FF_00FF, 0x0001_0203][k % 4];
                check_state(
                    &mut t,
                    &format!("{bits} bits, halvings {halvings}, ({x}, {y}), {colour:#x}"),
                    call_original,
                    Regs {
                        eax: colour,
                        ecx: x,
                        edx: y,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe {
                        restore(&[pixels], &start);
                        game_write(HALVINGS_RVA, halvings);
                    },
                    &[pixels, at(SCRATCH_RVA)],
                    &rust,
                );
            }
        }
        drop(surface);
    }
    restore(&kept, &saved);
    t.finish()
}
