//! `race::save_last_replay` (0x41ECFF): the last race's replay saved as replay eax: opened
//! (`race::open_viewed_replay`, 222), saved (`race::save_viewed_replay`, ebx the slot) and
//! closed (`race::close_viewed_replay`).
//!
//! In:  eax the slot
//! Out: carry set when the opening or the saving failed (the saving's flags kept round
//!      the closing); every register kept (`pusha` / `popa`)
//!
//! Caller: `race::replay_viewer` (mode 3, the slot from `menu::save_replay_loop`; no flags
//! read). Its calls through `part`: 1 `open_viewed_replay` (ebx 222), 2
//! `save_viewed_replay` (ebx the slot), 3 `close_viewed_replay`; in the game their Rust
//! versions. Shadow mode runs the original alone (it writes a file). Self-test: a copy
//! whose calls go to stand-ins, against the Rust version: the opening failing, the saving
//! failing or not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    CF, LAST_RACE, Part, answer, dispatch_parts, regs_entry, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::save_last_replay", rva(0x41ECFF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers and flags it leaves.
pub fn save_last_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut out = *entry;
    let mut r = Regs {
        ebx: LAST_RACE,
        ..*entry
    };
    part(1, &mut r)?;
    if r.carry() {
        out.eflags = r.eflags;
        return Ok(out);
    }
    r.ebx = r.eax;
    part(2, &mut r)?;
    let flags = r.eflags;
    part(3, &mut r)?;
    out.eflags = flags;
    Ok(out)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::{close_viewed_replay as c, open_viewed_replay as o, save_viewed_replay as s};
    *r = match place {
        1 => o::open_viewed_replay(r, &mut o::game_part)?,
        2 => s::save_viewed_replay(r, &mut s::game_part)?,
        _ => c::close_viewed_replay(r, &mut c::game_part)?,
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        save_last_replay(r, &mut game_part)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| save_last_replay(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; mov ebx, 0DEh
    let copy = match crate::race::viewer::copy(
        0x41ECFF,
        0x1D,
        &[0x60, 0xBB, 0xDE, 0x00, 0x00, 0x00],
        &[0x6, 0xF, 0x15],
        &[(0x41E9FF, 1), (0x41ED1C, 2), (0x41EB7C, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for (open_fails, save_fails) in [(true, false), (false, false), (false, true)] {
        run_both(
            &mut t,
            &format!("open fails {open_fails}, save fails {save_fails}"),
            copy.addr(),
            Regs {
                eax: 5,
                ..Regs::SENTINEL
            },
            &[],
            CF,
            &|| {
                answer(Box::new(move |place, r| match place {
                    1 => r.set_carry(open_fails),
                    2 => r.set_carry(save_fails),
                    _ => r.set_carry(!save_fails),
                }))
            },
            &|r| save_last_replay(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    t.finish()
}
