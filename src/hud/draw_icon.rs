//! `hud::draw_icon` (0x4B587E): a square icon of the HUD record's entry at ebp + edi
//! (none when its +0x84 is -1): its width (+0x88) and drop (+0x90) times `dword_75C06C`
//! (`fistp` into `dword_75C064` / `dword_75C068`), centred on 0x320 / 2 plus
//! `dword_75C070`, its top ebx plus the drop, as tall as wide; texture places 0 / 1;
//! colour white under its +0x8C; `hud::add_polygon` on four (inside `pusha` .. `popa`).
//!
//! In:  ebp the record, edi the entry's offset, ebx the top
//! Out: ebx the bottom, edx the colour, esi the left, edi the right (kept when none)
//!
//! Callers: `hud::icons_0`, `hud::icons_1`, `hud::icons_2`. The polygon through `part`
//! (1), in the game its Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version, entries by
//! lot, at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::batch::{POLYGON_COUNT_RVA, POLYGON_RVA};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::hud::icons;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_icon", rva(0x4B587E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const SIZE_RVA: [usize; 2] = [rva(0x75C064), rva(0x75C068)]; // dword_75C064, dword_75C068
pub const SCALE_RVA: usize = rva(0x75C06C); // dword_75C06C
pub const SHIFT_RVA: usize = rva(0x75C070); // dword_75C070
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060

/// A call by place: 1 `add_polygon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::icons`
pub unsafe fn draw_icon(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let e = g.ebp.wrapping_add(g.edi);
        let get = |o: u32| ((e + o) as *const u32).read_unaligned();
        if !icons::shown(get(0x84)) {
            return Ok(g);
        }
        let scale = f32::from_bits(game_read(SCALE_RVA));
        let width = get(0x88);
        game_write(SIZE_RVA[0], icons::scaled(width, scale));
        let drop = get(0x90);
        game_write(SIZE_RVA[1], icons::scaled(drop, scale));
        let i = icons::icon(width, get(0x8C), drop, scale, game_read(SHIFT_RVA), g.ebx);
        [g.ebx, g.edx, g.esi, g.edi] = [i.bottom, i.colour, i.left, i.right];
        for (n, u, v, x, y) in icons::corners(&i) {
            let at = POLYGON_RVA + 0x20 * n;
            game_write(at + 0x18, [u, v].map(|k| (k as i32 as f32).to_bits()));
            game_write(at, [x, y].map(|k| (k as i32 as f32).to_bits()));
            game_write(at + 0x10, [i.colour, 0]);
            // `fild` through `dword_75C060`: it keeps the last.
            game_write(SCRATCH_RVA, y);
        }
        game_write(POLYGON_COUNT_RVA, 4u32);
        let mut r = g;
        part(1, &mut r)?;
    }
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_icon(r, &mut crate::hud::add_quad::game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_icon(r, &mut crate::hud::add_quad::game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword [ebp+edi+84h], -1
    let entry: &[u8] = &[0x83, 0xBC, 0x3D, 0x84, 0x00, 0x00, 0x00, 0xFF];
    let copy = match copy(0x4B587E, 0x208, entry, &[0x201], &[(0x4B7413, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(POLYGON_RVA) as u32, 0x80),
        at(POLYGON_COUNT_RVA),
        at(SCRATCH_RVA),
        at(SIZE_RVA[0]),
        at(SIZE_RVA[1]),
    ];
    let mut vars = regions.clone();
    vars.extend([SCALE_RVA, SHIFT_RVA].map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..5u32).map(move |c| (w, c)))
    {
        let mut record = vec![0u32; 0x40];
        let e = (0x84 + 0x10 * (case % 2) as usize) / 4;
        record[e] = if case == 4 { 0xFFFF_FFFF } else { 1 };
        record[e + 1] = 40 + case * 9;
        record[e + 2] = 0x80 + case;
        record[e + 3] = 12 * case;
        let before = Regs {
            ebp: record.as_ptr() as u32,
            edi: 0x10 * (case % 2),
            ebx: 0xC8,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(SCALE_RVA, (1.0f32 + case as f32 * 0.25).to_bits());
            game_write(SHIFT_RVA, if case == 2 { 0xFFFF_FF38u32 } else { 0 });
            std::ptr::write_bytes(game_addr(POLYGON_RVA) as *mut u8, 0xAA, 0x80);
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
            draw_icon(&before, &mut |p, r| {
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
