//! `gfx::halve_texture` (0x40545F): a texture's picture made smaller for its mip
//! levels.
//!
//! In:  eax = how many times, ebx = the pixels, ecx = width, edx = height
//! Out: ecx, edx = the size after the last time (as they were for none); everything
//!      else preserved
//!
//! In place, so many times: half as wide and high, each pixel the average of four,
//! the colour counting only those of an alpha above 0 (`gfx/texture_pixels.rs`). An
//! odd width leaves the rows a pixel short of where the original's pointer goes, as it
//! does. Two call sites, as `gfx::alpha_from_kept`'s.
//! Refuses a size that halves to 0 (the original counts through all 32 bits) and a
//! null pointer.
//! Self-test: 14 pictures by lot, 0 to 5 times.
use crate::errors::{BadInput, Refused};
use crate::framework::{ECX, EDX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::texture_pixels::{self, halvable, test_pixels};
use crate::screens::harness::Left;
use crate::service::harness::Region;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::halve_texture", rva(0x40545F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn usable(r: &Regs) -> bool {
    r.eax == 0 || (r.ebx != 0 && halvable(r.ecx, r.edx, r.eax))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    if !usable(r) {
        return Err(BadInput("a size that halves to nothing, or a null pointer").into());
    }
    if r.eax == 0 {
        return Ok(Left::nothing());
    }
    let (width, height) =
        unsafe { texture_pixels::halve_texture(r.ebx as *mut u32, r.ecx, r.edx, r.eax) };
    Ok(Left::nothing().with(ECX, width).with(EDX, height))
}

fn regions(r: &Regs) -> Vec<Region> {
    if r.eax == 0 || !usable(r) {
        Vec::new()
    } else {
        vec![(r.ebx, r.ecx as usize * r.edx as usize * 4)]
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_pixels(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut seed = 0x4054_5F00u32;
    for (case, &(width, height, times)) in [
        (8u32, 8u32, 0u32),
        (2, 2, 1),
        (8, 8, 1),
        (8, 8, 3),
        (16, 4, 2),
        (32, 32, 5),
        (5, 5, 1),
        (7, 3, 1),
        (9, 6, 2),
        (64, 2, 1),
        (2, 64, 1),
        (13, 11, 3),
        (256, 4, 2),
        (4, 256, 2),
    ]
    .iter()
    .enumerate()
    {
        let n = (width * height) as usize;
        let first = test_pixels(&mut seed, n);
        let buffer = RefCell::new(vec![0xAAAA_AAAAu32; n]);
        let at = buffer.borrow_mut().as_mut_ptr() as u32;
        check_pixels(
            &mut t,
            &format!("case {case}: {width} x {height}, {times} times"),
            original,
            Regs {
                eax: times,
                ebx: at,
                ecx: width,
                edx: height,
                ..Regs::SENTINEL
            },
            &|| buffer.borrow_mut().copy_from_slice(&first),
            &[(at, n * 4)],
            &rust,
        );
    }
    t.finish()
}
