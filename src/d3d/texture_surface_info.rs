//! `d3d::texture_surface_info` (0x41AA93): a locked system texture made the surface the
//! blitters draw on (`g_surface`): width and height from the record (esi), bytes and
//! bits a pixel from its pixel kind's description (+0xC), the row's bytes and dwords,
//! the pixels and pitch from the lock's description (`dword_5ABA18`: +0x24, +0x10), the
//! row's gap, the format (the kind's description + 8); then the clip (`gfx::set_clip`
//! on `g_surface`).
//!
//! In:  esi the record
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `d3d::lock_texture`. Self-test: the original and the Rust version on a record
//! of the test's, the surface and the clip compared.

use crate::d3d::texture_lock::{DESCRIPTION_RVA, SURFACE_RVA, get, put};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("d3d::texture_surface_info", rva(0x41AA93), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub fn texture_surface_info(record: u32) -> Result<(), crate::errors::BadInput> {
    let s = game_addr(SURFACE_RVA) as u32;
    let d = game_addr(DESCRIPTION_RVA) as u32;
    let width = get(record + 4);
    let kind = get(record + 0xC);
    let bytes = get(kind + 4);
    let row = bytes.wrapping_mul(width);
    let pitch = get(d + 0x10);
    put(s + 8, width);
    put(s + 0xC, get(record + 8));
    put(s + 0x14, bytes);
    put(s + 0x10, bytes << 3);
    put(s + 0x18, row);
    put(s + 0x1C, row >> 2);
    put(s + 0x28, get(d + 0x24));
    put(s + 0x24, pitch);
    put(s + 0x2C, pitch.wrapping_sub(row));
    put(s + 0x20, kind + 8);
    unsafe { crate::gfx::set_clip::set_clip(s as *const [u32; 4]) }
}

fn regions() -> Vec<Region> {
    use crate::gfx::surface::{CLIP_BOTTOM_RVA, CLIP_LEFT_RVA, CLIP_RIGHT_RVA, CLIP_TOP_RVA};
    vec![
        (game_addr(SURFACE_RVA) as u32, 0x48),
        at(CLIP_LEFT_RVA),
        at(CLIP_TOP_RVA),
        at(CLIP_RIGHT_RVA),
        at(CLIP_BOTTOM_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, crate::errors::Refused> {
    texture_surface_info(r.esi)?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

static mut RECORD: [u32; 0x1E] = [0; 0x1E];
static mut KIND: [u32; 8] = [0, 2, 0x55AA, 0, 0, 0, 0, 0];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.push((game_addr(DESCRIPTION_RVA) as u32, 0x6C));
    let saved = capture(&vars);
    let record = (&raw const RECORD) as u32;
    for (width, bytes, pitch) in [
        (0x100u32, 2u32, 0x200u32),
        (0x80, 4, 0x240),
        (0x40, 1, 0x40),
        (0x10, 2, 0x18),
    ] {
        check_state(
            &mut t,
            &format!("width {width:#x}, bytes {bytes}, pitch {pitch:#x}"),
            call_original,
            Regs {
                esi: record,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&vars, &saved);
                (&raw mut KIND).cast::<u32>().add(1).write(bytes);
                put(record + 4, width);
                put(record + 8, width / 2);
                put(record + 0xC, (&raw const KIND) as u32);
                let d = game_addr(DESCRIPTION_RVA) as u32;
                put(d + 0x10, pitch);
                put(d + 0x24, 0x1234_5000);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
