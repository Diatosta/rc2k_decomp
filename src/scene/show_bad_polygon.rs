//! `scene::show_bad_polygon` (0x443AEC): edi `g_polygonVertices` and
//! `scene::log_bad_polygon`: the polygon in the buffer dumped to the log.
//!
//! In:  nothing
//! Out: edi = `g_polygonVertices`; the rest kept
//!
//! Caller: `scene::submit_polygon` (a vertex off the screen; its Rust version calls this
//! one's directly). The dump's log calls through `part` as `log_bad_polygon`'s. Shadow mode
//! runs the original alone (it writes the log). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::parts::{Part, copy, dispatch_parted, run_both, test_part};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::show_bad_polygon", rva(0x443AEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

/// The Rust version: the registers it leaves. Its one call (0, the dump) through `part`.
pub unsafe fn show_bad_polygon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = Regs {
        edi: game_addr(VERTICES_RVA) as u32,
        ..*entry
    };
    part(0, &mut r)?;
    Ok(r)
}

/// In the game: the dump's Rust version with the Rust log functions.
pub fn game_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::scene::log_bad_polygon::{game_part as log_part, log_bad_polygon};
    *r = unsafe { log_bad_polygon(r, &mut log_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { show_bad_polygon(r, part) },
        game_part,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The dump as one stand-in: its own test covers it.
    let copy = match copy(
        0x443AEC,
        0xC,
        &[0x8D, 0x3D, 0xA0, 0x05, 0x96, 0x00],
        &[0x6],
        &[0x443AF8],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(
        &mut t,
        "once",
        &copy,
        &[],
        &|| {},
        Regs::SENTINEL,
        false,
        &|r| unsafe { show_bad_polygon(r, &mut test_part) },
    );
    t.finish()
}
