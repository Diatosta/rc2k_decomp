//! `marks::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (`dword_5D1E34`),
//! `scene::depth_test_off`; `dword_939BA6` the settings' +0x7C's low 24 bits,
//! `g_marksKind1Choice` `dword_8BC50D` (the rain's drops); `marks::run_ring`; render state
//! 0x14 = 2; `marks::run_ribbons`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `run_ribbons` leaves them
//!
//! Caller: `marks::run_frame` (0x4D160B). Its calls through `part` (1 a render state, 2 the
//! texture, 3 the depth test, 4 `run_ring`, 5 `run_ribbons`); in the game their Rust
//! versions. Refuses null settings, before anything. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose calls go to stand-ins, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::marks::setup::SETTINGS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::ring_frame", rva(0x4CB5B9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
const TINT_RVA: usize = rva(0x939BA6); // dword_939BA6
const CHOICE_RVA: usize = rva(0x93A1D6); // g_marksKind1Choice
const DROPS_RVA: usize = rva(0x8BC50D); // dword_8BC50D
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn ring_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            part(1, &mut g)?;
        }
        g.eax = game_read(TEXTURE_RVA);
        part(2, &mut g)?;
        part(3, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        g.eax = ((g.esi + 0x7C) as *const u32).read_unaligned() & 0xFF_FFFF;
        game_write(TINT_RVA, g.eax);
        g.eax = game_read(DROPS_RVA);
        game_write(CHOICE_RVA, g.eax);
        part(4, &mut g)?;
        g.eax = 2;
        g.ebx = 0x14;
        part(1, &mut g)?;
        part(5, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(r, value, state);
            }
            2 => crate::scene::set_texture::set_texture_in_game(r),
            3 => crate::scene::depth_test_off::depth_test_off(r),
            4 => {
                use crate::marks::run_ring as ring;
                let left = ring::run_ring(r, &mut ring::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
            _ => {
                use crate::marks::run_ribbons as ribbons;
                let left = ribbons::run_ribbons(r, &mut ribbons::game_part)?;
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
    match unsafe { ring_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { ring_frame(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, 4
    let entry: &[u8] = &[0xB8, 0x04, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4CB5B9,
        0x94,
        entry,
        &[0xA, 0x19, 0x28, 0x37, 0x46, 0x50, 0x55, 0x7A, 0x89, 0x8E],
        &[
            (0x44EB72, 1),
            (0x44E77E, 2),
            (0x44E3D1, 3),
            (0x4CB64D, 4),
            (0x4CB71A, 5),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x40];
    let sa = settings.as_mut_ptr() as u32;
    let regions: Vec<Region> = vec![at(TINT_RVA), at(CHOICE_RVA)];
    let mut vars = regions.clone();
    vars.extend([SETTINGS_RVA, TEXTURE_RVA, DROPS_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..3u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                ((sa + 0x7C) as *mut u32).write_unaligned(0xAB12_3456u32.rotate_left(case));
                game_write(TEXTURE_RVA, 0x40 + case);
                game_write(DROPS_RVA, 7 * case);
            },
            &|r| unsafe { ring_frame(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
