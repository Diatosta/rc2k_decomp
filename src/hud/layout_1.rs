//! `hud::layout_1` (0x4B2743): the HUD of layout 1, two players side by side (the record's
//! +0xF8 flags the first's, +0xFC the second's): the dials (`hud::disc_1a` / `_1b`), a
//! black divider, the stage bars (`hud::draw_stage_bars_1`), the cut calls (`sub_4B6A78` /
//! `sub_4B6A83`: ecx 1 / 4 and an empty function), the icons (`hud::icons_1`), each
//! player's sprites scaled 0xC0 over `flt_75C00C` (`hud::add_scaled_sprite`), the markers
//! (`hud::draw_markers_1`), signs (`hud::draw_signs_1a` / `_1b`), needles
//! (`hud::needle_1a` / `_1b`), each player's time with its leading digits left out, the
//! four clocks (`dword_75C074`, `dword_75C078`, `dword_75C080`, `dword_75C084`, by the
//! first player's flag 0x40 and +0x28), and each player's lap panel (+0x1C / +0x4C,
//! `dword_75C07C` / `dword_75C088`). Textures, alpha and colour set between, as the
//! original.
//!
//! In:  ebp the HUD's record
//! Out: as the path leaves them
//!
//! Caller: `sub_4B1CD0`. The draws through `part` (1 `add_sprite`, 2 `disc_1a`, 3
//! `disc_1b`, 4 `add_solid_quad`, 5 `draw_stage_bars_1`, 6 `icons_1`, 7
//! `draw_markers_1`, 8 / 9 the signs, 10 / 11 the needles, 12 `add_scaled_sprite` with its
//! two scales), in the game their Rust versions. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose draws go to stand-ins (the scaled sprites' reading
//! their scales), against the Rust version, flags and times by lot, at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_markers_1::{int_over_80, taken_off};
use crate::hud::draw_signs::row;
use crate::hud::layout_0::*;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::layout_1", rva(0x4B2743), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
pub const CLOCKS_RVA: [usize; 2] = [rva(0x75C080), rva(0x75C084)];
pub const LAP_2_RVA: usize = rva(0x75C088); // dword_75C088
pub const SCALED: u32 = 12;

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// A sprite scaled 0xC0 over `flt_75C00C` both ways; the scales then taken off into
/// `dword_75C060`.
pub fn scaled(g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part) -> Result<(), Refused> {
    g.eax = rect.wrapping_mul(0x10);
    g.ebx = x;
    g.ecx = y;
    g.esi = (game_addr(crate::hud::draw_marker::RECTS_RVA) as u32).wrapping_add(g.eax);
    let k: u32 = unsafe { game_read(UV_SCALE_RVA) };
    set(SCRATCH_RVA, 0xC0);
    let v = int_over_80(0xC0, k);
    part(SCALED, g, &[v, v])?;
    set(SCRATCH_RVA, taken_off(v));
    Ok(())
}

/// The sign of the needle by the record's `field` (`dword_75C040`, `dword_75BFAC`, the
/// texture).
pub fn pick(field: u32) -> u32 {
    needle_sign(field)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::panels` (through `layout_0`'s panels and `needle_sign`),
// `rc2k_core::hud::quads::int_over` (the sprites' scales)
pub unsafe fn layout_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    let get = |o: u32| unsafe { ((entry.ebp + o) as *const u32).read_unaligned() };
    let [f1, f2] = [get(0xF8), get(0xFC)];
    let plain = |g: &mut Regs, rect: u32, x: u32, y: u32, part: &mut Part| {
        g.eax = rect.wrapping_mul(0x10);
        g.ebx = x;
        g.ecx = y;
        g.esi = (game_addr(crate::hud::draw_marker::RECTS_RVA) as u32).wrapping_add(g.eax);
        part(1, g, &[])
    };
    set(QUAD_RVA, 0xE0);
    set(TEXTURE_RVA, 0xC6);
    if f1 & 0x10 != 0 {
        part(2, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC7);
    if f2 & 0x10 != 0 {
        part(3, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC0);
    set(QUAD_RVA, 0xFF);
    set(QUAD_RVA + 0x14, 0);
    for (n, v) in [0x18F, 0, 0x191, 0x258].into_iter().enumerate() {
        set(QUAD_RVA + 4 + 4 * n, v);
    }
    part(4, &mut g.clone(), &[])?;
    if f1 & 0x40 != 0 {
        part(5, &mut g, &[])?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    if f1 & 8 != 0 {
        g.ecx = 1;
    }
    if f2 & 8 != 0 {
        g.ecx = 4;
    }
    set(TEXTURE_RVA, 0xDC);
    part(6, &mut g, &[])?;
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    set(QUAD_RVA, 0xFF);
    set(TEXTURE_RVA, 0xC1);
    if f1 & 0x20 != 0 {
        scaled(&mut g, 0x49, 7, 0x223, &mut part)?;
        scaled(&mut g, get(0x18).wrapping_add(0x41), 0xD, 0x22A, &mut part)?;
    }
    if f2 & 0x20 != 0 {
        scaled(&mut g, 0x49, 0x198, 0x223, &mut part)?;
        scaled(
            &mut g,
            get(0x48).wrapping_add(0x41),
            0x19E,
            0x22A,
            &mut part,
        )?;
    }
    let p = pick(get(0xCC));
    if f1 & 1 != 0 {
        scaled(&mut g, p, 0x19, 0x1DE, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if f1 & 0x10 != 0 {
        scaled(&mut g, 0x5F, 0x116, 0x1DE, &mut part)?;
    }
    let p = pick(get(0xD0));
    if f2 & 1 != 0 {
        scaled(&mut g, p, 0x1AA, 0x1DE, &mut part)?;
    }
    set(TEXTURE_RVA, 0xC0);
    if f2 & 0x10 != 0 {
        scaled(&mut g, 0x5F, 0x2A7, 0x1DE, &mut part)?;
    }
    if f1 & 0x40 != 0 {
        part(7, &mut g, &[])?;
    }
    set(QUAD_RVA + 0x14, 0xFFFF_FFFF);
    if f1 & 0x80 != 0 {
        part(8, &mut g, &[])?;
    }
    if f2 & 0x80 != 0 {
        part(9, &mut g, &[])?;
    }
    set(TEXTURE_RVA, 0xC1);
    if f1 & 1 != 0 {
        part(10, &mut g, &[])?;
    }
    if f2 & 1 != 0 {
        part(11, &mut g, &[])?;
    }
    for (flags, field, xs) in [
        (f1, 0x14, [0x43, 0x57, 0x6B]),
        (f2, 0x44, [0x1D4, 0x1E8, 0x1FC]),
    ] {
        if flags & 0x20 != 0 {
            unsafe {
                leading_time(&mut g, get(field), xs, 0x245, &mut |g, r, x, y| {
                    plain(g, r, x, y, &mut part)
                })
            }?;
        }
    }
    let left = [8, 0x16, 0x23, 0x2A, 0x38, 0x45, 0x4C, 0x5A];
    let right = [0x2B7, 0x2C5, 0x2D2, 0x2D9, 0x2E7, 0x2F4, 0x2FB, 0x309];
    for (times, y) in [([TIMES_RVA[0], TIMES_RVA[1]], 2u32), (CLOCKS_RVA, 0x11)] {
        if f1 & 0x40 != 0 && get(0x28) != 0xFFFF_FFFF {
            for (time, xs) in [(times[0], left), (times[1], right)] {
                let t: u32 = unsafe { game_read(time) };
                unsafe {
                    clock_panel(&mut g, t, xs, y, &mut |g, r, x, y| {
                        plain(g, r, x, y, &mut part)
                    })
                }?;
            }
        }
    }
    let laps = [
        (
            f1,
            0x1C,
            [0x55, 0x81],
            0u32,
            [3, 0x21, 4, 0x46],
            TIMES_RVA[2],
            [0x23, 0x37, 0x4B, 0x53, 0x67, 0x7B, 0x83, 0x97],
        ),
        (
            f2,
            0x4C,
            [0x1E6, 0x212],
            1,
            [0x194, 0x21, 0x195, 0x46],
            LAP_2_RVA,
            [0x1B4, 0x1C8, 0x1DC, 0x1E4, 0x1F8, 0x20C, 0x214, 0x228],
        ),
    ];
    for (flags, field, last, mark, sprites, time, xs) in laps {
        if flags & 2 == 0 {
            continue;
        }
        let v = get(field);
        if v != 0xFFFF_FFFF {
            let d = unsafe { time_digits(&mut g, v, 2) };
            sign(v, 0, 1);
            plain(&mut g, d[4], last[0], row(0x24), &mut part)?;
            plain(&mut g, d[5], last[1], row(0x24), &mut part)?;
        }
        plain(&mut g, mark, sprites[0], row(sprites[1]), &mut part)?;
        plain(&mut g, 0x4A, sprites[2], row(sprites[3]), &mut part)?;
        let t: u32 = unsafe { game_read(time) };
        unsafe {
            lap_panel(&mut g, t, xs, row(0x51), &mut |g, r, x, y| {
                plain(g, r, x, y, &mut part)
            })
        }?;
    }
    Ok(g)
}

/// In the game: the draws' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        *r = match place {
            1 => return draw_marker::game_part(place, r),
            2 => dials::set_up(r, &disc_1a::SETUP, &mut add_triangle::game_part_disc)?,
            3 => dials::set_up(r, &disc_1b::SETUP, &mut add_triangle::game_part_disc)?,
            4 => return draw_stage_bar::game_part(place, r),
            5 => draw_stage_bars_1::draw_stage_bars_1(r, &mut draw_stage_bar::game_part)?,
            6 => icons_1::icons_1(r, &mut icons_1::game_part)?,
            7 => draw_markers_1::draw_markers_1(r, &mut draw_markers_1::game_part)?,
            8 => draw_signs_1a::draw_signs_1a(r, &mut draw_signs::game_part)?,
            9 => draw_signs_1b::draw_signs_1b(r, &mut draw_signs::game_part)?,
            10 => dials::set_up(r, &needle_1a::SETUP, &mut draw_needle::game_part_needle)?,
            11 => dials::set_up(r, &needle_1b::SETUP, &mut draw_needle::game_part_needle)?,
            _ => return draw_markers_1::game_part(place, r, v),
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
    match unsafe { layout_1(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { layout_1(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn note_plain(place: u32, r: &mut Regs) {
    crate::hud::draw_markers_1::note(place, r, &[]);
}

/// The test of a split layout: a copy whose draws go to stand-ins (by `places`; the
/// scaled sprites' to the stand-in reading their scales, noted as place 1), against
/// `rust`, flags and times by lot.
pub fn test_layout(
    hook: &RawHook,
    address: usize,
    len: usize,
    calls: &[usize],
    places: &'static [(usize, u32)],
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    use crate::hud::draw_markers_1::{kept_stand_in, note, take};
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    set_handler(note_plain);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(address)),
            len,
            // mov g_hudQuad, 0E0h
            &[0xC7, 0x05, 0xB0, 0xBF, 0x75, 0x00, 0xE0, 0x00, 0x00, 0x00],
            calls,
            &|c| {
                if c + 0x40_0000 == 0x4B716A {
                    return Some(kept_stand_in as *const () as usize);
                }
                places
                    .iter()
                    .find(|&&(k, _)| rva(k) == c)
                    .map(|&(_, p)| stand_in(p))
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut regions = regions();
    regions.extend(
        [
            SCRATCH_RVA,
            QUAD_RVA + 4,
            QUAD_RVA + 8,
            QUAD_RVA + 12,
            QUAD_RVA + 16,
        ]
        .map(at),
    );
    let mut vars = regions.clone();
    vars.extend(TIMES_RVA.map(at));
    vars.extend(CLOCKS_RVA.map(at));
    vars.extend([LAP_2_RVA, UV_SCALE_RVA, crate::hud::draw_signs::FLAT_RVA].map(at));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..6u32).map(move |c| (w, c)))
    {
        let mut record = [0u32; 0x48];
        record[0xF8 / 4] = [0xFFu32, 0, 0x55, 0xAA, 0x62, 0x3D][case as usize];
        record[0xFC / 4] = [0xFFu32, 0x5A, 0, 0xA5, 0x3D, 0x62][case as usize];
        record[0x14 / 4] = [0u32, 1234, 59_999, 0xFFFF_FF00, 6_000, 361_234][case as usize];
        record[0x44 / 4] = 98_765 * case;
        record[0x18 / 4] = case;
        record[0x48 / 4] = case + 2;
        record[0x1C / 4] = if case == 2 { 0xFFFF_FFFF } else { 4321 * case };
        record[0x4C / 4] = if case == 3 { 0xFFFF_FFFF } else { 1111 * case };
        record[0x28 / 4] = [5u32, 0xFFFF_FFFF, 0xFFFF_FFF0, 30, 0x16, 3][case as usize];
        record[0x30 / 4] = [7u32, 9, 0xFFFF_FFFF, 0, 40, 2][case as usize];
        record[0xCC / 4] = case & 1;
        record[0xD0 / 4] = (case >> 1) & 1;
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            for (n, &r) in TIMES_RVA
                .iter()
                .chain(CLOCKS_RVA.iter())
                .chain([LAP_2_RVA].iter())
                .enumerate()
            {
                game_write(
                    r,
                    (54_321 * (case + 1) + 777 * n as u32)
                        ^ if n % 2 == 1 { 0x8000_0000 } else { 0 },
                );
            }
            game_write(UV_SCALE_RVA, 256.0f32.to_bits());
            game_write(crate::hud::draw_signs::FLAT_RVA, case & 1);
        };
        prepare();
        take();
        let mut ro = before;
        with_control_word(word, &mut || unsafe {
            crate::framework::call_game(&mut ro, copy.addr())
        });
        let theirs = (Some(regs(&ro)), take(), capture(&regions));
        prepare();
        let mine = rust(&before, &mut |p, r, v| {
            note(if p == SCALED { 1 } else { p }, r, v);
            Ok(())
        });
        let mine = (mine.as_ref().ok().map(regs), take(), capture(&regions));
        t.check(theirs == mine, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!("control word {word:#x}, case {case}: original {:x?} {} calls; rust {:x?} {} calls; first differing call {:?}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), first, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    const CALLS: [usize; 82] = [
        0x20, 0x3B, 0x9E, 0xB1, 0xD6, 0xE7, 0xF6, 0x16D, 0x1C5, 0x22A, 0x282, 0x320, 0x38B, 0x429,
        0x494, 0x4B1, 0x4CC, 0x4DD, 0x4F8, 0x509, 0x5E3, 0x618, 0x63F, 0x719, 0x74E, 0x775, 0x846,
        0x863, 0x880, 0x89D, 0x8BA, 0x8D7, 0x8F4, 0x911, 0x9C8, 0x9E5, 0xA02, 0xA1F, 0xA3C, 0xA59,
        0xA76, 0xA93, 0xB64, 0xB81, 0xB9E, 0xBBB, 0xBD8, 0xBF5, 0xC12, 0xC2F, 0xCE6, 0xD03, 0xD20,
        0xD3D, 0xD5A, 0xD77, 0xD94, 0xDB1, 0xE89, 0xEB2, 0xEDB, 0xF04, 0xFC7, 0xFF0, 0x1019,
        0x1042, 0x106B, 0x1094, 0x10BD, 0x10E6, 0x11BE, 0x11E7, 0x1210, 0x1239, 0x12FC, 0x1325,
        0x134E, 0x1377, 0x13A0, 0x13C9, 0x13F2, 0x141B,
    ];
    test_layout(
        &HOOK,
        0x4B2743,
        0x1421,
        &CALLS,
        &[
            (0x4B70EE, 1),
            (0x4B6B5B, 2),
            (0x4B6BAC, 3),
            (0x4B7216, 4),
            (0x4B5E29, 5),
            (0x4B5839, 6),
            (0x4B56E3, 7),
            (0x4B52F7, 8),
            (0x4B5451, 9),
            (0x4B6E67, 10),
            (0x4B6E9A, 11),
        ],
        &|r, p| unsafe { layout_1(r, p) },
    )
}
