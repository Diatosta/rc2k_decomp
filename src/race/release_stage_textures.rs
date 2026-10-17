//! `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (0x4051D3, the
//! platform's: called in the game); a `call` and a `retn`.
//!
//! In:  nothing
//! Out: the registers as `gfx::release_all_textures` leaves them
//!
//! See `race/race_shell.rs`. Callers: `sub_444940` (`pushf` follows), `sub_443396` (a
//! call follows) and code at 0x44336D (a call follows): no flags read. It begins with its
//! call: private copy, the call relocated, manual patch. Shadow mode runs the original
//! alone (it releases the textures). Self-test: a copy whose call goes to a stand-in that
//! notes it, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva, shadow};
use crate::race::race_shell::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::release_stage_textures",
    rva(0x44E79F),
    thunk,
    &ORIGINAL,
)
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

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let g = unsafe { release_stage_textures(r, &mut game_release_textures) };
    *r = Regs { esp: r.esp, ..g };
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x44E79F)),
            0x6,
            &[0xE8, 0x2F, 0x6A, 0xFB, 0xFF],
            &[0x0],
            &|c| (c == RELEASE_TEXTURES_RVA).then(|| crate::install::stand_ins::stand_in(1)),
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
        let g = unsafe { release_stage_textures(&before, &mut |r| note(1, r)) };
        let mine = (
            take_called(),
            [g.eax, g.ebx, g.ecx, g.edx, g.esi, g.edi, g.ebp],
        );
        t.check(theirs == mine, || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
