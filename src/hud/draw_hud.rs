//! `hud::draw_hud` (0x4B1CD0): the race's HUD: `hud::set_batch_depth`; the batch empty and
//! without a texture (`dword_75C038` -1); the record `unk_6D13FE`'s times in tenths
//! (`hud::tenths`); by its +0xF4 `hud::layout_1` (1), `hud::layout_2` (2) or
//! `hud::layout_0`; then `hud::flush_batch`.
//!
//! In:  nothing
//! Out: ebp the record; the rest as the flush leaves them
//!
//! Caller: `sub_45A893` (0x45A8F0). Hooked through a private copy (a call at the entry).
//! The depth and tenths are their Rust versions; the layouts and the flush through `part`
//! (1 / 2 / 3 the layouts 0 / 1 / 2, 4 the flush), in the game their Rust versions. Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose layouts and flush go
//! to stand-ins, against the Rust version, every layout.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::hud::batch::{BATCH_TEXTURE_RVA, INDEX_COUNT_RVA, POLYGON_RVA, VERTEX_COUNT_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("hud::draw_hud", rva(0x4B1CD0), thunk, &ORIGINAL)
    .private_copy(0x5B)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CALLS: [usize; 6] = [0x0, 0x29, 0x40, 0x47, 0x4E, 0x55];
pub const RECORD_RVA: usize = rva(0x6D13FE); // unk_6D13FE

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::{Layout, NO_TEXTURE}`
pub unsafe fn draw_hud(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use rc2k_core::hud::batch;
    let mut g = *entry;
    unsafe {
        crate::hud::set_batch_depth::set_batch_depth();
        game_write(BATCH_TEXTURE_RVA, batch::NO_TEXTURE);
        game_write(VERTEX_COUNT_RVA, 0u32);
        game_write(INDEX_COUNT_RVA, 0u32);
        g.ebp = game_addr(RECORD_RVA) as u32;
        g = crate::hud::tenths::tenths(&g);
        let layout = match batch::Layout::of(((g.ebp + 0xF4) as *const u32).read_unaligned()) {
            batch::Layout::Single => 1,
            batch::Layout::One => 2,
            batch::Layout::Two => 3,
        };
        part(layout, &mut g)?;
        part(4, &mut g)?;
    }
    Ok(g)
}

/// In the game: the layouts' and the flush's Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => layout_0::layout_0(r, &mut layout_0::game_part)?,
            2 => layout_1::layout_1(r, &mut layout_1::game_part)?,
            3 => layout_2::layout_2(r, &mut layout_2::game_part)?,
            _ => flush_batch::flush_batch(r, &mut flush_batch::game_part)?,
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_hud(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call hud::set_batch_depth (its first bytes as the original's, not the hook's)
    let first: [u8; 5] = {
        let rel = (0x4B726Ci64 - (0x4B1CD0 + 5)) as i32;
        let b = rel.to_le_bytes();
        [0xE8, b[0], b[1], b[2], b[3]]
    };
    let copy = match copy(
        0x4B1CD0,
        0x5B,
        &first,
        &CALLS,
        &[(0x4B1D8C, 1), (0x4B2743, 2), (0x4B3B64, 3), (0x4B74EF, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(POLYGON_RVA) as u32, 0x80),
        at(BATCH_TEXTURE_RVA),
        at(VERTEX_COUNT_RVA),
        at(INDEX_COUNT_RVA),
        (game_addr(rva(0x75C074)) as u32, 0x18),
    ];
    let mut vars = regions.clone();
    vars.push((game_addr(RECORD_RVA) as u32, 0x100));
    let saved = capture(&vars);
    for layout in 0..4u32 {
        run_both(
            &mut t,
            &format!("layout {layout}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(RECORD_RVA + 0xF4, layout);
                game_write(VERTEX_COUNT_RVA, 9u32);
                std::ptr::write_bytes(game_addr(POLYGON_RVA) as *mut u8, 0xAA, 0x80);
            },
            &|r| unsafe { draw_hud(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
