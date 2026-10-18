//! `marks::swap_mask` (0x4C95F5): when the settings' +0x2704 differs from the last seen
//! (`dword_934D6E`, then it), the marks' mask texture is refilled from `g_marksMaskBlock`
//! (or `g_marksBMaskBlock` when it is set; the choice into `dword_934D72`): the texture slot
//! `dword_5D0E10` locked (`sub_41AA4B`, the platform's), rows 0 .. 0x7D of 0x100 pixels
//! (`g_surface`'s width each) put at y + 0x80 by `gfx::put_surface_pixel`, unlocked
//! (`sub_41AB02`), and slot `dword_5D1E10` forgotten (`scene::forget_slot_texture`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: unchanged when nothing changed; else eax `dword_5D1E10`, ecx the width, edx 0x7E,
//!      ebp and esi as the last pixel left them
//!
//! Caller: `marks::run_frame` (0x4D1549). Its calls through `part` (1 the lock, 2 a pixel,
//! 3 the unlock, 4 the forgetting); in the game the platform's two through the game
//! (section 15), the others their Rust versions. Refuses null settings (before anything) and
//! a locked texture (`g_surface` after the lock) wider than the mask's 0x100 (unlocked again,
//! what was written put back). Shadow mode runs the original alone (it tells
//! the device). Self-test: a copy whose calls go to stand-ins, against the Rust version,
//! the switch changed or not, on a narrow test surface width.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::setup::SETTINGS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::swap_mask", rva(0x4C95F5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SEEN_RVA: usize = rva(0x934D6E); // dword_934D6E
const CHOSEN_RVA: usize = rva(0x934D72); // dword_934D72
const MASK_RVA: usize = rva(0x934DB6); // g_marksMaskBlock
const BMASK_RVA: usize = rva(0x934DFA); // g_marksBMaskBlock
const SLOT_RVA: usize = rva(0x5D0E10); // dword_5D0E10
const FORGET_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const LOCK_RVA: usize = rva(0x41AA4B); // the platform's texture lock
const UNLOCK_RVA: usize = rva(0x41AB02); // the platform's texture unlock
const ROWS: u32 = 0x7E;

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn swap_mask(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        g.esi = s;
        let switch = ((s + 0x2704) as *const u32).read_unaligned();
        g.eax = switch;
        if g.eax == game_read::<u32>(SEEN_RVA) {
            return Ok(g);
        }
        let kept: [u32; 2] = [game_read(SEEN_RVA), game_read(CHOSEN_RVA)];
        game_write(SEEN_RVA, g.eax);
        g.eax = game_read(if switch != 0 { BMASK_RVA } else { MASK_RVA });
        game_write(CHOSEN_RVA, g.eax);
        g.eax = game_read(SLOT_RVA);
        part(1, &mut g)?;
        // The width is the locked texture's (`g_surface` while it is locked), not the
        // screen's: checked here (2026-10-18: checked before the lock, it refused every
        // frame). A row wider than the mask's would read past its block: unlocked, put
        // back and refused.
        if game_read::<u32>(WIDTH_RVA) > 0x100 {
            let mut u = Regs {
                eax: game_read(SLOT_RVA),
                ..g
            };
            part(3, &mut u)?;
            game_write(SEEN_RVA, kept[0]);
            game_write(CHOSEN_RVA, kept[1]);
            return Err(BadInput("a locked mask texture wider than the mask's 0x100").into());
        }
        g.edx = 0;
        loop {
            g.ecx = 0;
            loop {
                g.ebp = (g.edx << 8).wrapping_add(g.ecx);
                g.esi = game_read(CHOSEN_RVA);
                g.eax = ((g.esi + 4 * g.ebp) as *const u32).read_unaligned();
                let row = g.edx;
                g.edx = row + 0x80;
                part(2, &mut g)?;
                g.edx = row;
                g.ecx += 1;
                if g.ecx >= game_read::<u32>(WIDTH_RVA) {
                    break;
                }
            }
            g.edx += 1;
            if g.edx >= ROWS {
                break;
            }
        }
        g.eax = game_read(SLOT_RVA);
        part(3, &mut g)?;
        g.eax = game_read(FORGET_RVA);
        part(4, &mut g)?;
    }
    Ok(g)
}

/// In the game: the platform's lock and unlock through the game, the others' Rust
/// versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => call_game(r, game_addr(LOCK_RVA)),
            2 => {
                let left = crate::gfx::put_surface_pixel::put_surface_pixel(r)?;
                r.eax = left.eax;
            }
            3 => call_game(r, game_addr(UNLOCK_RVA)),
            _ => crate::scene::forget_slot_texture::forget_slot_texture(r.eax)?,
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
    match unsafe { swap_mask(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov esi, g_marksSettings
    let entry: &[u8] = &[0x8B, 0x35, 0x28, 0x7A, 0x76, 0x00];
    let copy = match copy(
        0x4C95F5,
        0x86,
        entry,
        &[0x36, 0x5C, 0x76, 0x80],
        &[(0x41AA4B, 1), (0x4065BB, 2), (0x41AB02, 3), (0x44EB66, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut settings = vec![0u32; 0x2708 / 4];
    let sa = settings.as_mut_ptr() as u32;
    let mut masks = vec![0u32; 2 * 0x7E * 0x100];
    let ma = masks.as_mut_ptr() as u32;
    for (n, v) in masks.iter_mut().enumerate() {
        *v = (n as u32).wrapping_mul(0x9E37_79B9);
    }
    let regions: Vec<Region> = vec![at(SEEN_RVA), at(CHOSEN_RVA)];
    let mut vars = regions.clone();
    vars.extend(
        [
            SETTINGS_RVA,
            MASK_RVA,
            BMASK_RVA,
            WIDTH_RVA,
            SLOT_RVA,
            FORGET_RVA,
        ]
        .map(at),
    );
    let saved = capture(&vars);
    for (case, (switch, seen, width)) in [(0u32, 0u32, 4u32), (1, 0, 3), (1, 1, 4), (0, 5, 0)]
        .into_iter()
        .enumerate()
    {
        run_both(
            &mut t,
            &format!("switch {switch}, seen {seen}, width {width}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(SETTINGS_RVA, sa);
                ((sa + 0x2704) as *mut u32).write_unaligned(switch);
                game_write(SEEN_RVA, seen);
                game_write(MASK_RVA, ma);
                game_write(BMASK_RVA, ma + 4 * 0x7E * 0x100);
                game_write(WIDTH_RVA, width);
                game_write(SLOT_RVA, 0x30 + case as u32);
                game_write(FORGET_RVA, 0x40u32);
            },
            &|r| unsafe { swap_mask(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop((settings, masks));
    t.finish()
}
