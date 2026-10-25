//! The skidmarks' drawing (`sub_4585E3` / `sub_458C03`, IDA 2026-10-18). A skidmark is a
//! chain of up to 0x400 links (`word_643AE8`, 8 bytes each: +0 the link before, -1 none;
//! +6 the views it shows in, a bit each) with two points each (`unk_645B18`, 0xC bytes a
//! point: x, y, z whole numbers); each link and the one before give a quad.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

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
    use crate::math::x87_float::{f80, int, ld};
    f80(int(i as i32) + ld(f))
}

/// `fild i; fstp tbyte`.
pub fn int_80(i: u32) -> [u8; 10] {
    crate::math::x87_float::f80(crate::math::x87_float::int(i as i32))
}

/// `fld tbyte; fstp dword`.
fn float_of_80(t: [u8; 10]) -> u32 {
    crate::math::x87_float::st(crate::math::x87_float::from_f80(t))
}

/// The projection of a turned vertex record (`v` +0x10 x, +0x14 y, +0x18 depth, floats)
/// onto the screen (+0xC, +0x1C): flat (`dword_60932C`) by `dword_5CCD80` + 1, else by the
/// depth (the original's `fdivrp st(1), st` is `dword_624088` over that). x is
/// x times that scale plus `dword_62409C`, y is `dword_6240A0` less y times it. When flat,
/// the depth is also stored at `dword_62400C`.
pub(crate) unsafe fn project_vertex(v: u32) {
    use crate::math::x87_float::{ONE, int, ld, st};
    unsafe {
        let (x, y, z) = (ld(get(v + 0x10)), ld(get(v + 0x14)), ld(get(v + 0x18)));
        let centre = |a: usize| ld(game_read::<u32>(rva(a)));
        let reach = centre(0x624088);
        let scale = if game_read::<u32>(rva(0x60932C)) != 0 {
            game_write(rva(0x62400C), st(z));
            reach / (int(game_read::<u32>(rva(0x5CCD80)) as i32) + ONE)
        } else {
            reach / z
        };
        put(v + 0xC, st(x * scale + centre(0x62409C)));
        put(v + 0x1C, st(-(y * scale) + centre(0x6240A0)));
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

/// `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view (all in modes
/// 0 and 2, else those with the view's bit, `dword_643AE4`): the polygon's four work
/// entries set once (vertices 0x623E40 .., texture places `dword_623DBC` / `dword_623DC0`,
/// bytes 0 and 0x60, textures 0xC0); then for each link with a link before, its two points
/// and the two before as a quad (vertices 3, 2, 0, 1), `scene::fog_polygon`, cut by depth
/// and to the screen and drawn in black in mode 3 without the depth test. Refuses what the scene's functions refuse (mid-way; not
/// met with four vertices).
pub unsafe fn draw_skidmarks() -> Result<(), Refused> {
    unsafe {
        let mut views = u32::MAX;
        let mode: u32 = game_read(rva(0x62389C));
        if mode != 0 && mode != 2 {
            views = 1u32.wrapping_shl(game_read::<u32>(rva(0x63DB9C)) & 0x1F);
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
            // Each quad inside the original's `pusha` .. `popa`; the depth test's switches
            // and the polygon's drawing read no register.
            if game_read::<u32>(rva(0x6240F8)) < 3 {
                continue;
            }
            crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
            if game_read::<u32>(rva(0x6240F8)) >= 3 {
                game_write(rva(0x6240CC), 0u32);
                crate::scene::depth_test_off::depth_test_off();
                game_write(rva(0x63DAB4), 3u32);
                game_write(rva(0x6240E0), 0u32);
                game_write(rva(0x6240E4), 1u32);
                crate::scene::draw_polygon::draw_polygon_in_game()?;
                game_write(rva(0x63DAB4), 0u32);
                game_write(rva(0x624134), 1u32);
                crate::scene::depth_test_on::depth_test_on();
            }
            game_write(rva(0x6240CC), 0x00FF_FFFFu32);
        }
        Ok(())
    }
}

/// `race::draw_skidmarks_if_on` (0x4585E3): `draw_skidmarks` unless `dword_6416B8` is set.
/// Its one caller, `race::draw_view_scene`, reads no register it leaves.
pub unsafe fn draw_skidmarks_if_on() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(rva(0x6416B8)) != 0 {
            return Ok(());
        }
        draw_skidmarks()
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::math::x87_float::{X, f80};
    use crate::race::asm_test::{Rng, Saved, at_each_control_word, lock};

    /// The original blocks, verbatim.
    fn int_plus_float_80_asm(i: u32, f: u32) -> [u8; 10] {
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

    fn int_80_asm(i: u32) -> [u8; 10] {
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

    fn float_of_80_asm(t: [u8; 10]) -> u32 {
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

    unsafe fn project_vertex_asm(v: u32, flat: bool) {
        unsafe {
            if flat {
                core::arch::asm!(
                    "fld dword ptr [{v} + 0x10]", "fld dword ptr [{v} + 0x14]", "fld dword ptr [{v} + 0x18]",
                    "fstp dword ptr [{rc} + 0x22400C]", "fild dword ptr [{rc} + 0x1CCD80]", "fld1",
                    ".byte 0xDE, 0xC1",
                    "fld dword ptr [{rc} + 0x224088]", ".byte 0xDE, 0xF1",
                    ".byte 0xDC, 0xC9", ".byte 0xDE, 0xCA", "fchs",
                    "fadd dword ptr [{rc} + 0x2240A0]", ".byte 0xD9, 0xC9", "fadd dword ptr [{rc} + 0x22409C]",
                    "fstp dword ptr [{v} + 0xC]", "fstp dword ptr [{v} + 0x1C]",
                    rc = in(reg) crate::race::store::asm_base(),
                    v = in(reg) v,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            } else {
                core::arch::asm!(
                    "fld dword ptr [{v} + 0x10]", "fld dword ptr [{v} + 0x14]", "fld dword ptr [{v} + 0x18]",
                    "fld dword ptr [{rc} + 0x224088]", ".byte 0xDE, 0xF1",
                    ".byte 0xDC, 0xC9", ".byte 0xDE, 0xCA", "fchs",
                    "fadd dword ptr [{rc} + 0x2240A0]", ".byte 0xD9, 0xC9", "fadd dword ptr [{rc} + 0x22409C]",
                    "fstp dword ptr [{v} + 0xC]", "fstp dword ptr [{v} + 0x1C]",
                    rc = in(reg) crate::race::store::asm_base(),
                    v = in(reg) v,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
        }
    }

    fn same_bits(a: u32, b: u32) -> bool {
        crate::race::asm_test::same(a, b)
    }

    #[test]
    fn conversions_match() {
        let _l = lock();
        let mut r = Rng(3);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let (i, f) = (r.int() as u32, r.float());
                assert_eq!(
                    int_plus_float_80(i, f),
                    int_plus_float_80_asm(i, f),
                    "{i} {f:#x}"
                );
                assert_eq!(int_80(i), int_80_asm(i), "{i}");
                let v = if r.below(2) == 0 {
                    f64::from(r.int())
                } else {
                    f64::from(f32::from_bits(r.float())) * f64::from(f32::from_bits(r.float()))
                };
                let t = f80(X(v));
                let (want, got) = (float_of_80_asm(t), float_of_80(t));
                assert!(same_bits(want, got), "{v}: {want:#x} {got:#x}");
            }
        });
    }

    #[test]
    fn project_vertex_matches() {
        let _l = lock();
        let _saved = Saved::new(&[0x62400C, 0x5CCD80, 0x624088, 0x6240A0, 0x62409C, 0x60932C]);
        let mut r = Rng(4);
        at_each_control_word(|| {
            for n in 0..20000 {
                let flat = n % 2 == 0;
                unsafe {
                    game_write(rva(0x60932C), flat as u32);
                    game_write(rva(0x5CCD80), r.int() as u32);
                    game_write(rva(0x624088), r.float());
                    game_write(rva(0x6240A0), r.float());
                    game_write(rva(0x62409C), r.float());
                    game_write(rva(0x62400C), 0xAAAA_AAAAu32);
                }
                let mut a = [0xAAAA_AAAAu32; 8];
                for q in a.iter_mut().skip(4).take(3) {
                    *q = r.float();
                }
                let mut b = a;
                unsafe { project_vertex_asm(a.as_mut_ptr() as u32, flat) };
                let kept = unsafe { game_read::<u32>(rva(0x62400C)) };
                unsafe { game_write(rva(0x62400C), 0xAAAA_AAAAu32) };
                unsafe { project_vertex(b.as_mut_ptr() as u32) };
                let now = unsafe { game_read::<u32>(rva(0x62400C)) };
                for k in 0..8 {
                    assert!(
                        same_bits(a[k], b[k]),
                        "flat {flat} word {k}: {:#x} {:#x} from {a:x?}",
                        a[k],
                        b[k]
                    );
                }
                assert!(same_bits(kept, now), "depth store {kept:#x} {now:#x}");
            }
        });
    }
}
