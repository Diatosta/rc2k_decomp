//! `race::draw_viewer_panel` (0x41DA25): the replay viewer's panel: its texture
//! (`dword_5BECB8`) locked (`d3d::lock_texture`; failing, nothing), cleared, and written
//! on: text 0x1A4 and the time at the frame shown (block * 100 + frame, `strings::append_time`),
//! text 0x1A5 and the replay's length, the names of the block's two cameras
//! (`word_5C291C` by its bytes 0 and 1, at most 0x2E; `strings::append_string_by_id`),
//! each three times 0x78 to the left and 0xD down, texts 0x560 and 0x561 (the fonts at
//! `stru_51EAE4` ..; `text::draw_text_keep_regs`, `sub_402368`); unlocked
//! (`d3d::unlock_texture`), copied to its device texture (`dword_5BECBC`, `sub_41A8F9`),
//! made the texture (`d3d::set_texture`) and drawn as ten quads (`drive::tyre_quad`, the
//! texture's and the screen's rectangles `dword_5C2A1C`, `dword_5C2A0C`), the switch lights
//! dimmed (0x555555) by the block's camera byte 5 (0: the second, 1: the first).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_frame`. Its calls through `part` (`CALLEES`; `strings::append_nul`
//! written inline); in the game their Rust versions, the texture's copy (Direct3D) and the
//! texture's setting through the game. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, a surface of the
//! test's locked: camera bytes 5 of 0, 1 and 2, names in and past the table, the lock
//! failing.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{
    BLOCK, BLOCKS, CAMERA_NAMES, CAMERAS, FRAME, Part, Saved, answer, dispatch_parts, dword, gb,
    pb, region, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::draw_viewer_panel", rva(0x41DA25), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 10] = [
    0, 0x41AA4B, // 1 d3d::lock_texture (eax; carry)
    0x402368, // 2 sub_402368: text::draw_string_by_id in pusha / popa
    0x401A39, // 3 strings::append_time (eax, ebx -> ebx)
    0x402303, // 4 text::draw_text_keep_regs (eax font, ebx text, ecx, edx)
    0x401FA3, // 5 strings::append_string_by_id (eax, ebx -> ebx, eax)
    0x41AB02, // 6 d3d::unlock_texture
    0x41A8F9, // 7 sub_41A8F9: the texture copied to the device's (Direct3D)
    0x41ACF5, // 8 d3d::set_texture
    0x41DD3B, // 9 drive::tyre_quad
];
const CALLS: [usize; 34] = [
    0x6, 0x42, 0x5B, 0x60, 0x6F, 0x87, 0x9A, 0x9F, 0xAE, 0xE0, 0xE5, 0xF6, 0x101, 0x10C, 0x13A,
    0x13F, 0x153, 0x15E, 0x169, 0x180, 0x18D, 0x197, 0x1A7, 0x1B1, 0x22B, 0x23D, 0x27A, 0x28E,
    0x2CB, 0x2D7, 0x2E3, 0x2F7, 0x303, 0x30F,
];

const TEXTURE: usize = 0x5BECB8; // dword_5BECB8: the panel's texture
const DEVICE_TEXTURE: usize = 0x5BECBC; // dword_5BECBC
const TEXT: usize = 0x93E430; // text: the game's text buffer
const FONT_A: usize = 0x51EAE4; // stru_51EAE4
const FONT_B: usize = 0x51EB44; // stru_51EB44
const FONT_C: usize = 0x51EB04; // unk_51EB04
const FONT_D: usize = 0x51EB64; // stru_51EB64
const FONT_E: usize = 0x51EAC4; // unk_51EAC4
const UV: usize = 0x5C2A1C; // dword_5C2A1C: the texture's rectangle
const RECT: usize = 0x5C2A0C; // dword_5C2A0C: the screen's
const PIXELS: usize = 0x4F49C0; // g_surface.pixels
const HEIGHT: usize = 0x4F49A4; // g_surface.height
const ROW_DWORDS: usize = 0x4F49B4; // g_surface.row_dwords
const ROW_GAP: usize = 0x4F49C4; // g_surface.row_gap

/// The Rust version: the registers it leaves (the entry's).
pub fn draw_viewer_panel(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{shown_time, replay_length, camera_name,
    // switch_lights}`.
    use rc2k_core::race::viewer_flow::{camera_name, replay_length, shown_time, switch_lights};
    let mut r = *entry;
    r.eax = g(TEXTURE);
    part(1, &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    // The surface cleared (as `rep stosd` row by row; a height of 0 goes round 2^32 times
    // in the original as here: the texture has rows).
    r.eax = 0;
    r.edi = g(PIXELS);
    r.edx = g(HEIGHT);
    loop {
        let dwords = g(ROW_DWORDS);
        unsafe { std::ptr::write_bytes(r.edi as *mut u32, 0, dwords as usize) };
        r.edi = r.edi.wrapping_add(4 * dwords).wrapping_add(g(ROW_GAP));
        r.edx = r.edx.wrapping_sub(1);
        if r.edx == 0 {
            break;
        }
    }
    r.ecx = 0;
    let string = |part: Part, r: &mut Regs, id: u32| -> Result<(), Refused> {
        r.eax = id;
        part(2, r)
    };
    let nul = |r: &mut Regs| {
        unsafe { (r.ebx as *mut u8).write(0) };
        r.ebx = r.ebx.wrapping_add(1);
    };
    // The time shown, the replay's length.
    r.eax = 0x1A4;
    r.ebx = a(FONT_A);
    r.ecx = 4;
    r.edx = 0;
    string(part, &mut r, 0x1A4)?;
    for (hundredths, font) in [(shown_time(g(BLOCK), g(FRAME)), FONT_B), (0, FONT_D)] {
        if font == FONT_D {
            r.ebx = a(FONT_C);
            r.ecx = 0x60;
            r.edx = r.edx.wrapping_add(0x11);
            string(part, &mut r, 0x1A5)?;
            r.eax = replay_length(g(BLOCKS));
        } else {
            r.eax = hundredths;
        }
        r.ebx = a(TEXT);
        part(3, &mut r)?;
        nul(&mut r);
        r.eax = a(font);
        r.ebx = a(TEXT);
        r.edx = r.edx.wrapping_add(8);
        part(4, &mut r)?;
    }
    // The two cameras' names.
    r.edx = 0x32;
    for byte in 0..2usize {
        let camera = camera_name(gb((CAMERAS + byte).wrapping_add(8 * g(BLOCK) as usize)));
        r.eax = unsafe { (a(CAMERA_NAMES + 2 * camera as usize) as *const u16).read_unaligned() }
            as u32;
        r.ebx = a(TEXT);
        part(5, &mut r)?;
        nul(&mut r);
        r.eax = a(FONT_A);
        r.ebx = a(TEXT);
        r.ecx = 4;
        if byte == 1 {
            r.edx = r.edx.wrapping_add(0xD);
        }
        part(4, &mut r)?;
        for _ in 0..2 {
            r.edx = r.edx.wrapping_add(0xD);
            r.ecx = r.ecx.wrapping_sub(0x78);
            part(4, &mut r)?;
        }
    }
    r.ebx = a(FONT_E);
    r.ecx = 0x75;
    r.edx = 0;
    string(part, &mut r, 0x560)?;
    r.edx = r.edx.wrapping_add(0xF);
    string(part, &mut r, 0x561)?;
    r.eax = g(TEXTURE);
    part(6, &mut r)?;
    r.eax = g(TEXTURE);
    r.ebx = g(DEVICE_TEXTURE);
    part(7, &mut r)?;
    r.eax = g(DEVICE_TEXTURE);
    part(8, &mut r)?;
    // The quads.
    let camera_5 = gb((CAMERAS + 5).wrapping_add(8 * g(BLOCK) as usize));
    (r.esi, r.edi) = switch_lights(camera_5);
    r.edx = g(BLOCK);
    r.eax = a(UV);
    r.ebx = a(RECT);
    r.ecx = 0x00FF_FFFF;
    let set = |at: usize, v: [u32; 4]| {
        for (n, x) in v.into_iter().enumerate() {
            p(at + 4 * n, x);
        }
    };
    let add = |at: usize, v: u32| p(at, g(at).wrapping_add(v));
    set(UV, [0, 0, 0xC8, 0x32]);
    set(RECT, [4, 2, 0x64, 0x19]);
    part(9, &mut r)?;
    p(UV + 4, 0x32);
    p(RECT, 0x2B8);
    part(9, &mut r)?;
    set(UV, [0xDC, 0, 0x1A, 0x1A]);
    set(RECT, [1, 0x20, 0xD, 0xD]);
    r.ecx = r.esi;
    part(9, &mut r)?;
    p(UV + 4, 0x1E);
    p(RECT, 0x191);
    r.ecx = r.edi;
    part(9, &mut r)?;
    set(UV, [0, 0x64, 0x100, 0x1A]);
    set(RECT, [0x11, 0x20, 0x80, 0xD]);
    r.ecx = r.esi;
    part(9, &mut r)?;
    for _ in 0..2 {
        add(UV + 4, 0x1A);
        add(RECT, 0x78);
        part(9, &mut r)?;
    }
    p(UV + 4, 0xB2);
    p(RECT, 0x1A1);
    r.ecx = r.edi;
    part(9, &mut r)?;
    for _ in 0..2 {
        add(UV + 4, 0x1A);
        add(RECT, 0x78);
        part(9, &mut r)?;
    }
    Ok(*entry)
}

/// In the game: the Rust versions; the texture's copy and setting through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    unsafe {
        match place {
            1 => *r = crate::d3d::lock_texture::lock_texture(r)?,
            2 => {
                let out = crate::text::draw_string_by_id::draw_string_by_id_at(
                    r.eax,
                    r.ebx as *mut Font,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                r.set_carry(out.too_wide);
            }
            3 => {
                let time = crate::strings::append_time::append_time(
                    r.eax,
                    crate::strings::append_time::show_hours(),
                );
                std::ptr::copy_nonoverlapping(time.as_ptr(), r.ebx as *mut u8, time.len());
                r.ebx += time.len() as u32;
            }
            4 => {
                let font = r.eax as *mut Font;
                let mut copy = font.read_unaligned();
                let too_wide = crate::text::draw_text_keep_regs::draw_text_keep_regs(
                    &mut copy,
                    r.ebx as *const u8,
                    r.ecx as i32,
                    r.edx as i32,
                    &mut GameCanvas,
                )?;
                if copy != font.read_unaligned() {
                    font.write_unaligned(copy);
                }
                r.set_carry(too_wide);
            }
            5 => {
                let mut out = r.ebx as *mut u8;
                r.eax = crate::strings::append_string_by_id::append_string_by_id(
                    r.eax,
                    &mut |from, n| {
                        std::ptr::copy_nonoverlapping(from, out, n);
                        out = out.add(n);
                    },
                );
                r.ebx = out as u32;
            }
            6 => *r = crate::d3d::unlock_texture::unlock_texture(r)?,
            7 | 8 => call_game(r, game_addr(rva(CALLEES[place as usize]))),
            _ => {
                *r =
                    crate::drive::tyre_quad::tyre_quad(r, &mut crate::drive::tyre_parts::game_part)?
            }
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        draw_viewer_panel(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{TestSurface, has_format};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    let places: &'static [(usize, u32)] = Box::leak(
        (1..10u32)
            .map(|p| (CALLEES[p as usize], p))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // pusha; mov eax, dword_5BECB8
    let copy = match crate::race::viewer::copy(
        0x41DA25,
        0x316,
        &[0x60, 0xA1, 0xB8, 0xEC, 0x5B, 0x00],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut all = vec![
        region(TEXT, 0x40),
        region(UV, 0x10),
        region(RECT, 0x10),
        dword(BLOCK),
        dword(BLOCKS),
        dword(FRAME),
        region(CAMERAS, 0x40),
        region(CAMERA_NAMES, 0x60),
    ];
    let saved = Saved::new(&all);
    for (camera, names, fails) in [
        ([3u8, 0x2E], [0u8, 1, 2], false),
        ([0x40, 0], [1, 0, 0], false),
        ([5, 6], [2, 1, 0], false),
        ([5, 6], [0, 0, 0], true),
    ] {
        let surface = TestSurface::new(32);
        // The surface's rows as dwords, no gap.
        p(ROW_DWORDS, 64);
        p(ROW_GAP, 0);
        let pixels = (surface.pixels.as_ptr() as u32, surface.pixels.len());
        all.push(pixels);
        run_both(
            &mut t,
            &format!("cameras {camera:x?}, bytes 5 {names:?}, the lock failing {fails}"),
            copy.addr(),
            Regs::SENTINEL,
            &all,
            0,
            &|| {
                unsafe { std::ptr::write_bytes(pixels.0 as *mut u8, 0x5A, pixels.1) };
                p(BLOCK, 2);
                p(BLOCKS, 9);
                p(FRAME, 0x2A);
                crate::race::viewer::fill(CAMERAS, 0x40, 26);
                pb(CAMERAS + 16, camera[0]);
                pb(CAMERAS + 17, camera[1]);
                pb(CAMERAS + 21, names[0]);
                crate::race::viewer::fill(CAMERA_NAMES, 0x60, 27);
                crate::race::viewer::fill(TEXT, 0x40, 28);
                answer(Box::new(move |place, r| match place {
                    1 => r.set_carry(fails),
                    3 | 5 => {
                        r.ebx += 5;
                        r.eax = r.eax.wrapping_add(place);
                    }
                    _ => {}
                }));
            },
            &|r| draw_viewer_panel(r, &mut test_part),
        );
        all.pop();
        drop(surface);
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
