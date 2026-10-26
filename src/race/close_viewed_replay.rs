//! `race::close_viewed_replay` (0x41EB7C): the viewer's replay closed, if open
//! (`dword_5BEC88` 1): the game's physics patches back (`race::restore_replay_patches`),
//! the file `h` closed (`files::close`), `dword_5BEC88` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::open_viewed_replay` (failing), `race::save_last_replay`, 0x444034. Its calls
//! through `part`: 1 `restore_replay_patches`, 2 `files::close` (eax the handle); in the
//! game their Rust versions. Shadow mode runs the original alone (it closes a file).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, open or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    HANDLE, OPEN, Part, Saved, dispatch_parts, dword, get, put, regs_entry, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::close_viewed_replay", rva(0x41EB7C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves (the entry's).
pub fn close_viewed_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::closes`.
    if rc2k_core::race::viewer_flow::closes(get(OPEN)) {
        let mut r = *entry;
        part(1, &mut r)?;
        r.eax = get(HANDLE);
        part(2, &mut r)?;
        put(OPEN, 0);
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 1 {
        *r = crate::race::replay_patches::restore_replay_patches(
            r,
            &mut crate::race::replay_patches::game_part,
        )?;
    } else {
        let closed = unsafe { crate::files::handle::close(r.eax) };
        r.set_carry(!closed);
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        close_viewed_replay(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| close_viewed_replay(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp dword_5BEC88, 1
    let copy = match crate::race::viewer::copy(
        0x41EB7C,
        0x25,
        &[0x60, 0x83, 0x3D, 0x88, 0xEC, 0x5B, 0x00, 0x01],
        &[0xA, 0x14],
        &[(0x41F31D, 1), (0x402E23, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let regions = [dword(OPEN), dword(HANDLE)];
    let saved = Saved::new(&regions);
    for open in [0u32, 1, 2] {
        run_both(
            &mut t,
            &format!("open {open}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                put(OPEN, open);
                put(HANDLE, 0x4321);
            },
            &|r| close_viewed_replay(r, &mut test_part),
        );
    }
    drop(saved);
    t.finish()
}
