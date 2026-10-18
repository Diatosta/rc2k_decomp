//! `screens::service_area_frame` (0x46CFD7): one frame of the service area (the callback
//! `screens::run_frame` is given): the back surface locked (`gfx::lock_surface`), the
//! picture copied in (`gfx::copy_picture_to_surface`), unlocked; `scene::begin_scene`,
//! `screens::draw_service_boxes`, `scene::end_scene`; locked again, the text and icons
//! (`screens::draw_service_area`), unlocked; `gfx::show_frame`. A lock, the scene's begin
//! or the show failing: carry set.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set on a failure, else clear with ZF
//!      from the show's eax
//!
//! Callers: through the frame callbacks of `service::apply_repairs` and its kind (stored at
//! 0x46C33E .. 0x46CF7C). Hooked through a private copy (a call in the first bytes). The
//! calls through `part` (1 lock, 2 the picture, 3 unlock, 4 begin, 5 the boxes, 7 the text,
//! 9 show, 10 end), in the game their Rust
//! versions (the lock, unlock, show and the scene's layer the platform's). Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose calls go to stand-ins setting
//! carry by the case, against the Rust version, every way out.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::service_area_frame",
    rva(0x46CFD7),
    thunk,
    &ORIGINAL,
)
.private_copy(0x42)
.relocate_calls(&CALLS)
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls' offsets (the first inside the patched bytes: hence the private copy).
const CALLS: [usize; 10] = [0x1, 0x8, 0xD, 0x12, 0x19, 0x1E, 0x23, 0x2A, 0x2F, 0x34];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers and flags it leaves.
pub unsafe fn service_area_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    let done = 'frame: {
        for steps in [&[1u32, 2, 3, 4][..], &[5, 10, 1], &[7, 3, 9]] {
            for &p in steps {
                part(p, &mut g)?;
                if matches!(p, 1 | 4 | 9) && g.carry() {
                    break 'frame false;
                }
            }
        }
        true
    };
    let mut left = *entry;
    left.eflags = g.eflags;
    if done {
        left.set_carry(false);
        left.set_zero(g.eax == 0);
    } else {
        left.set_carry(true);
    }
    Ok(left)
}

/// In the game: the calls' Rust versions (the platform's through their addresses).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::screens::{draw_service_area, draw_service_boxes};
    unsafe {
        match place {
            1 => call_game(r, game_addr(rva(0x4058AD))),
            2 => crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
                &mut crate::gfx::copy_picture_to_surface::GameSurface,
            )?,
            3 => call_game(r, game_addr(rva(0x405949))),
            4 => crate::scene::begin_scene::begin_scene(r, &mut |r| {
                call_game(r, game_addr(rva(0x41ABC7)))
            }),
            5 => {
                *r = draw_service_boxes::draw_service_boxes(r, &mut draw_service_boxes::game_part)?
            }
            7 => {
                let carry =
                    draw_service_area::draw_service_area(&mut crate::text::draw_text::GameCanvas)?;
                r.set_carry(carry);
            }
            9 => call_game(r, game_addr(rva(0x4057D1))),
            _ => crate::scene::end_scene::end_scene(r, &mut |r| {
                call_game(r, game_addr(rva(0x41AB8A)))
            }),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if crate::screens::blanked("screens::service_area_frame") {
        return;
    }
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { service_area_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The places whose stand-ins set carry in this case (a bit each).
static CARRY: AtomicU32 = AtomicU32::new(0);

fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    r.set_carry(CARRY.load(Ordering::SeqCst) & (1 << place) != 0);
    r.eax = place;
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::take_called;
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call gfx::lock_surface
    let entry: &[u8] = &[0x60, 0xE8, 0xD0, 0x88, 0xF9, 0xFF];
    let copy = match copy(
        0x46CFD7,
        0x42,
        entry,
        &CALLS,
        &[
            (0x4058AD, 1),
            (0x4060E3, 2),
            (0x405949, 3),
            (0x44E8A1, 4),
            (0x4940BC, 5),
            (0x44E952, 10),
            (0x4A5FA2, 7),
            (0x4057D1, 9),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let regs = |r: &Regs| {
        (
            [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
            r.carry(),
            r.eflags & 0x40 != 0,
        )
    };
    for carry in [0u32, 1 << 1, 1 << 4, 1 << 9, 0x200 | 0x10] {
        CARRY.store(carry, Ordering::SeqCst);
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_called());
        let mine = unsafe {
            service_area_frame(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (mine.as_ref().ok().map(regs), take_called());
        t.check(theirs == mine, || {
            format!("carry {carry:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
