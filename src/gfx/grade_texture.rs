//! `gfx::grade_texture` (0x4055A2): a texture's colours made brighter or duller.
//!
//! In:  eax = the key (components), ebx = the pixels, ecx = width, edx = height
//! Out: every register preserved
//!
//! Only with the reading switch `dword_4F4A24` at 1 and the scale `dword_4F4A28` and
//! the saturation `dword_4F4A2C` not both 0x100 (`scene::reload_system_textures` sets
//! them): the table `byte_4F4A30` made from the scale, then every pixel of an alpha
//! above 0 graded (`gfx/texture_pixels.rs`). Two call sites, as
//! `gfx::alpha_from_kept`'s.
//! Refuses no pixels (the original counts through all 32 bits) and a null pointer.
//! Self-test: 16 pictures by lot under eight settings.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::gfx::harness::{check_pixels, dispatch_pixels, pixels_entry};
use crate::gfx::texture_pixels::{
    self, GRADES_RVA, READING_RVA, SATURATION_RVA, SCALE_RVA, grading, test_pixels,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::grade_texture", rva(0x4055A2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn count(r: &Regs) -> u32 {
    r.ecx.wrapping_mul(r.edx)
}

fn settings() -> [u32; 3] {
    unsafe {
        [
            game_read(READING_RVA),
            game_read(SCALE_RVA),
            game_read(SATURATION_RVA),
        ]
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [reading, scale, saturation] = settings();
    if !grading(reading, scale, saturation) {
        return Ok(Left::nothing());
    }
    if r.ebx == 0 || count(r) == 0 {
        return Err(BadInput("no pixels, or a null pointer").into());
    }
    unsafe {
        let mut table = [0u8; 0x100];
        texture_pixels::grade_texture(
            r.ebx as *mut u8,
            count(r),
            r.eax,
            scale,
            saturation,
            &mut table,
        );
        game_write(GRADES_RVA, table);
    }
    Ok(Left::nothing())
}

fn regions(r: &Regs) -> Vec<Region> {
    let [reading, scale, saturation] = settings();
    if !grading(reading, scale, saturation) || r.ebx == 0 || count(r) == 0 {
        Vec::new()
    } else {
        vec![
            (r.ebx, count(r) as usize * 4),
            (game_addr(GRADES_RVA) as u32, 0x100),
        ]
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
    let inputs: [Region; 4] = [
        at(READING_RVA),
        at(SCALE_RVA),
        at(SATURATION_RVA),
        (game_addr(GRADES_RVA) as u32, 0x100),
    ];
    let saved = capture(&inputs);
    let settings = [
        (1u32, 0x100u32, 0x100u32),
        (0, 0x80, 0x80),
        (1, 0x80, 0x100),
        (1, 0x100, 0x180),
        (1, 0x140, 0x40),
        (1, 0x200, 0),
        (1, 0x10, 0x300),
        (1, 0xFFFF_FFF0, 0xFFFF_FF00),
    ];
    let mut seed = 0x4055_A200u32;
    for case in 0..16usize {
        let (reading, scale, saturation) = settings[case % settings.len()];
        let (width, height) = [(1u32, 1u32), (8, 8), (16, 4), (3, 5)][case % 4];
        let n = (width * height) as usize;
        let first = test_pixels(&mut seed, n);
        let key = test_pixels(&mut seed, 1)[0];
        let buffer = RefCell::new(vec![0xAAAA_AAAAu32; n]);
        let at = buffer.borrow_mut().as_mut_ptr() as u32;
        check_pixels(
            &mut t,
            &format!(
                "case {case}: {width} x {height}, key {key:#x}, settings {reading} {scale:#x} {saturation:#x}"
            ),
            original,
            Regs {
                eax: key,
                ebx: at,
                ecx: width,
                edx: height,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                buffer.borrow_mut().copy_from_slice(&first);
                game_write(READING_RVA, reading);
                game_write(SCALE_RVA, scale);
                game_write(SATURATION_RVA, saturation);
                game_write(GRADES_RVA, [0xAAu8; 0x100]);
            },
            &[(at, n * 4), (game_addr(GRADES_RVA) as u32, 0x100)],
            &rust,
        );
    }
    restore(&inputs, &saved);
    t.finish()
}
