//! `gfx::put_surface_pixel` (0x4065BB): four 8-bit components made a pixel of the surface's
//! format and put at x, y on `g_surface`.
//!
//! In:  eax the components, ecx x, edx y
//! Out: eax the pixel; everything else kept
//!
//! The format through `g_surface`'s pointer (0x4F49B8), `gfx::components_to_pixel`; the
//! pixel 16 bits wide when the surface's bits per pixel are 16, else 32. Five callers
//! (among them the marks' snow textures). Refuses no format (with components not 0) and
//! no surface. Self-test: the original and the Rust version on a surface of the test's
//! (16 and 32 bits, the test formats), put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::gfx::components_to_pixel::components_to_pixel;
use crate::gfx::pixel_format::{PixelFormat, SCRATCH_RVA, TEST_FORMATS};
use crate::gfx::surface::{BASE_RVA, BITS_PER_PIXEL_RVA, PITCH_RVA};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::put_surface_pixel", rva(0x4065BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `g_surface`'s format pointer.
pub const FORMAT_POINTER_RVA: usize = rva(0x4F49B8);

/// Where the pixel goes and how wide it is.
unsafe fn place(x: u32, y: u32) -> (u32, u32) {
    unsafe {
        let base: u32 = game_read(BASE_RVA);
        let row = base.wrapping_add(y.wrapping_mul(game_read(PITCH_RVA)));
        let wide = if game_read::<u32>(BITS_PER_PIXEL_RVA) == 16 {
            2
        } else {
            4
        };
        (row.wrapping_add(x.wrapping_mul(wide)), wide)
    }
}

/// The Rust version (see the header).
pub unsafe fn put_surface_pixel(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let format: u32 = game_read(FORMAT_POINTER_RVA);
        if g.eax != 0 && format == 0 {
            return Err(BadInput("no surface format").into());
        }
        if game_read::<u32>(BASE_RVA) == 0 {
            return Err(BadInput("no surface").into());
        }
        g.eax = if g.eax == 0 {
            0
        } else {
            components_to_pixel(&(format as *const PixelFormat).read_unaligned(), g.eax)
        };
        let (at, wide) = place(g.ecx, g.edx);
        if wide == 2 {
            (at as *mut u16).write_unaligned(g.eax as u16);
        } else {
            (at as *mut u32).write_unaligned(g.eax);
        }
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { put_surface_pixel(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let (at, wide) = unsafe { place(r.ecx, r.edx) };
    let regions = [
        (at, wide as usize),
        crate::service::harness::at(SCRATCH_RVA),
    ];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs: Vec<Region> = vec![
        at(FORMAT_POINTER_RVA),
        at(BASE_RVA),
        at(PITCH_RVA),
        at(BITS_PER_PIXEL_RVA),
        at(SCRATCH_RVA),
    ];
    let saved = capture(&inputs);
    let pixels = vec![0xAAu8; 0x400];
    let p = pixels.as_ptr() as u32;
    let mut format = TEST_FORMATS[0];
    let fp = &mut format as *mut PixelFormat as u32;
    let regions = [(p, pixels.len()), at(SCRATCH_RVA)];
    for (n, f) in TEST_FORMATS.iter().enumerate() {
        for bits in [16u32, 32] {
            for (x, y, components) in [(0u32, 0u32, 0u32), (3, 1, 0x80FF_4020), (7, 3, 0x0102_0304)]
            {
                check_state(
                    &mut t,
                    &format!("format {n}, {bits} bits, ({x}, {y}), {components:#x}"),
                    call_original,
                    Regs {
                        eax: components,
                        ecx: x,
                        edx: y,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe {
                        std::ptr::write_bytes(p as *mut u8, 0xAA, pixels.len());
                        (fp as *mut PixelFormat).write_unaligned(*f);
                        game_write(FORMAT_POINTER_RVA, fp);
                        game_write(BASE_RVA, p);
                        game_write(PITCH_RVA, 0x40u32);
                        game_write(BITS_PER_PIXEL_RVA, bits);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    unsafe { game_write(BASE_RVA, 0u32) };
    t.check(
        unsafe { put_surface_pixel(&Regs::SENTINEL) }.is_err(),
        || "no surface: the Rust version went on".into(),
    );
    restore(&inputs, &saved);
    t.finish()
}
