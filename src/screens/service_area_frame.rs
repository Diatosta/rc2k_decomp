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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};

/// The Rust version (see the header): the registers and flags it leaves.
pub unsafe fn service_area_frame(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let done = 'frame: {
        for steps in [&[1u32, 2, 3, 4][..], &[5, 10, 1], &[7, 3, 9]] {
            for &p in steps {
                crate::screens::service_area_frame::game_part(p, &mut g)?;
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
            1 => crate::calls::call(rva(0x4058AD), r),
            2 => crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
                &mut crate::gfx::copy_picture_to_surface::GameSurface,
            )?,
            3 => crate::calls::call(rva(0x405949), r),
            4 => crate::scene::begin_scene::begin_scene(r, &mut |r| {
                crate::calls::call(rva(0x41ABC7), r)
            }),
            5 => {
                *r = draw_service_boxes::draw_service_boxes(r, &mut draw_service_boxes::game_part)?
            }
            7 => {
                let carry =
                    draw_service_area::draw_service_area(&mut crate::text::draw_text::GameCanvas)?;
                r.set_carry(carry);
            }
            9 => crate::calls::call(rva(0x4057D1), r),
            _ => {
                crate::scene::end_scene::end_scene(r, &mut |r| crate::calls::call(rva(0x41AB8A), r))
            }
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    if crate::screens::blanked("screens::service_area_frame") {
        return Ok(());
    }
    let left = unsafe { service_area_frame(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
