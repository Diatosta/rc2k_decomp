//! `gfx::key_texture_colours` (0x40540C): a texture's black and magenta.
//!
//! In:  eax = the key (components), ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Each pixel's alpha made 0xFF; one whose colour's top five bits are all 0 becomes the
//! key with alpha 0xFF, one that is 0xF800F8 by those bits becomes 0
//! (`gfx/texture_pixels.rs`). Two call sites, as `gfx::alpha_from_kept`'s, when the
//! texture's record has +0x5C at 0.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 12 pictures by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels, pixels_entry};
use crate::gfx::texture_pixels::{self, test_pixels};
use crate::screens::harness::Left;
use crate::service::harness::Region;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::key_texture_colours", rva(0x40540C), thunk, &ORIGINAL).self_test(self_test);

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
    if r.ebx == 0 || count(r) == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe { texture_pixels::key_texture_colours(r.ebx as *mut u32, count(r), r.eax) };
    Ok(Left::nothing())
}

fn regions(r: &Regs) -> Vec<Region> {
    if r.ebx == 0 || count(r) == 0 {
        Vec::new()
    } else {
        vec![(r.ebx, count(r) as usize * 4)]
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_pixels(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    pixels_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut seed = 0x4054_0C00u32;
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
        let key = test_pixels(&mut seed, 1)[0];
        let buffer = RefCell::new(vec![0xAAAA_AAAAu32; n]);
        let at = buffer.borrow_mut().as_mut_ptr() as u32;
        check_pixels(
            &mut t,
            &format!("case {case}: {width} x {height}, key {key:#x}"),
            original,
            Regs {
                eax: key,
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
