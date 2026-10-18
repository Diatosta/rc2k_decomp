//! `race::flush_draw_batch` (0x45FD1D): the scene's draw batch handed to the device: with
//! indices in it (`dword_6D3CEC`, edx), `sub_41ADF5` (the platform's `DrawIndexedPrimitive`:
//! eax the vertex count `dword_6D3CF0`, ebx the vertices `unk_6D3CF4`, ecx the indices
//! `unk_6D7CF4`); a failure (eax not 0) logged as "drawtri" and its code (level 1, each
//! inside `pusha`); then both counts 0.
//!
//! In:  nothing
//! Out: edx = 0 with an empty batch; else the registers as the device leaves them (ebx the
//!      vertices); no flags read
//!
//! Callers: `race::draw_sky` (0x4555E5), `sub_4557AB` (twice), `sub_4559A6`, 0x45FCB4.
//! Its calls through `part`: 3 the device, 4 the log's text (eax), 5 its code (eax); in the
//! game all three through the game (section 15, as `race/frame_flush.rs`). Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose calls go to stand-ins (the
//! device failing or not), against the Rust version, batches empty and not.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::mud::{copy, run_both};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::flush_draw_batch", rva(0x45FD1D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const INDEX_COUNT_RVA: usize = rva(0x6D3CEC); // dword_6D3CEC
const VERTEX_COUNT_RVA: usize = rva(0x6D3CF0); // dword_6D3CF0
const VERTICES_RVA: usize = rva(0x6D3CF4); // unk_6D3CF4
const INDICES_RVA: usize = rva(0x6D7CF4); // unk_6D7CF4
const DRAWTRI_RVA: usize = rva(0x4DDCA4); // "drawtri"

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn flush_draw_batch(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        r.edx = game_read(INDEX_COUNT_RVA);
        if r.edx != 0 {
            r.ecx = game_addr(INDICES_RVA) as u32;
            r.eax = game_read(VERTEX_COUNT_RVA);
            r.ebx = game_addr(VERTICES_RVA) as u32;
            part(3, &mut r)?;
            if r.eax != 0 {
                let mut log = Regs {
                    eax: game_addr(DRAWTRI_RVA) as u32,
                    ..r
                };
                part(4, &mut log)?;
                let mut log = r;
                part(5, &mut log)?;
            }
        }
        game_write(INDEX_COUNT_RVA, 0u32);
        game_write(VERTEX_COUNT_RVA, 0u32);
    }
    Ok(r)
}

/// In the game: the device and the log through the game (`frame_flush::batch_part`).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::race::frame_flush::batch_part(place, r)
}

/// The Rust version with the game's calls: for Rust callers.
pub unsafe fn flush_draw_batch_in_game(r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { flush_draw_batch(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { flush_draw_batch(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// What the test's device gives back (its HRESULT).
static DEVICE_GIVES: AtomicU32 = AtomicU32::new(0);

/// The stand-ins: noted as `level_init::note` notes them; the device gives `DEVICE_GIVES`
/// and leaves ecx and edx as a `stdcall` may.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    if place == 3 {
        r.eax = DEVICE_GIVES.load(Ordering::SeqCst);
        r.ecx = 0x0C0C_0C0C;
        r.edx = 0x0D0D_0D0D;
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov edx, dword_6D3CEC
    let copy = match copy(
        0x45FD1D,
        0x4F,
        &[0x8B, 0x15, 0xEC, 0x3C, 0x6D, 0x00],
        &[0x1C, 0x2D, 0x34],
        &[(0x41ADF5, 3), (0x401427, 4), (0x401622, 5)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let regions = [at(INDEX_COUNT_RVA), at(VERTEX_COUNT_RVA)];
    let saved = capture(&regions);
    for (indices, vertices, gives) in [
        (0u32, 0u32, 0u32),
        (0, 9, 0x8876_0001),
        (3, 4, 0),
        (0x1800, 0x400, 0x8876_017C),
        (6, 0, 0xFFFF_FFFF),
    ] {
        run_both(
            &mut t,
            &format!("indices {indices:#x}, vertices {vertices:#x}, device gives {gives:#x}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(INDEX_COUNT_RVA, indices);
                game_write(VERTEX_COUNT_RVA, vertices);
                DEVICE_GIVES.store(gives, Ordering::SeqCst);
            },
            &|r| unsafe { flush_draw_batch(r, &mut test_part) },
        );
    }
    DEVICE_GIVES.store(0, Ordering::SeqCst);
    restore(&regions, &saved);
    t.finish()
}
