//! `hud::draw_marker` (0x4B55AB): a marker sprite (rectangle 0x5C of `dword_75B940`) along
//! the top at the share +0x6C over +0x68 of the HUD's record, times `flt_75C0A8` plus
//! `flt_75C0AC` (`fistp` through `dword_75C060`), coloured `dword_75BF50`.
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the x, ecx 0, esi the rectangle (as `add_sprite` keeps them)
//!
//! Caller: layout 0's drawer (0x4B1F0C). The sprite through `part` (1), in the game
//! `add_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, shares by
//! lot, at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::QUAD_RVA;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::hud::quads::marker_x;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_marker", rva(0x4B55AB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COLOUR_RVA: usize = rva(0x75BF50); // dword_75BF50
const K_RVA: [usize; 2] = [rva(0x75C0A8), rva(0x75C0AC)];
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
pub const RECTS_RVA: usize = rva(0x75B940); // dword_75B940: 0x10 a rectangle
const RECT: u32 = 0x5C;

/// A call by place: 1 `add_sprite`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::marker_x`
pub unsafe fn draw_marker(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(QUAD_RVA + 0x14, game_read::<u32>(COLOUR_RVA));
        let get = |o: u32| ((g.ebp + o) as *const u32).read_unaligned();
        let f = f32::from_bits;
        let x = marker_x(
            f(get(0x6C)),
            f(get(0x68)),
            f(game_read(K_RVA[0])),
            f(game_read(K_RVA[1])),
        );
        game_write(SCRATCH_RVA, x);
        g.eax = RECT * 0x10;
        g.ebx = x;
        g.ecx = 0;
        g.esi = game_addr(RECTS_RVA) as u32 + g.eax;
        part(1, &mut g)?;
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_marker(r, &mut crate::hud::draw_marker::game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// In the game: `add_sprite`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_sprite as s;
    *r = unsafe { s::add_sprite(r, &mut crate::hud::add_triangle::game_part_quad) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, dword_75BF50
    let entry: &[u8] = &[0xA1, 0x50, 0xBF, 0x75, 0x00];
    let copy = match copy(0x4B55AB, 0x43, entry, &[0x3D], &[(0x4B70EE, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![at(QUAD_RVA + 0x14), at(SCRATCH_RVA)];
    let mut vars = regions.clone();
    vars.extend([COLOUR_RVA, K_RVA[0], K_RVA[1]].map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..4u32).map(move |c| (w, c)))
    {
        let mut record = [0u32; 0x20];
        record[0x68 / 4] = 1000.0f32.to_bits();
        record[0x6C / 4] = (case as f32 * 333.3).to_bits();
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(COLOUR_RVA, 0x00FF_8040u32 + case);
            game_write(K_RVA[0], 560.0f32.to_bits());
            game_write(K_RVA[1], 40.0f32.to_bits());
        };
        prepare();
        take_called();
        let mut ro = before;
        with_control_word(word, &mut || unsafe {
            crate::framework::call_game(&mut ro, copy.addr())
        });
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_marker(&before, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "control word {word:#x}, case {case}: original {:x?} {:x?}; rust {:x?} {:x?}; same state {}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2 == mine.2
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
