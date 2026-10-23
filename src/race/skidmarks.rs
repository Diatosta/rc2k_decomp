//! The skidmarks' drawing (`sub_4585E3` / `sub_458C03`, IDA 2026-10-18). A skidmark is a
//! chain of up to 0x400 links (`word_643AE8`, 8 bytes each: +0 the link before, -1 none;
//! +6 the views it shows in, a bit each) with two points each (`unk_645B18`, 0xC bytes a
//! point: x, y, z whole numbers); each link and the one before give a quad.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

fn g(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `fild i; fadd f; fstp tbyte`: a whole number plus a float in the x87's 80 bits.
fn int_plus_float_80(i: u32, f: u32) -> [u8; 10] {
    let (v, mut out) = ([i, f], [0u8; 10]);
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]",
            "fadd dword ptr [{v} + 4]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) out.as_mut_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `fild i; fstp tbyte`.
pub fn int_80(i: u32) -> [u8; 10] {
    let (v, mut out) = ([i], [0u8; 10]);
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) out.as_mut_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `fld tbyte; fstp dword`.
fn float_of_80(t: [u8; 10]) -> u32 {
    let mut out = 0u32;
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{t}]",
            "fstp dword ptr [{o}]",
            t = in(reg) t.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// The projection of a turned vertex record (`v` +0x10 x, +0x14 y, +0x18 depth, floats)
/// onto the screen (+0xC, +0x1C), as the original's run (`.byte 0xDE, 0xF1` its `fdivrp
/// st(1), st`): flat (`dword_60932C`) by `dword_5CCD80` + 1, else by the depth.
unsafe fn project_vertex(v: u32) {
    unsafe {
        let b = game_addr(0) as u32;
        if game_read::<u32>(rva(0x60932C)) != 0 {
            core::arch::asm!(
                "fld dword ptr [{v} + 0x10]", "fld dword ptr [{v} + 0x14]", "fld dword ptr [{v} + 0x18]",
                "fstp dword ptr [{b} + 0x22400C]", "fild dword ptr [{b} + 0x1CCD80]", "fld1",
                ".byte 0xDE, 0xC1",
                "fld dword ptr [{b} + 0x224088]", ".byte 0xDE, 0xF1",
                ".byte 0xDC, 0xC9", ".byte 0xDE, 0xCA", "fchs",
                "fadd dword ptr [{b} + 0x2240A0]", ".byte 0xD9, 0xC9", "fadd dword ptr [{b} + 0x22409C]",
                "fstp dword ptr [{v} + 0xC]", "fstp dword ptr [{v} + 0x1C]",
                v = in(reg) v,
                b = in(reg) b,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        } else {
            core::arch::asm!(
                "fld dword ptr [{v} + 0x10]", "fld dword ptr [{v} + 0x14]", "fld dword ptr [{v} + 0x18]",
                "fld dword ptr [{b} + 0x224088]", ".byte 0xDE, 0xF1",
                ".byte 0xDC, 0xC9", ".byte 0xDE, 0xCA", "fchs",
                "fadd dword ptr [{b} + 0x2240A0]", ".byte 0xD9, 0xC9", "fadd dword ptr [{b} + 0x22409C]",
                "fstp dword ptr [{v} + 0xC]", "fstp dword ptr [{v} + 0x1C]",
                v = in(reg) v,
                b = in(reg) b,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }
}

/// One skidmark point (`unk_645B18` + 0xC x n) into the vertex record `v`: raised by
/// `flt_643AD4`, turned by `g_viewMatrix` (`scene::rotate_point`), +0xC / +0x1C 0, and
/// projected when its depth is not below 4096.0.
unsafe fn skid_vertex(n: u32, v: u32) {
    unsafe {
        let p = g(0x645B18).wrapping_add(n.wrapping_mul(0xC));
        let lift: u32 = game_read(rva(0x643AD4));
        let point = [
            int_80(get(p)),
            int_plus_float_80(get(p + 4), lift),
            int_80(get(p + 8)),
        ];
        let t = crate::scene::rotate_point::rotate_point(g(0x5EA2CC), point);
        put(v + 0x10, float_of_80(t[0]));
        put(v + 0x14, float_of_80(t[1]));
        put(v + 0x18, float_of_80(t[2]));
        put(v + 0xC, 0);
        put(v + 0x1C, 0);
        if !crate::math::x87_load::below_or_unordered(get(v + 0x18), game_read(rva(0x5C5E18))) {
            project_vertex(v);
        }
    }
}

/// Whether a quad of `draw_skidmarks`' last run was cut by the screen clip (its registers
/// after are not modelled; the test then compares the calls by place). Its test clears it.
pub static SKIDS_CLIPPED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view (all in modes
/// 0 and 2, else those with the view's bit, `dword_643AE4`): the polygon's four work
/// entries set once (vertices 0x623E40 .., texture places `dword_623DBC` / `dword_623DC0`,
/// bytes 0 and 0x60, textures 0xC0); then for each link with a link before, its two points
/// and the two before as a quad (vertices 3, 2, 0, 1), `scene::fog_polygon`, cut by depth
/// and to the screen and drawn in black in mode 3 without the depth test (parts 1 .. 3).
/// Gives the registers it leaves. Refuses what the scene's functions refuse (mid-way; not
/// met with four vertices).
pub unsafe fn draw_skidmarks(
    entry: &Regs,
    part: crate::race::frame::Part,
) -> Result<Regs, Refused> {
    unsafe {
        let mut r = *entry;
        let mut views = u32::MAX;
        let mode: u32 = game_read(rva(0x62389C));
        if mode != 0 && mode != 2 {
            r.ecx = game_read(rva(0x63DB9C));
            r.eax = 1;
            views = 1u32.wrapping_shl(r.ecx & 0x1F);
        }
        game_write(rva(0x643AE4), views);
        let entries = g(0x975510);
        let vertices = g(0x623E40);
        for k in 0..4u32 {
            let e = entries + 0x44 * k;
            put(e + 4, vertices + 0x20 * k);
            put(e + 0x18, 0);
            put(e + 0x1C, 0);
        }
        r.esi = vertices + 0x60;
        let (u, w) = (
            game_read::<u32>(rva(0x623DBC)),
            game_read::<u32>(rva(0x623DC0)),
        );
        for k in 0..4u32 {
            let e = entries + 0x44 * k;
            put(e + 8, u);
            put(e + 0x10, u);
            put(e + 0xC, w);
            put(e + 0x14, w);
            put(e, u32::MAX);
            ((e + 0x21) as *mut u8).write(0x60);
            ((e + 0x20) as *mut u8).write(0);
        }
        r.eax = w & !0xFF;
        game_write(rva(0x6240BC), 0xC0u32);
        game_write(rva(0x6240C0), 0xC0u32);
        let links = g(0x643AE8);
        for n in 0..0x400u32 {
            let before = ((links + 8 * n) as *const i16).read_unaligned() as i32 as u32;
            if before == u32::MAX {
                continue;
            }
            let shown = ((links.wrapping_add(before.wrapping_mul(8)) + 6) as *const u16)
                .read_unaligned() as u32;
            if shown & views == 0 {
                continue;
            }
            skid_vertex(2 * n, vertices + 0x60);
            skid_vertex(2 * n + 1, vertices + 0x40);
            skid_vertex(before.wrapping_mul(2), vertices);
            skid_vertex(before.wrapping_mul(2).wrapping_add(1), vertices + 0x20);
            game_write(rva(0x6240F8), 4u32);
            game_write(rva(0x6240A4), g(0x977F10));
            let list = g(0x977710);
            for k in 0..4u32 {
                put(list + 4 * k, k);
            }
            put(list + 16, 0);
            game_write(rva(0x624114), 4u32);
            crate::scene::fog_polygon::fog_polygon()?;
            crate::scene::clip_polygon_by_depth::clip_polygon_by_depth()?;
            // Each quad inside the original's `pusha` .. `popa`: edx the link, ebx the count
            // the depth clip leaves, and for a quad wholly on the screen the screen clip's
            // registers (`unclipped_regs`; a cut one's are not modelled: SKIDS_CLIPPED).
            let mut inner = Regs {
                edx: n,
                ebx: game_read(rva(0x6240F8)),
                ..r
            };
            if game_read::<u32>(rva(0x6240F8)) < 3 {
                continue;
            }
            let unclipped = crate::scene::clip_polygon_to_screen::wholly_on_screen();
            if !unclipped {
                SKIDS_CLIPPED.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
            if unclipped {
                crate::scene::clip_polygon_to_screen::unclipped_regs(&mut inner);
            }
            if game_read::<u32>(rva(0x6240F8)) >= 3 {
                game_write(rva(0x6240CC), 0u32);
                part(1, &mut inner)?;
                game_write(rva(0x63DAB4), 3u32);
                game_write(rva(0x6240E0), 0u32);
                game_write(rva(0x6240E4), 1u32);
                part(2, &mut inner)?;
                game_write(rva(0x63DAB4), 0u32);
                game_write(rva(0x624134), 1u32);
                part(3, &mut inner)?;
            }
            game_write(rva(0x6240CC), 0x00FF_FFFFu32);
        }
        r.edx = 0x400;
        Ok(r)
    }
}

/// `draw_skidmarks`' parts in the game: the Rust versions.
pub fn skid_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::depth_test_off::depth_test_off(r),
            2 => crate::scene::draw_polygon::draw_polygon_in_game(r)?,
            _ => crate::scene::depth_test_on::depth_test_on(r),
        }
    }
    Ok(())
}

/// `race::draw_skidmarks_if_on` (0x4585E3): `draw` (`draw_skidmarks` in the game) unless
/// `dword_6416B8` is set.
pub unsafe fn draw_skidmarks_if_on(
    entry: &Regs,
    draw: &mut dyn FnMut(&Regs) -> Result<Regs, Refused>,
) -> Result<Regs, Refused> {
    unsafe {
        if game_read::<u32>(rva(0x6416B8)) != 0 {
            return Ok(*entry);
        }
        draw(entry)
    }
}
