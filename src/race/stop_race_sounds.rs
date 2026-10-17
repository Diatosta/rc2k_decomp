//! `race::stop_race_sounds` (0x45D28E): `race::reset_race_sounds` (a `call` and a `retn`;
//! "stop sounds").
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `sub_444AC7`, `sub_444F7A` twice, `sub_45C53B`'s
//! neighbour (no flags read). It begins with its call: private copy, the call relocated,
//! manual patch. Shadow mode runs the original alone (it sends commands). Self-test: a
//! copy whose call goes to a stand-in that notes it, against the Rust version's call.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::race_end::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::stop_race_sounds", rva(0x45D28E), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `race::stop_race_sounds`: its one call.
pub unsafe fn stop_race_sounds(entry: &Regs, part: Part) -> Result<Regs, crate::errors::Refused> {
    let mut g = *entry;
    part(0x10, &mut g)?;
    Ok(g)
}

fn game(_: u32, r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { reset_race_sounds(r, &SoundVars::game(), &mut sound_part) }.map(|_| ())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { stop_race_sounds(r, &mut game) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x45D28E)),
            0x6,
            &[0xE8, 0xE5, 0x00, 0xFB, 0xFF],
            &[0x0],
            &|c| (c == rva(0x40D378)).then(|| crate::install::stand_ins::stand_in(0x10)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for eax in [0u32, 0x1234_5678] {
        let before = Regs {
            eax,
            ..Regs::SENTINEL
        };
        take_called();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (
            take_called(),
            [ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp],
        );
        let g = unsafe {
            stop_race_sounds(&before, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            take_called(),
            g.map(|g| [g.eax, g.ebx, g.ecx, g.edx, g.esi, g.edi, g.ebp])
                .unwrap_or_default(),
        );
        t.check(theirs == mine, || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
