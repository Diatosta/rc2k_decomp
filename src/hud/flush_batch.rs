//! `hud::flush_batch` (0x4B74EF): unless the batch holds no indices, drawn by the device
//! (`sub_45A869`: `DrawIndexedPrimitive` through `g_d3dDevice`, the platform's) and
//! emptied.
//!
//! In:  nothing
//! Out: as the device's call leaves the registers when drawn; else kept
//!
//! Callers: `sub_4B1CD0` (the HUD's end), `hud::add_polygon`. The device through `part`
//! (1), in the game the original's call (section 15). Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose device call goes to a stand-in, against the Rust
//! version, the batch empty and not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::hud::batch::{INDEX_COUNT_RVA, VERTEX_COUNT_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::flush_batch", rva(0x4B74EF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const DEVICE_CALL_RVA: usize = rva(0x45A869); // sub_45A869

/// A call by place: 1 the device's draw.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::batch::flush_due`
pub unsafe fn flush_batch(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if !rc2k_core::hud::batch::flush_due(game_read(INDEX_COUNT_RVA)) {
            return Ok(g);
        }
        part(1, &mut g)?;
        game_write(VERTEX_COUNT_RVA, 0u32);
        game_write(INDEX_COUNT_RVA, 0u32);
    }
    Ok(g)
}

/// In the game: the device's draw through the original's call.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    crate::calls::call(DEVICE_CALL_RVA, r);
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { flush_batch(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_764F28, 0
    let entry: &[u8] = &[0x83, 0x3D, 0x28, 0x4F, 0x76, 0x00, 0x00];
    let copy = match copy(0x4B74EF, 0x23, entry, &[0x9], &[(0x45A869, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = [VERTEX_COUNT_RVA, INDEX_COUNT_RVA].map(at).to_vec();
    let saved = capture(&regions);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                game_write(VERTEX_COUNT_RVA, case * 20);
                game_write(INDEX_COUNT_RVA, case * 30);
            },
            &|r| unsafe { flush_batch(r, &mut test_part) },
        );
    }
    restore(&regions, &saved);
    t.finish()
}
