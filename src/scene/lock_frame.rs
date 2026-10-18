//! `scene::lock_frame` (0x44E837): the back surface locked (`gfx::lock_surface`, the
//! platform's) and its pitch and pixels kept for the drawing (`dword_4F4988` into
//! `dword_5C5FB4`, `dword_4F4984` into `dword_5C5FB0`).
//!
//! In:  nothing
//! Out: CF as the lock leaves it; eax the pixels; the rest as the lock leaves them
//!
//! Callers: `race::loading_bar_frame` and three of the game's (0x442675, 0x4435A0,
//! 0x4435C2). The lock through `part` (1), in the game the platform's call. Hooked through a private
//! copy (a call at the entry). Shadow mode runs the original alone (the platform). Self-test: a copy whose lock goes to a stand-in
//! (carry set and not), against the Rust version.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::lock_frame", rva(0x44E837), thunk, &ORIGINAL)
    .private_copy(0x1A)
    .relocate_calls(&[0x0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const PLATFORM_RVA: usize = rva(0x4058AD); // gfx::lock_surface

/// A call by place: 1 the platform's.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers (and flags) it leaves.
pub unsafe fn lock_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    part(1, &mut g)?;
    unsafe {
        game_write(rva(0x5C5FB4), game_read::<u32>(rva(0x4F4988)));
        g.eax = game_read(rva(0x4F4984));
        game_write(rva(0x5C5FB0), g.eax);
    }
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
    match unsafe { lock_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let entry: &[u8] = &[0xE8, 0x71, 0x70, 0xFB, 0xFF];
    let copy = match copy(0x44E837, 0x1A, entry, &[0x0], &[(0x4058AD, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = [rva(0x5C5FB4), rva(0x5C5FB0)].map(at).to_vec();
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
                lock_frame(r, &mut |p, r| {
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
