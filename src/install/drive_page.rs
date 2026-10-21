//! `install::drive_page` (0x41425E): the drive page. At the start (ebx 0) the highlight
//! (`g_listChoice`) is set to the item of the typed drive letter (`byte_95EBF8`, lowered
//! through `g_lowerCase`): its place in the drives' texts (`byte_52A30F`, 0x10 bytes each,
//! first byte the letter; 26 of them) plus 2, unless that is past the items (`[edi+1]`). At
//! the end (ebx negative) the typed letter becomes the highlighted drive's
//! (`g_listChoice - 2`, if not below 0, times 0x10 into the texts), `byte_95EBF9` ':' and
//! `dword_51BA00` 0.
//!
//! In:  ebx the state, edi the list
//! Out: eax, ecx, edx as the original leaves them; esi kept; the flags of the end's `jb`
//! Caller `install::run_page`. Shadow mode compares.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::install::installer::{CHOICE, DRIVE_LETTERS, PATH_PROBLEM, TYPED, a, g, gb, p, pb, rb};
use crate::race::viewer::dispatch_regs;
use crate::settings::find_key::LOWER_RVA;
use rc2k_core::install::pages;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::drive_page", rva(0x41425E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn drive_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    if r.ebx == 0 {
        let letter: u8 = unsafe { game_read(LOWER_RVA + gb(TYPED) as usize) };
        let letters = a(DRIVE_LETTERS);
        // Converted: `rc2k_core::install::pages` (`drive_highlight`, `chosen_drive`).
        let at = |n: u32| letters + pages::DRIVE_TEXT * n;
        let (place, highlight) = pages::drive_highlight(letter, |n| rb(at(n)), rb(r.edi + 1));
        r.ecx = place;
        r.edx = at(place);
        r.eax = place.wrapping_add(2);
        if let Some(highlight) = highlight {
            p(CHOICE, highlight);
        }
    }
    r.set_zero(false);
    r.set_carry(false);
    if (r.ebx as i32) < 0 {
        r.edx = a(DRIVE_LETTERS);
        let chosen = pages::chosen_drive(g(CHOICE));
        r.eax = g(CHOICE).wrapping_sub(2);
        r.set_carry(chosen.is_none());
        if let Some(offset) = chosen {
            r.eax = offset & !0xFF | rb(offset.wrapping_add(r.edx)) as u32;
            pb(TYPED, r.eax as u8);
            pb(0x95EBF9, b':');
            p(PATH_PROBLEM, 0);
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(drive_page(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = crate::install::page_action_test::globals();
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::page_action_test::run(&mut t, call_original, &rust);
    t.finish()
}
