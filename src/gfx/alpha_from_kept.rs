//! `gfx::alpha_from_kept` (0x405443): a texture's alpha from a second picture.
//!
//! In:  ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Each pixel's byte 3 = the first byte of the pixel 0x40000 bytes on, inverted
//! (`gfx/texture_pixels.rs`). Two call sites: `sub_41A68B` and `scene::upload_plate`,
//! when the texture's record has +0x5C at 1.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 12 pictures by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels};
use crate::gfx::texture_pixels::{self, test_pixels};
use crate::screens::harness::Left;
use crate::service::harness::Region;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::alpha_from_kept", rva(0x405443), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn count(r: &Regs) -> u32 {
    r.ecx.wrapping_mul(r.edx)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    if r.ebx == 0 || count(r) == 0 || count(r) > 0x10000 {
        return Err(
            BadInput("no pixels, more than the kept picture's room, or a null pointer").into(),
        );
    }
    unsafe { texture_pixels::alpha_from_kept(r.ebx as *mut u8, count(r)) };
    Ok(Left::nothing())
}

fn regions(r: &Regs) -> Vec<Region> {
    if r.ebx == 0 || count(r) == 0 || count(r) > 0x10000 {
        Vec::new()
    } else {
        vec![(r.ebx, count(r) as usize * 4)]
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
    let mut seed = 0x4054_4300u32;
    for (case, &(width, height)) in [
        (1u32, 1u32),
        (2, 2),
        (8, 8),
        (16, 4),
        (3, 5),
        (7, 1),
        (1, 9),
        (32, 32),
        (5, 5),
        (64, 2),
        (2, 64),
        (13, 11),
    ]
    .iter()
    .enumerate()
    {
        let n = (width * height) as usize;
        let first = test_pixels(&mut seed, n);
        let kept = test_pixels(&mut seed, n);
        let buffer = RefCell::new(vec![0xAAAA_AAAAu32; 0x10000 + n]);
        let at = buffer.borrow_mut().as_mut_ptr() as u32;
        check_pixels(
            &mut t,
            &format!("case {case}: {width} x {height}"),
            original,
            Regs {
                ebx: at,
                ecx: width,
                edx: height,
                ..Regs::SENTINEL
            },
            &|| {
                let mut b = buffer.borrow_mut();
                b[..n].copy_from_slice(&first);
                b[0x10000..].copy_from_slice(&kept);
            },
            &[(at, n * 4)],
            &rust,
        );
    }
    t.finish()
}
