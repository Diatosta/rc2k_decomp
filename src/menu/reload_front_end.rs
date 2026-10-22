//! `menu::reload_front_end` (0x48140D): when `dword_7587E8` is 1 (the scene was let go for
//! a race), the front end's scene set up again: `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! 36 callers (the menus after their races). The three calls through `part` (1 the engine,
//! 2 the scene), in the game their Rust versions (`set_up_engine_in_game`,
//! `set_up_front_end_scene_in_game`). Shadow mode runs the original alone (it loads the
//! scene). Self-test: a copy whose calls go to stand-ins, against the Rust version, with
//! the switch 0 and 1.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::reload_front_end", rva(0x48140D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const SWITCH_RVA: usize = rva(0x7587E8); // dword_7587E8

/// A call by place: 1 `set_up_engine`, 2 `set_up_front_end_scene`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reload_front_end(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    g.eax = unsafe { game_read(SWITCH_RVA) };
    if g.eax == 1 {
        for place in [1, 2, 1] {
            part(place, &mut g)?;
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::set_up_engine::set_up_engine_in_game(r),
            _ => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { reload_front_end(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { reload_front_end(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy(
        0x48140D,
        0x1C,
        // pusha; mov eax, dword_7587E8
        &[0x60, 0xA1, 0xE8, 0x87, 0x75, 0x00],
        &[0xB, 0x10, 0x15],
        &[(0x442AF6, 1), (0x474E04, 2)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = vec![at(SWITCH_RVA)];
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("switch {case}"),
            &copy,
            &[],
            &|| unsafe { game_write(SWITCH_RVA, case) },
            &|r| unsafe { reload_front_end(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
