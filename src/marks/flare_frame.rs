//! `marks::flare_frame` (0x4C4EA2): `scene::set_texture` with `dword_5D1E10`,
//! `scene::depth_test_off`, and when the settings' +0x2754 (the flare's strength) is set
//! `marks::draw_lens_flare`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: esi the settings unless the flare ran; the rest as the calls leave them
//!
//! Caller: `marks::run_frame` (0x4D1645). Its calls through `part` (1 the texture, 2 the
//! depth test, 3 the flare); in the game their Rust versions. Refuses null settings,
//! before anything. Shadow mode runs the original alone (it tells the device). Self-test:
//! a copy whose calls go to stand-ins, against the Rust version, the strength 0 and not.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::marks::setup::SETTINGS_RVA;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::flare_frame", rva(0x4C4EA2), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXTURE_RVA: usize = rva(0x5D1E10); // dword_5D1E10

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn flare_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        if game_read::<u32>(SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        g.eax = game_read(TEXTURE_RVA);
        part(1, &mut g)?;
        part(2, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        if ((g.esi + 0x2754) as *const u32).read_unaligned() != 0 {
            part(3, &mut g)?;
        }
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::set_texture::set_texture_in_game(r),
            2 => crate::scene::depth_test_off::depth_test_off(r),
            _ => {
                use crate::marks::draw_lens_flare as flare;
                let left = flare::draw_lens_flare(r, &mut flare::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
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
    match unsafe { flare_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { flare_frame(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_5D1E10
    let entry: &[u8] = &[0xA1, 0x10, 0x1E, 0x5D, 0x00];
    let copy = match copy(
        0x4C4EA2,
        0x31,
        entry,
        &[0x5, 0xA, 0x22, 0x27],
        &[(0x44E77E, 1), (0x44E3D1, 2), (0x4C4ED3, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2758 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let vars = [at(SETTINGS_RVA), at(TEXTURE_RVA)];
    let saved = capture(&vars);
    for strength in [0u32, 0x40] {
        run_both(
            &mut t,
            &format!("strength {strength:#x}"),
            &copy,
            &[],
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                ((sa + 0x2754) as *mut u32).write_unaligned(strength);
                game_write(TEXTURE_RVA, 0x55u32);
            },
            &|r| unsafe { flare_frame(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
