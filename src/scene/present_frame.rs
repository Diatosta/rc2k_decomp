//! `scene::present_frame` (0x44E895): `gfx::show_frame` (the platform's), nothing more (a
//! call and a return; hooked through a private copy).
//!
//! In:  nothing
//! Out: as the platform's call leaves them (CF read by the callers)
//!
//! Callers: `race::loading_bar_frame` and the game's (0x4395FB, 0x439605, 0x4426A0,
//! 0x4435DD). Shadow mode runs the original alone. Self-test: a copy whose call goes to a
//! stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::present_frame", rva(0x44E895), thunk, &ORIGINAL)
    .private_copy(0x6)
    .relocate_calls(&[0x0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLATFORM_RVA: usize = rva(0x4057D1); // gfx::show_frame

/// A call by place: 1 the platform's.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers (and flags) it leaves.
pub unsafe fn present_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    part(1, &mut g)?;
    Ok(g)
}

/// In the game: the platform's function at its address.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { call_game(r, game_addr(PLATFORM_RVA)) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { present_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0xE8, 0x37, 0x6F, 0xFB, 0xFF];
    let copy = match copy(0x44E895, 0x6, entry, &[0x0], &[(0x4057D1, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![];
    let saved = capture(&regions);
    for carry in [0u32, 1] {
        crate::install::stand_ins::set_handler(if carry == 1 { carry_on } else { carry_off });
        run_both(
            &mut t,
            &format!("carry {carry}"),
            &copy,
            &regions,
            &|| {
                restore(&regions, &saved);
            },
            &|r| unsafe {
                present_frame(r, &mut |p, r| {
                    if carry == 1 {
                        carry_on(p, r)
                    } else {
                        carry_off(p, r)
                    };
                    Ok(())
                })
            },
        );
    }
    restore(&regions, &saved);
    t.finish()
}

fn carry_on(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    r.set_carry(true);
}

fn carry_off(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    r.set_carry(false);
}
