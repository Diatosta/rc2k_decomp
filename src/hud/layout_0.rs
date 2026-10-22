//! `hud::layout_0` (0x4B1D8C): the HUD of layout 0 (the record's +0xF4 neither 1 nor 2),
//! by the record's flags (+0xF8): 0x10 the dial (`hud::disc_0`) and a sprite; 0x40 the
//! stage bar (`hud::draw_stage_bar`), the marker (`hud::draw_marker`) and the two time
//! panels (sprite 0x29 + the record's +0x28 / +0x30 held to 0 .. 0x16, the times
//! `dword_75C074` / `dword_75C078` as digits); 8 the cut call (`sub_4B6A6D`: ecx 0 and an
//! empty function); always the icons (`hud::icons_0`); 0x20 two sprites (the second by the
//! record's +0x18) and the time +0x14 with its leading digits left out while they are 0;
//! 1 the sign by +0xCC (also into `dword_75BFAC`) and the needle (`hud::needle_0`); 0x80
//! the signs (`hud::draw_signs`); 2 the time +0x1C's last two digits, two sprites and the
//! time `dword_75C07C`. Textures (`dword_75C034`), the quad record's alpha and colour set
//! between, as the original.
//!
//! In:  ebp the HUD's record
//! Out: as the path leaves them (edx the last time's last digit, ebx 10 after a time)
//!
//! Caller: `sub_4B1CD0`. The draws through `part` (1 `add_sprite`, 2 `disc_0`, 3
//! `draw_stage_bar`, 4 `icons_0`, 5 `draw_marker`, 6 `draw_signs`, 7 `needle_0`), in the
//! game their Rust versions. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draws go to a stand-in, against the Rust version, flags and times by lot,
//! at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_marker::RECTS_RVA;
use crate::hud::draw_signs::row;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::hud::panels;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::layout_0", rva(0x4B1D8C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const TIMES_RVA: [usize; 3] = [rva(0x75C074), rva(0x75C078), rva(0x75C07C)];
pub const DIGITS_RVA: usize = rva(0x75C08C); // dword_75C08C .. dword_75C0A0: six
pub const SIGN_RVA: usize = rva(0x75C0A4); // dword_75C0A4
pub const PICK_RVA: usize = rva(0x75C040); // dword_75C040
pub const NEEDLE_KIND_RVA: usize = rva(0x75BFAC); // dword_75BFAC
pub const LEADING_RVA: usize = rva(0x75C03C); // dword_75C03C

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A time made digits (`dword_75C08C` ..: `rc2k_core::hud::panels::time_digits`), each
/// plus `base`; eax, ebx and edx as the divisions leave them.
pub unsafe fn time_digits(g: &mut Regs, value: u32, base: u32) -> [u32; 6] {
    let d = panels::time_digits(value, base);
    for (n, v) in d.into_iter().enumerate() {
        unsafe { game_write(DIGITS_RVA + 4 * n, v) };
    }
    [g.eax, g.ebx, g.edx] = panels::division_registers(&d);
    d
}

/// A sprite: rectangle `rect` of `dword_75B940` at (x, y).
pub fn sprite(g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part) -> Result<(), Refused> {
    g.eax = panels::rect_offset(rect);
    g.ebx = x;
    g.ecx = y;
    g.esi = (game_addr(RECTS_RVA) as u32).wrapping_add(g.eax);
    part(1, g)
}

/// The sign of a time into `dword_75C0A4`: `plus` or `minus`.
pub fn sign(value: u32, plus: u32, minus: u32) -> u32 {
    let s = panels::sign(value, plus, minus);
    unsafe { game_write(SIGN_RVA, s) };
    s
}

pub fn set(r: usize, v: u32) {
    unsafe { game_write(r, v) };
}

/// A sprite drawer for the shared panels: (registers, rectangle, x, y).
pub type Draw<'a> = &'a mut dyn FnMut(&mut Regs, u32, u32, u32) -> Result<(), Refused>;

/// A clock panel: the time `t` (digits from 0x18), its sign 0x22 / 0x23, then its
/// rectangles (`panels::clock_rects`) at `xs`, row `y`.
pub unsafe fn clock_panel(
    g: &mut Regs,
    t: u32,
    xs: [u32; 8],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, t, panels::CLOCK_BASE) };
    sign(t, 0x22, 0x23);
    for (rect, x) in panels::clock_rects(t).into_iter().zip(xs) {
        draw(g, rect, x, y)?;
    }
    Ok(())
}

/// A split-time panel: the time `t` (digits from 0xC, sign 0 / 1), then its rectangles
/// (`panels::lap_rects`) at `xs`, row `y`.
pub unsafe fn lap_panel(
    g: &mut Regs,
    t: u32,
    xs: [u32; 8],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, t, panels::LAP_BASE) };
    sign(t, 0, 1);
    for (rect, x) in panels::lap_rects(t).into_iter().zip(xs) {
        draw(g, rect, x, y)?;
    }
    Ok(())
}

/// The time `v` (digits from 0xC, sign 0 / 1) as its last three digits at `xs`, row `y`,
/// the leading ones left out while they are 0 (`dword_75C03C`;
/// `panels::leading_digits`).
pub unsafe fn leading_time(
    g: &mut Regs,
    v: u32,
    xs: [u32; 3],
    y: u32,
    draw: Draw,
) -> Result<(), Refused> {
    unsafe { time_digits(g, v, panels::LAP_BASE) };
    sign(v, 0, 1);
    set(LEADING_RVA, 1);
    for (digit, slot, drawn) in panels::leading_digits(v) {
        g.eax = digit;
        if !drawn {
            continue;
        }
        draw(g, digit, xs[slot], y)?;
        if slot < 2 {
            set(LEADING_RVA, 0);
        }
    }
    Ok(())
}

/// The needle's sign by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture, as the original writes them): its rectangle.
pub fn needle_sign(field: u32) -> u32 {
    set(PICK_RVA, 0x5E);
    set(TEXTURE_RVA, 0xC0);
    set(NEEDLE_KIND_RVA, field);
    let (pick, texture) = panels::needle_sign(field);
    if field != 0 {
        set(PICK_RVA, pick);
        set(TEXTURE_RVA, texture);
    }
    pick
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::panels` (the digits, signs, panels, leading digits, the
// stage panel and the needle's sign)
pub unsafe fn layout_0(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let get = |o: u32| unsafe { ((entry.ebp + o) as *const u32).read_unaligned() };
    let flags = get(0xF8);
    set(QUAD_RVA, 0xE0);
    set(TEXTURE_RVA, 0xC6);
    if flags & 0x10 != 0 {
        part(2, &mut g)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if flags & 0x40 != 0 {
        part(3, &mut g)?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    if flags & 8 != 0 {
        g.ecx = 0;
    }
    set(TEXTURE_RVA, 0xDC);
    part(4, &mut g)?;
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    set(TEXTURE_RVA, 0xC1);
    if flags & 0x20 != 0 {
        sprite(&mut g, 0x49, 9, 0x215, &mut part)?;
        sprite(&mut g, get(0x18).wrapping_add(0x41), 0x11, 0x21D, &mut part)?;
    }
    let pick = needle_sign(get(0xCC));
    if flags & 1 != 0 {
        sprite(&mut g, pick, 0x21, 0x1B8, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if flags & 0x10 != 0 {
        sprite(&mut g, 0x5F, 0x282, 0x1B8, &mut part)?;
    }
    if flags & 0x40 != 0 {
        part(5, &mut g)?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    if flags & 0x80 != 0 {
        part(6, &mut g)?;
    }
    set(TEXTURE_RVA, 0xC1);
    if flags & 1 != 0 {
        part(7, &mut g)?;
    }
    if flags & 0x40 != 0 {
        let panels = [
            (
                0x28u32,
                0x4Cu32,
                TIMES_RVA[0],
                [0x14u32, 0x22, 0x2F, 0x36, 0x44, 0x51, 0x58, 0x66],
            ),
            (
                0x30,
                0x2F8,
                TIMES_RVA[1],
                [0x2BD, 0x2CB, 0x2D8, 0x2DF, 0x2ED, 0x2FA, 0x301, 0x30F],
            ),
        ];
        for (field, x, time, xs) in panels {
            let Some(rect) = panels::stage_rect(get(field)) else {
                continue;
            };
            sprite(&mut g, rect, x, 1, &mut part)?;
            let t: u32 = unsafe { game_read(time) };
            unsafe {
                clock_panel(&mut g, t, xs, 0x13, &mut |g, r, x, y| {
                    sprite(g, r, x, y, &mut part)
                })
            }?;
        }
    }
    if flags & 2 != 0 {
        let v = get(0x1C);
        if v != 0xFFFF_FFFF {
            let d = unsafe { time_digits(&mut g, v, 2) };
            sign(v, 0, 1);
            sprite(&mut g, d[4], 0x60, row(0x36), &mut part)?;
            sprite(&mut g, d[5], 0x8C, row(0x36), &mut part)?;
        }
        sprite(&mut g, 0, 0xE, row(0x33), &mut part)?;
        sprite(&mut g, 0x4A, 0xF, row(0x58), &mut part)?;
        let t: u32 = unsafe { game_read(TIMES_RVA[2]) };
        let xs = [0x2E, 0x42, 0x56, 0x5E, 0x72, 0x86, 0x8E, 0xA2];
        unsafe {
            lap_panel(&mut g, t, xs, row(0x63), &mut |g, r, x, y| {
                sprite(g, r, x, y, &mut part)
            })
        }?;
    }
    if flags & 0x20 != 0 {
        let v = get(0x14);
        unsafe {
            leading_time(&mut g, v, [0x43, 0x57, 0x6B], 0x23F, &mut |g, r, x, y| {
                sprite(g, r, x, y, &mut part)
            })
        }?;
    }
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => return draw_marker::game_part(place, r),
            2 => dials::set_up(r, &disc_0::SETUP, &mut add_triangle::game_part_disc)?,
            3 => draw_stage_bar::draw_stage_bar(r, &mut draw_stage_bar::game_part)?,
            4 => icons_0::icons_0(r, &mut icons_0::game_part)?,
            5 => draw_marker::draw_marker(r, &mut draw_marker::game_part)?,
            6 => draw_signs::draw_signs(r, &mut draw_signs::game_part)?,
            _ => dials::set_up(r, &needle_0::SETUP, &mut draw_needle::game_part_needle)?,
        };
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { layout_0(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { layout_0(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// What the layouts write themselves (for their tests).
pub fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [
        TEXTURE_RVA,
        QUAD_RVA,
        QUAD_RVA + 0x14,
        PICK_RVA,
        NEEDLE_KIND_RVA,
        LEADING_RVA,
        SIGN_RVA,
    ]
    .map(at)
    .to_vec();
    r.push((game_addr(DIGITS_RVA) as u32, 0x18));
    r
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls = [
        0x20, 0x3B, 0x60, 0x6F, 0xB6, 0xD6, 0x13C, 0x16F, 0x180, 0x19B, 0x1B6, 0x204, 0x2BB, 0x2D8,
        0x2F5, 0x312, 0x32F, 0x34C, 0x369, 0x386, 0x3C4, 0x47B, 0x498, 0x4B5, 0x4D2, 0x4EF, 0x50C,
        0x529, 0x546, 0x61E, 0x647, 0x670, 0x699, 0x75C, 0x785, 0x7AE, 0x7D7, 0x800, 0x829, 0x852,
        0x87B, 0x955, 0x98A, 0x9B1,
    ];
    // mov g_hudQuad, 0E0h
    let entry: &[u8] = &[0xC7, 0x05, 0xB0, 0xBF, 0x75, 0x00, 0xE0, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x4B1D8C,
        0x9B7,
        entry,
        &calls,
        &[
            (0x4B70EE, 1),
            (0x4B6BFD, 2),
            (0x4B5A86, 3),
            (0x4B57D8, 4),
            (0x4B55AB, 5),
            (0x4B4F2B, 6),
            (0x4B6DB9, 7),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(TIMES_RVA.map(at));
    vars.push(at(crate::hud::draw_signs::FLAT_RVA));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..6u32).map(move |c| (w, c)))
    {
        let mut record = [0u32; 0x40];
        record[0xF8 / 4] = [0xFFu32, 0, 0x55, 0xAA, 0x62, 0x3D][case as usize];
        record[0x14 / 4] = [0u32, 1234, 59_999, 0xFFFF_FF00, 6_000, 361_234][case as usize];
        record[0x18 / 4] = case;
        record[0x1C / 4] = if case == 2 { 0xFFFF_FFFF } else { 4321 * case };
        record[0x28 / 4] = [5u32, 0xFFFF_FFFF, 0xFFFF_FFF0, 30, 0x16, 3][case as usize];
        record[0x30 / 4] = [7u32, 9, 0xFFFF_FFFF, 0, 40, 2][case as usize];
        record[0xCC / 4] = case & 1;
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(TIMES_RVA[0], 65_432 * case);
            game_write(TIMES_RVA[1], (12_345 * case).wrapping_neg());
            game_write(TIMES_RVA[2], 777 + case);
            game_write(crate::hud::draw_signs::FLAT_RVA, case & 1);
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
            layout_0(&before, &mut |p, r| {
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
            format!("control word {word:#x}, case {case}: original {:x?} {} calls; rust {:x?} {} calls; same calls {}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), theirs.1 == mine.1, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}
