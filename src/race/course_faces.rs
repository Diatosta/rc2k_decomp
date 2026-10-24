//! The course's faces drawn (`sub_44F9F3` -> `sub_4537E3` -> `sub_45232A`, part R1, IDA
//! 2026-10-18). A face record is 0x40 bytes at `g_courseFaces` (+0 the kind and flags, +1
//! the object, +4 .. +0x10 four scene points, +0x18 / +0x30 / +0x38 and +0x1C / +0x34 /
//! +0x3C its two triangles' normals, by the reading of `sub_4525F8`, +0x20 / +0x21 the
//! second's flags and object). Here its leaves.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

fn g(a: u32) -> u32 {
    game_addr(rva(a as usize)) as u32
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// `race::face_depth_test_off` (0x452358): with no texture animation (`dword_63DA2C` 0)
/// and `g_polygonMode` 4, the depth test off and `dword_63D9E0` 1.
pub unsafe fn face_depth_test_off() {
    unsafe {
        // Converted: `rc2k_core::race::faces::depth_test_off`.
        if !rc2k_core::race::faces::depth_test_off(
            game_read(rva(0x63DA2C)),
            game_read(rva(0x63DAB4)),
        ) {
            return;
        }
        crate::scene::depth_test_off::depth_test_off();
        game_write(rva(0x63D9E0), 1u32);
    }
}

/// `race::face_depth_test_on` (0x45237A): the depth test on unless `dword_63D9E0` is set,
/// then `dword_63D9E0` 0. (It turns the test on when it was not turned off: the test looks
/// the wrong way round, `notes/bugs.md`.)
pub unsafe fn face_depth_test_on() {
    unsafe {
        // Converted: `rc2k_core::race::faces::depth_test_on`.
        if rc2k_core::race::faces::depth_test_on(game_read(rva(0x63D9E0))) {
            crate::scene::depth_test_on::depth_test_on();
        }
        game_write(rva(0x63D9E0), 0u32);
    }
}

/// `race::face_corners` (0x452444): face esi's four points into `dword_63D9EC` ..
/// `dword_63D9F8`, the fourth -1 when it is the third (a triangle). Leaves eax the fourth.
pub unsafe fn face_corners(face: u32) -> u32 {
    // Converted: `rc2k_core::race::faces::corners`.
    unsafe {
        let points = [4, 8, 0xC, 0x10].map(|o| get(face + o));
        let corners = rc2k_core::race::faces::corners(points);
        game_write(rva(0x63D9EC), corners);
        corners[3]
    }
}

/// `sub_4525F8`: 1 when the face's triangle at `face + half` (half 0 or 4) faces the view
/// (its point +8 less the view's place, `flt_6062A4` .., dotted with its normal +0x18,
/// +0x30, +0x38 above 0, or not a number; the original's x87 run), 0 outside a race.
unsafe fn face_turned_to_view(face: u32, half: u32) -> u32 {
    unsafe {
        if game_read::<u32>(rva(0x621480)) == 0 {
            return 0;
        }
        let e = face + half;
        let p = get(e + 8)
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)));
        let b = game_addr(0) as u32;
        let status: u16;
        core::arch::asm!(
            "fild dword ptr [{p}]",
            "fsub dword ptr [{b} + 0x2062A4]",
            "fld dword ptr [{e} + 0x18]",
            ".byte 0xDE, 0xC9",
            "fild dword ptr [{p} + 4]",
            "fsub dword ptr [{b} + 0x2062A8]",
            "fld dword ptr [{e} + 0x30]",
            ".byte 0xDE, 0xC9",
            "fild dword ptr [{p} + 8]",
            "fsub dword ptr [{b} + 0x2062AC]",
            "fld dword ptr [{e} + 0x38]",
            ".byte 0xDE, 0xC9",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xEE",
            ".byte 0xDE, 0xD9",
            "fnstsw ax",
            p = in(reg) p,
            e = in(reg) e,
            b = in(reg) b,
            out("ax") status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        u32::from(status & 0x0100 != 0)
    }
}

/// `race::face_sides` (0x4525BE): which of face `face`'s (esi) triangles face the view:
/// `dword_63D9E4` the first's (`sub_4525F8`), `dword_63D9E8` 1, or the second's when it
/// has one (`dword_63D9F8` not -1). The registers the original leaves (eax the last
/// answer, ebx the last triangle's point, ebp the face) are read by no one: its one
/// caller's parts take only esi, the face.
pub unsafe fn face_sides(face: u32) {
    unsafe {
        game_write(rva(0x63D9E4), face_turned_to_view(face, 0));
        game_write(rva(0x63D9E8), 1u32);
        if game_read::<u32>(rva(0x63D9F8)) != u32::MAX {
            game_write(rva(0x63D9E8), face_turned_to_view(face, 4));
        }
    }
}

/// `fild t; fmul f; fmul a; fmul b; fistp`.
fn frame_of(t: u32, f: u32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([t, f, a, b], 0u32);
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]",
            "fmul dword ptr [{v} + 4]",
            "fmul dword ptr [{v} + 8]",
            "fmul dword ptr [{v} + 12]",
            "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `race::face_materials` (0x45265C): face `face`'s two triangles' textures: for each, its
/// object (+1, +0x21) animated when `byte_609C4C` gives it a speed (car 0's +8 clock times
/// `g_faceKeyFactors[speed]`, `flt_5C5D54`, `flt_5C5DA4`, rounded, & 7 picks the frame of
/// `byte_609384`; the first's frame also to `dword_63DA08`), into `dword_63DA2C` /
/// `dword_63DA38`; the kind's low three bits and bit 3 (+0, +0x20) into `dword_63DA30` /
/// `dword_63DA34` and `dword_63DA3C` / `dword_63DA40`. The registers it leaves are read
/// by no one.
pub unsafe fn face_materials(face: u32) {
    unsafe {
        let byte = |a: u32| (a as *const u8).read() as u32;
        let clock: u32 = game_read(rva(0x70F3F4));
        let (k1, k2) = (
            game_read::<u32>(rva(0x5C5D54)),
            game_read::<u32>(rva(0x5C5DA4)),
        );
        let texture = |object: u32, first: bool| -> u32 {
            let speed = byte(g(0x609C4C) + object);
            if speed == 0 {
                return object;
            }
            let n = frame_of(clock, get(g(0x5C6950) + 4 * speed), k1, k2);
            game_write(rva(0x62400C), n);
            let (frame, place) = rc2k_core::race::faces::frame_place(object, n);
            if first {
                game_write(rva(0x63DA08), frame);
            }
            byte(g(0x609384) + place)
        };
        // Converted: `rc2k_core::race::faces::{frame_place, material_bits}` (the frame's
        // rounding the x87's).
        use rc2k_core::race::faces::material_bits;
        game_write(rva(0x63DA2C), texture(byte(face + 1), true));
        let (order, back) = material_bits(byte(face) as u8);
        game_write(rva(0x63DA30), order);
        game_write(rva(0x63DA34), back);
        game_write(rva(0x63DA38), texture(byte(face + 0x21), false));
        let (order, back) = material_bits(byte(face + 0x20) as u8);
        game_write(rva(0x63DA3C), order);
        game_write(rva(0x63DA40), back);
    }
}

/// `race::draw_marked_face` (0x44CD90): `scene::scale_texture_coordinates`,
/// `scene::fog_polygon`, the drawing (`scene::draw_marked_polygon`), `g_polygonDrawn` 1.
pub unsafe fn draw_marked_face() -> Result<(), Refused> {
    unsafe {
        crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
        crate::scene::fog_polygon::fog_polygon()?;
        crate::scene::draw_marked_polygon::draw_marked_polygon(&mut || {
            crate::scene::draw_polygon::draw_polygon_in_game()
        })?;
        game_write(rva(0x624134), 1u32);
        Ok(())
    }
}

/// `race::face_switches` (0x45322C): `dword_6240E0` / `dword_6240E4` 1 when the face's
/// first / second texture is animated (`dword_63DA2C` / `dword_63DA38` not 0), else 0.
pub unsafe fn face_switches() {
    // Converted: `rc2k_core::race::faces::switches`.
    unsafe {
        let [first, second] =
            rc2k_core::race::faces::switches([game_read(rva(0x63DA2C)), game_read(rva(0x63DA38))]);
        game_write(rva(0x6240E0), first);
        game_write(rva(0x6240E4), second);
    }
}

/// `sub_453786`: the light at scene point `n` (`race::light_colour_at`, through
/// `dword_62400C`), 0 for -1.
unsafe fn point_light(n: u32) -> Result<u32, Refused> {
    unsafe {
        if n == u32::MAX {
            return Ok(0);
        }
        let p = n.wrapping_mul(12).wrapping_add(game_read(rva(0x5CCCC0)));
        game_write(rva(0x636588), get(p));
        game_write(rva(0x63658C), get(p + 4));
        game_write(rva(0x636590), get(p + 8));
        crate::race::frame_draw::light_colour_at()?;
        let light: u32 = game_read(rva(0x62DAC4));
        game_write(rva(0x62400C), light);
        Ok(light)
    }
}

/// `race::face_lights` (0x453715): the light at each of the face's four corners
/// (`dword_63DA54` .. `dword_63DA60`; 0 for a missing fourth), when lamps are wanted
/// (`g_lampsWanted`) and the face is not a plain mode-4 one; all 0 otherwise. Refuses
/// (before anything) what `light_colour_at` refuses.
pub unsafe fn face_lights() -> Result<(), Refused> {
    unsafe {
        // Converted: `rc2k_core::race::faces::corners_lit`.
        let wanted: u32 = game_read(rva(0x63DB58));
        let lit = wanted != 0
            && rc2k_core::race::faces::corners_lit(
                wanted,
                game_read(rva(0x63DA2C)),
                game_read(rva(0x63DAB4)),
            );
        if lit {
            crate::race::frame_draw::check_light_colour()?;
        }
        for k in 0..4 {
            game_write(rva(0x63DA54 + 4 * k), 0u32);
        }
        if !lit {
            return Ok(());
        }
        for k in 0..4 {
            let n: u32 = game_read(rva(0x63D9EC + 4 * k));
            let light = point_light(n)?;
            game_write(rva(0x63DA54 + 4 * k), light);
        }
        Ok(())
    }
}

/// `race::clip_and_draw_face` (0x454187): the face's polygon cut by depth (when some
/// vertices are outside, `g_polygonOutsideCount`) and to the screen, and with three
/// vertices or more drawn (`scene::draw_polygon_in_game`), `g_polygonDrawn` 1.
pub unsafe fn clip_and_draw_face() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::draws_polygon`.
    use rc2k_core::race::face_drawing::draws_polygon;
    unsafe {
        if game_read::<u32>(rva(0x6240F4)) != 0 {
            crate::scene::clip_polygon_by_depth::clip_polygon_by_depth()?;
            if !draws_polygon(game_read(rva(0x6240F8))) {
                return Ok(());
            }
        }
        crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
        if !draws_polygon(game_read(rva(0x6240F8))) {
            return Ok(());
        }
        crate::scene::draw_polygon::draw_polygon_in_game()?;
        game_write(rva(0x624134), 1u32);
        Ok(())
    }
}

/// `race::face_out_of_view` (0x45246F): true (the original's carry) when none of the
/// face's corners (`dword_63D9EC` ..; the fourth when there is one) is in sight: a corner
/// is in sight when its scene record's depth (+0x18) is not negative and |x| (+0x10) is
/// below `flt_63DA00` (|x| then left in `dword_62400C`) or |x| times `flt_63D9FC` is below
/// the depth. The records are made when not there (as `sub_44CF33`). The eax and edx the
/// original leaves are read by no one. Refuses (before anything) no point records or
/// points.
pub unsafe fn face_out_of_view() -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::face_drawing::corner_in_sight` (exact in `f64`).
    use rc2k_core::race::face_drawing::{InSight, corner_in_sight};
    unsafe {
        crate::race::frame_draw::points_there()?;
        let near: u32 = game_read(rva(0x63DA00));
        let slope: u32 = game_read(rva(0x63D9FC));
        for k in 0..4 {
            let n: u32 = game_read(rva(0x63D9EC + 4 * k));
            if k == 3 && n == u32::MAX {
                break;
            }
            let (record, _) = crate::race::frame_draw::point_record(n)?;
            let depth = get(record + 0x18);
            if depth & 0x8000_0000 != 0 {
                continue;
            }
            let x = get(record + 0x10);
            match corner_in_sight(depth, x, near, slope) {
                Some(InSight::Near) => {
                    game_write(rva(0x62400C), x & 0x7FFF_FFFF);
                    return Ok(false);
                }
                Some(InSight::Slope) => return Ok(false),
                None => {}
            }
        }
        Ok(true)
    }
}

/// Where one of the face's two triangles keeps its texture's working values
/// (`sub_453267` for the first, `sub_45349F` for the second: the same code on other
/// variables).
pub struct TextureSet {
    /// The texture (`dword_63DA2C` / `dword_63DA38`).
    pub texture: usize,
    /// The kind's low bits, the corner order (`dword_63DA30` / `dword_63DA3C`).
    pub order: usize,
    /// The texture's size word with bit 31 (`dword_6240BC` / `dword_6240C0`).
    pub size: usize,
    /// The texture's mapping mode, its +0x2D signed (`dword_623DD8` / `dword_623DE0`).
    pub mapping: usize,
    /// 1 when the mapping is 0 (`dword_63E168` / `dword_63E16C`).
    pub plain: usize,
    /// The four corners' u, v (`dword_63DA0C` .. / `dword_63DAB8` ..).
    pub uv: usize,
}

/// The first triangle's set.
pub const FIRST_SET: TextureSet = TextureSet {
    texture: 0x63DA2C,
    order: 0x63DA30,
    size: 0x6240BC,
    mapping: 0x623DD8,
    plain: 0x63E168,
    uv: 0x63DA0C,
};

/// The second triangle's set.
pub const SECOND_SET: TextureSet = TextureSet {
    texture: 0x63DA38,
    order: 0x63DA3C,
    size: 0x6240C0,
    mapping: 0x623DE0,
    plain: 0x63E16C,
    uv: 0x63DAB8,
};

/// `race::face_texture_coordinates` (0x453267) / `race::face_texture_coordinates_2`
/// (0x45349F): the triangle's texture's size word (+0x20, bit 31 set) and mapping (+0x2D);
/// the four corners' u, v from the texture's corner table for the order
/// (`dword_63DAD8[order * 4 + k]` points at a u16 pair inside the texture record, << 8,
/// doubled for sizes 0x190 .. 0x31F); with a mapping above 0 the corners' scene x and z
/// shifted by (mapping & 3) + 2 instead, less the first corner's top byte (the fourth
/// corner read even when it is -1: `notes/bugs.md`). The registers the original leaves
/// (eax the size word, ebx the texture record) are read by no one.
pub unsafe fn face_texture_coordinates(set: &TextureSet) {
    // Converted: `rc2k_core::race::faces::texture_coordinates`.
    use rc2k_core::race::faces::{TextureInfo, texture_coordinates};
    unsafe {
        let texture: u32 = game_read(rva(set.texture));
        let info = g(0x979F30).wrapping_add(texture << 6);
        let texture_info = TextureInfo {
            size: get(info + 0x20),
            mapping: ((info + 0x2D) as *const i8).read(),
        };
        let order: u32 = game_read(rva(set.order));
        let pairs = [0usize, 1, 2, 3].map(|k| {
            let table = game_read::<u32>(rva(0x63DAD8) + (order << 4) as usize + 4 * k);
            let pair = table.wrapping_add(info);
            (
                (pair as *const u16).read_unaligned(),
                ((pair + 2) as *const u16).read_unaligned(),
            )
        });
        let c = texture_coordinates(texture_info, pairs, |k| {
            let points: u32 = game_read(rva(0x5CCCC0));
            let n: u32 = game_read(rva(0x63D9EC) + 4 * k);
            let p = n.wrapping_mul(12).wrapping_add(points);
            (get(p), get(p + 8))
        });
        game_write(rva(set.size), c.size);
        game_write(rva(set.mapping), c.mapping);
        game_write(rva(set.plain), c.plain);
        game_write(rva(set.uv), c.uv);
    }
}

/// `race::face_corners_to_new_points` (0x452722): the face's corners copied to the four
/// scene points after the last (`g_scenePointCount` + 0 .. 3, their records marked unmade,
/// all four), the corners then naming the copies (the fourth only when there is one); the
/// count is left. Refuses (before anything) no point records or points.
pub unsafe fn face_corners_to_new_points() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::faces::new_points`.
    unsafe {
        crate::race::frame_draw::points_there()?;
        let count: u32 = game_read(rva(0x5CCD20));
        let records: u32 = game_read(rva(0x5C6FB8));
        let corners: [u32; 4] = game_read(rva(0x63D9EC));
        let (marked, copies) = rc2k_core::race::faces::new_points(count, corners);
        for n in marked {
            ((records + (n << 5)) as *mut u32).write_unaligned(u32::MAX);
        }
        let points: u32 = game_read(rva(0x5CCCC0));
        for (k, (from, to)) in copies.into_iter().enumerate() {
            let to_at = to.wrapping_mul(12).wrapping_add(points);
            let from_at = from.wrapping_mul(12).wrapping_add(points);
            for w in 0..3 {
                ((to_at + 4 * w) as *mut u32).write_unaligned(get(from_at + 4 * w));
            }
            game_write(rva(0x63D9EC) + 4 * k, to);
        }
        Ok(())
    }
}

const ENTRIES: u32 = 0x975510; // g_polygonEntries: 0x44 bytes each
const ENTRY_SIZE: u32 = 0x44;

fn entry(slot: u32) -> u32 {
    g(ENTRIES + ENTRY_SIZE * slot)
}

/// One polygon entry filled from one of the face's corners: +0 its point, +8 / +0xC the
/// first texture's u, v, +0x10 / +0x14 the second's, +0x18 its colour, +0x1C its light,
/// +0x21 its fog byte (`dword_63DA64` ..).
unsafe fn fill_entry(slot: u32, corner: u32) {
    unsafe {
        let e = entry(slot);
        let c = corner as usize;
        let w = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
        w(0, game_read(rva(0x63D9EC) + 4 * c));
        w(8, game_read(rva(0x63DA0C) + 8 * c));
        w(0xC, game_read(rva(0x63DA10) + 8 * c));
        w(0x18, game_read(rva(0x63DA44) + 4 * c));
        w(0x1C, game_read(rva(0x63DA54) + 4 * c));
        w(0x10, game_read(rva(0x63DAB8) + 8 * c));
        w(0x14, game_read(rva(0x63DABC) + 8 * c));
        ((e + 0x21) as *mut u8).write(game_read::<u32>(rva(0x63DA64) + 4 * c) as u8);
    }
}

/// The light's third byte (bits 16 .. 23) of entry `slot`: the lamps' part.
unsafe fn lamp_byte(slot: u32) -> u8 {
    unsafe { ((entry(slot) + 0x1E) as *const u8).read() }
}

/// `g_polygonList` 0 .. count - 1 and a closing 0, `g_polygonVertexCount` count,
/// `g_polygonNextEntry` `next`.
unsafe fn plain_list(count: u32, next: u32) {
    unsafe {
        game_write(rva(0x6240F8), count);
        for k in 0..count {
            game_write(rva(0x977710) + 4 * k as usize, k);
        }
        game_write(rva(0x977710) + 4 * count as usize, 0u32);
        game_write(rva(0x624114), next);
    }
}

/// `race::draw_whole_quad` (0x452F3D): the face drawn as one quad when it can be: the
/// entries 1, 2, 3, 0 from corners 0, 1, 3, 2; carry (draw it as triangles) when a corner
/// has lamp light or some vertex is outside after `scene::project_polygon` or off the
/// screen's inside (the screen edges `dword_6241D4` .. less one; each vertex's x and y
/// rounded into `dword_62400C`, compared by the sign of the difference); nothing when
/// all are outside; else the quad drawn (`race::draw_marked_face`). Gives the carry
/// (true: draw it as triangles).
pub unsafe fn draw_whole_quad() -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside,
    // screen_inside, inside}` (the roundings the x87's).
    use rc2k_core::race::face_drawing::{QUAD_ENTRIES, quad_by_outside, screen_inside};
    unsafe {
        for (slot, corner) in QUAD_ENTRIES {
            fill_entry(slot, corner);
        }
        if (0..4).any(|s| lamp_byte(s) != 0) {
            return Ok(true);
        }
        plain_list(4, 4);
        crate::scene::project_polygon::project_polygon()?;
        if let Some(triangles) = quad_by_outside(game_read(rva(0x6240F4))) {
            return Ok(triangles);
        }
        let [left, right, top, bottom] = screen_inside(
            game_read(rva(0x6241DC)),
            game_read(rva(0x6241E0)),
            game_read(rva(0x6241D4)),
            game_read(rva(0x6241D8)),
        );
        let inside = |v: u32, low: u32, high: u32| {
            let n = crate::math::x87::to_int(v) as u32;
            game_write(rva(0x62400C), n);
            rc2k_core::race::face_drawing::inside(n, low, high)
        };
        for slot in 0..4 {
            let v = get(entry(slot) + 4);
            if !inside(get(v + 0xC), left, right) || !inside(get(v + 0x1C), top, bottom) {
                return Ok(true);
            }
        }
        game_write(rva(0x6240F8), 4u32);
        draw_marked_face()?;
        Ok(false)
    }
}

/// `race::draw_face_triangles` (0x452CF3): the face's two triangles each drawn
/// (`race::draw_lit_triangle`) unless facing away: the first (corners 0, 1, 2) when
/// `dword_63D9E4` is 0, the second (3, 2, 1) when there is one and `dword_63D9E8` is 0.
pub unsafe fn draw_face_triangles() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{draws_first, draws_second,
    // FIRST_TRIANGLE, SECOND_TRIANGLE}`.
    use rc2k_core::race::face_drawing::{
        FIRST_TRIANGLE, SECOND_TRIANGLE, draws_first, draws_second,
    };
    unsafe {
        if draws_first(game_read(rva(0x63D9E4))) {
            for (slot, corner) in FIRST_TRIANGLE {
                fill_entry(slot, corner);
            }
            game_write(rva(0x6240F8), 3u32);
            draw_lit_triangle()?;
        }
        let fourth: u32 = game_read(rva(0x63D9F8));
        if fourth != u32::MAX && draws_second(fourth, game_read(rva(0x63D9E8))) {
            for (slot, corner) in SECOND_TRIANGLE {
                fill_entry(slot, corner);
            }
            game_write(rva(0x6240F8), 3u32);
            draw_lit_triangle()?;
        }
        Ok(())
    }
}

/// `race::draw_face_polygons` (0x452CCF): in a race with both triangles facing the view
/// (`dword_63D9E4` + `dword_63D9E8` 0) the face as one quad (`race::draw_whole_quad`, its
/// carry asking for triangles), else as triangles (`race::draw_face_triangles`).
pub unsafe fn draw_face_polygons() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::tries_quad`.
    unsafe {
        let in_race: u32 = game_read(rva(0x621480));
        if in_race != 0
            && rc2k_core::race::face_drawing::tries_quad(
                in_race,
                game_read(rva(0x63D9E4)),
                game_read(rva(0x63D9E8)),
            )
            && !draw_whole_quad()?
        {
            return Ok(());
        }
        draw_face_triangles()
    }
}

/// `race::draw_face` (0x452C6F): the face's textures (`race::face_switches`, the first's
/// coordinates when animated; `g_polygonMode` 2, or with a second texture the mode its
/// record's +0x2C picks in `dword_63D9D0` and the second's coordinates), the corners'
/// lights (`race::face_lights`), then the depth test off, the polygons
/// (`race::draw_face_polygons`), the depth test on. The original keeps esi (the face);
/// the rest it leaves no caller reads. Refuses (before anything) what `face_lights`
/// refuses.
pub unsafe fn draw_face() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{face_mode, PLAIN_MODE}`, the lights'
    // test `rc2k_core::race::faces::corners_lit`.
    use rc2k_core::race::face_drawing::{PLAIN_MODE, face_mode};
    unsafe {
        let second: u32 = game_read(rva(0x63DA38));
        let picked = || {
            let record = g(0x979F30).wrapping_add(second << 6);
            let pick = ((record + 0x2C) as *const u8).read() as usize;
            game_read::<u32>(rva(0x63D9D0) + 4 * pick)
        };
        let mode = face_mode(second, picked);
        let wanted: u32 = game_read(rva(0x63DB58));
        if wanted != 0
            && rc2k_core::race::faces::corners_lit(wanted, game_read(rva(0x63DA2C)), mode)
        {
            crate::race::frame_draw::check_light_colour()?;
        }
        face_switches();
        if game_read::<u32>(rva(0x63DA2C)) != 0 {
            face_texture_coordinates(&FIRST_SET);
        }
        game_write(rva(0x63DAB4), PLAIN_MODE);
        if second != 0 {
            game_write(rva(0x63DAB4), face_mode(second, picked));
            face_texture_coordinates(&SECOND_SET);
        }
        face_lights()?;
        face_depth_test_off();
        draw_face_polygons()?;
        face_depth_test_on();
        Ok(())
    }
}

/// `sub_4537E9`'s new vertex between two entries' vertices `a` and `b` into `v`: the
/// view floats (+0x10 .. +0x18) halfway (b - a) x `flt_5C5E40` + a, the screen place
/// (+0xC, +0x1C) 0, or when the depth is not below `flt_5C5E18` projected (by
/// `flt_624088` over the depth, or over `dword_5CCD80` + 1 when `dword_60932C` is set,
/// the depth then kept in `dword_62400C`; centre `flt_62409C`, `flt_6240A0`).
unsafe fn halfway_vertex(a: u32, b: u32, v: u32) {
    use crate::math::x87_load::below_or_unordered;
    unsafe {
        let base = game_addr(0) as u32;
        core::arch::asm!(
            "fld dword ptr [{b} + 0x10]",
            "fsub dword ptr [{a} + 0x10]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fadd dword ptr [{a} + 0x10]",
            "fstp dword ptr [{v} + 0x10]",
            "fld dword ptr [{b} + 0x14]",
            "fsub dword ptr [{a} + 0x14]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fadd dword ptr [{a} + 0x14]",
            "fstp dword ptr [{v} + 0x14]",
            "fld dword ptr [{b} + 0x18]",
            "fsub dword ptr [{a} + 0x18]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fadd dword ptr [{a} + 0x18]",
            "fstp dword ptr [{v} + 0x18]",
            a = in(reg) a,
            b = in(reg) b,
            v = in(reg) v,
            g = in(reg) base,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        ((v + 0xC) as *mut u32).write_unaligned(0);
        ((v + 0x1C) as *mut u32).write_unaligned(0);
        if below_or_unordered(get(v + 0x18), game_read(rva(0x5C5E18))) {
            return;
        }
        if game_read::<u32>(rva(0x60932C)) != 0 {
            core::arch::asm!(
                "fld dword ptr [{v} + 0x10]",
                "fld dword ptr [{v} + 0x14]",
                "fld dword ptr [{v} + 0x18]",
                "fstp dword ptr [{g} + 0x22400C]",
                "fild dword ptr [{g} + 0x1CCD80]",
                ".byte 0xD9, 0xE8",
                ".byte 0xDE, 0xC1",
                "fld dword ptr [{g} + 0x224088]",
                ".byte 0xDE, 0xF1",
                ".byte 0xDC, 0xC9",
                ".byte 0xDE, 0xCA",
                ".byte 0xD9, 0xE0",
                "fadd dword ptr [{g} + 0x2240A0]",
                ".byte 0xD9, 0xC9",
                "fadd dword ptr [{g} + 0x22409C]",
                "fstp dword ptr [{v} + 0xC]",
                "fstp dword ptr [{v} + 0x1C]",
                v = in(reg) v,
                g = in(reg) base,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        } else {
            core::arch::asm!(
                "fld dword ptr [{v} + 0x10]",
                "fld dword ptr [{v} + 0x14]",
                "fld dword ptr [{v} + 0x18]",
                "fld dword ptr [{g} + 0x224088]",
                ".byte 0xDE, 0xF1",
                ".byte 0xDC, 0xC9",
                ".byte 0xDE, 0xCA",
                ".byte 0xD9, 0xE0",
                "fadd dword ptr [{g} + 0x2240A0]",
                ".byte 0xD9, 0xC9",
                "fadd dword ptr [{g} + 0x22409C]",
                "fstp dword ptr [{v} + 0xC]",
                "fstp dword ptr [{v} + 0x1C]",
                v = in(reg) v,
                g = in(reg) base,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }
}

/// `sub_4537E9`'s new entry between entries `first` and `second` into `to`: the texture
/// floats (+8 .. +0x14) (second + first) x `flt_5C5E40`, the bytes +0x20, +0x21, +0x18 ..
/// +0x1A the halves of their sums (+0x1B left).
unsafe fn halfway_entry(first: u32, second: u32, to: u32) {
    unsafe {
        let base = game_addr(0) as u32;
        core::arch::asm!(
            "fld dword ptr [{s} + 8]",
            "fadd dword ptr [{f} + 8]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fstp dword ptr [{t} + 8]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{f} + 0xC]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fstp dword ptr [{t} + 0xC]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{f} + 0x10]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fstp dword ptr [{t} + 0x10]",
            "fld dword ptr [{s} + 0x14]",
            "fadd dword ptr [{f} + 0x14]",
            "fmul dword ptr [{g} + 0x1C5E40]",
            "fstp dword ptr [{t} + 0x14]",
            s = in(reg) second,
            f = in(reg) first,
            t = in(reg) to,
            g = in(reg) base,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
        for o in [0x20u32, 0x21, 0x18, 0x19, 0x1A] {
            let sum = ((second + o) as *const u8).read() as u32
                + ((first + o) as *const u8).read() as u32;
            ((to + o) as *mut u8).write((sum >> 1) as u8);
        }
    }
}

/// `race::draw_lit_triangle` (0x4537E9): the triangle in entries 0 .. 2. Without lamp
/// light at its corners drawn as it is (`scene::draw_clipped_polygon`). Else
/// (unless `scene::project_polygon` finds it all outside; `g_polygonDrawn` 0, the record
/// pool restarted, `scene::scale_texture_coordinates`, `scene::fog_polygon`) cut in four:
/// for each side (0-1, 1-2, 2-0) a new scene point halfway (vertices 0x623E40 ..; its
/// light by `race::light_colour_at`) and entry (3 .. 5); in mode 5 each entry's fog byte
/// less twice its lamp light (not below 0); each entry's colour plus its light
/// (`add_colours`); then the triangles (0, 3, 5), (1, 4, 3), (2, 5, 4), (3, 4, 5) each
/// clipped and drawn (`race::clip_and_draw_face`). Refuses (before anything) no points,
/// or what `light_colour_at` refuses.
pub unsafe fn draw_lit_triangle() -> Result<(), Refused> {
    use crate::race::frame_draw::add_colours;
    unsafe {
        if (0..3).all(|s| lamp_byte(s) == 0) {
            return crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game();
        }
        crate::race::frame_draw::points_there()?;
        crate::race::frame_draw::check_light_colour()?;
        game_write(rva(0x624134), 0u32);
        game_write(rva(0x6240A4), g(0x977F10));
        plain_list(game_read(rva(0x6240F8)), 6);
        crate::scene::project_polygon::project_polygon()?;
        if game_read::<u32>(rva(0x6240F4)) == game_read::<u32>(rva(0x6240F8)) {
            return Ok(());
        }
        crate::scene::scale_texture_coordinates::scale_texture_coordinates()?;
        crate::scene::fog_polygon::fog_polygon()?;
        for k in 0..3 {
            ((entry(3 + k) + 4) as *mut u32).write_unaligned(g(0x623E40 + 0x20 * k));
        }
        for (k, (first, second)) in [(0u32, 1u32), (1, 2), (2, 0)].into_iter().enumerate() {
            let v = g(0x623E40 + 0x20 * k as u32);
            let points: u32 = game_read(rva(0x5CCCC0));
            let pa = get(entry(first)).wrapping_mul(12).wrapping_add(points);
            let pb = get(entry(second)).wrapping_mul(12).wrapping_add(points);
            for (n, lit) in [0x636588u32, 0x63658C, 0x636590].into_iter().enumerate() {
                let o = 4 * n as u32;
                let (a, b) = (get(pa + o), get(pb + o));
                let m = ((b.wrapping_sub(a) as i32) >> 1) as u32;
                let m = m.wrapping_add(a);
                ((v + o) as *mut u32).write_unaligned(m);
                game_write(rva(lit as usize), m);
            }
            halfway_vertex(get(entry(first) + 4), get(entry(second) + 4), v);
            let to = entry(3 + k as u32);
            halfway_entry(entry(first), entry(second), to);
            crate::race::frame_draw::light_colour_at()?;
            let light: u32 = game_read(rva(0x62DAC4));
            game_write(rva(0x62400C), light);
            ((to + 0x1C) as *mut u32).write_unaligned(light);
        }
        let mode5 = game_read::<u32>(rva(0x63DAB4)) == 5;
        for slot in 0..6 {
            let e = entry(slot);
            if mode5 {
                let fog = ((e + 0x21) as *const u8).read() as i32 - 2 * lamp_byte(slot) as i32;
                ((e + 0x21) as *mut u8).write(fog.max(0) as u8);
            }
            ((e + 0x18) as *mut u32).write_unaligned(add_colours(get(e + 0x18), get(e + 0x1C)));
        }
        for list in [[0u32, 3, 5], [1, 4, 3], [2, 5, 4], [3, 4, 5]] {
            for (k, n) in list.iter().enumerate() {
                game_write(rva(0x977710) + 4 * k, *n);
            }
            game_write(rva(0x977710) + 12, list[0]);
            game_write(rva(0x6240F8), 3u32);
            game_write(rva(0x624114), 6u32);
            clip_and_draw_face()?;
        }
        Ok(())
    }
}

const LIGHT_TABLE: u32 = 0x635118; // g_lightTable: 256 colours

/// `race::face_colours` (0x452887): the corners' colours from `g_lightTable` by the face's
/// bytes +0x14 .. +0x17, with flag 4 of +2 plus `dword_635518[+0x28 ..]` and
/// `dword_635918[+0x2C ..]` (`add_colours`, MMX or not as the original); the fog bytes
/// +0x22 .. +0x25 into `dword_63DA64` ..; then the face drawn (`race::draw_face`).
pub unsafe fn face_colours(face: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{corner_colour, EXTRA_LIGHTS}`.
    use rc2k_core::race::lights::{EXTRA_LIGHTS, corner_colour};
    unsafe {
        let byte = |o: u32| ((face + o) as *const u8).read() as u32;
        for k in 0..4 {
            let c = get(g(LIGHT_TABLE) + 4 * byte(0x14 + k));
            game_write(rva(0x63DA44) + 4 * k as usize, c);
            let lit = ((face + 2) as *const u16).read_unaligned() & EXTRA_LIGHTS != 0;
            if lit {
                let extra = || {
                    let a = get(g(0x635518) + 4 * byte(0x28 + k));
                    let b = get(g(0x635918) + 4 * byte(0x2C + k));
                    (a, b)
                };
                let c = corner_colour(c, lit, extra, crate::race::frame_draw::has_mmx());
                game_write(rva(0x63DA44) + 4 * k as usize, c);
            }
        }
        for k in 0..4 {
            game_write(rva(0x63DA64) + 4 * k as usize, byte(0x22 + k));
        }
        draw_face()
    }
}

/// The corners turned over (0 with 1, and 2 with 3 when there is a fourth) and the first
/// texture's order by `byte_63DA7C` (its bit 3 kept): `sub_452B58`'s step, done twice.
unsafe fn turn_over() {
    unsafe {
        let swap = |a: usize, b: usize| {
            let (x, y): (u32, u32) = (game_read(rva(a)), game_read(rva(b)));
            game_write(rva(a), y);
            game_write(rva(b), x);
        };
        // Converted: `rc2k_core::race::face_drawing::{turns_last_pair, turned_order}`.
        use rc2k_core::race::face_drawing::{turned_order, turns_last_pair};
        swap(0x63D9EC, 0x63D9F0);
        if turns_last_pair(game_read(rva(0x63D9F8))) {
            swap(0x63D9F4, 0x63D9F8);
        }
        let order: u32 = game_read(rva(0x63DA30));
        let turned = turned_order(order, |n| ((g(0x63DA7C) + n) as *const u8).read());
        game_write(rva(0x63DA30), turned);
    }
}

/// `race::draw_back_face` (0x452B58): a two-sided face's back (when `dword_63DA34`, the
/// first kind's bit 3): the corners turned over, the colours from `g_lightTable` by
/// +0x28 .. +0x2B and the fog bytes +0x2C .. +0x2F, drawn (`race::draw_face`), the
/// corners turned back.
pub unsafe fn draw_back_face(face: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{has_back, turns_last_pair,
    // turned_order}`.
    unsafe {
        if !rc2k_core::race::face_drawing::has_back(game_read(rva(0x63DA34))) {
            return Ok(());
        }
        let byte = |o: u32| ((face + o) as *const u8).read() as u32;
        turn_over();
        for k in 0..4 {
            game_write(
                rva(0x63DA44) + 4 * k as usize,
                get(g(LIGHT_TABLE) + 4 * byte(0x28 + k)),
            );
        }
        for k in 0..4 {
            game_write(rva(0x63DA64) + 4 * k as usize, byte(0x2C + k));
        }
        draw_face()?;
        turn_over();
        Ok(())
    }
}

/// `race::draw_face_sides` (0x452393): outside a race both sides (`race::face_colours`,
/// `race::draw_back_face`) with neither facing away; in a race, a face with objects (+1
/// or +0x21): one-sided (kind bit 3 clear) its front unless both triangles face away;
/// two-sided its front unless both face away, then the facings flipped and its back
/// unless both (now) face away. Each after `race::face_materials`. `face` is esi.
pub unsafe fn draw_face_sides(face: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{has_objects, two_sided, side_drawn,
    // flipped}`.
    use rc2k_core::race::face_drawing::{flipped, has_objects, side_drawn, two_sided};
    unsafe {
        let byte = |o: u32| ((face + o) as *const u8).read() as u32;
        if game_read::<u32>(rva(0x621480)) == 0 {
            face_materials(face);
            game_write(rva(0x63D9E4), 0u32);
            game_write(rva(0x63D9E8), 0u32);
            face_colours(face)?;
            game_write(rva(0x63D9E4), 0u32);
            game_write(rva(0x63D9E8), 0u32);
            return draw_back_face(face);
        }
        let facings = || -> bool { side_drawn(game_read(rva(0x63D9E4)), game_read(rva(0x63D9E8))) };
        if !has_objects(byte(1) as u8, byte(0x21) as u8) {
            return Ok(());
        }
        if !two_sided(byte(0) as u8) {
            if !facings() {
                return Ok(());
            }
            face_materials(face);
            return face_colours(face);
        }
        face_materials(face);
        if facings() {
            face_colours(face)?;
        }
        game_write(rva(0x63D9E4), flipped(game_read(rva(0x63D9E4))));
        game_write(rva(0x63D9E8), flipped(game_read(rva(0x63D9E8))));
        if facings() {
            draw_back_face(face)?;
        }
        Ok(())
    }
}

/// `race::draw_course_face` (0x45232A): face `face` (esi): its corners
/// (`race::face_corners`; with flag 0x200 of +2 copied to new points,
/// `race::face_corners_to_new_points`), and unless none is in sight
/// (`race::face_out_of_view`) its facings (`race::face_sides`), the depth test off, its
/// sides (`race::draw_face_sides`), the depth test on. Refuses (before anything) no point
/// records or points.
pub unsafe fn draw_course_face(face: u32) -> Result<(), Refused> {
    unsafe {
        crate::race::frame_draw::points_there()?;
        face_corners(face);
        if ((face + 2) as *const u16).read_unaligned() & 0x200 != 0 {
            face_corners_to_new_points()?;
        }
        if face_out_of_view()? {
            return Ok(());
        }
        face_sides(face);
        face_depth_test_off();
        draw_face_sides(face)?;
        face_depth_test_on();
        Ok(())
    }
}

/// `race::draw_tinted_face` (0x44F9F3): face `face` (esi) drawn (`sub_4537E3`, that is
/// `race::draw_course_face`) with `g_polygonColour` times its object's tint
/// (`dword_60A880[+1]`, when not 0; byte by byte (a x b) >> 8, all four with MMX, the low
/// three without), the colour put back after. Of the registers its one caller
/// (`race::draw_list_entry`) gives, the drawing reads only esi.
pub unsafe fn draw_tinted_face(face: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::face_drawing::{tints, tinted}`.
    use rc2k_core::race::face_drawing::{tinted, tints};
    unsafe {
        let colour: u32 = game_read(rva(0x6240CC));
        let object = ((face + 1) as *const u8).read() as u32;
        let tint = get(g(0x60A880) + 4 * object);
        if tints(tint) {
            game_write(
                rva(0x6240CC),
                tinted(tint, colour, crate::race::frame_draw::has_mmx()),
            );
        }
        let done = draw_course_face(face);
        game_write(rva(0x6240CC), colour);
        done
    }
}
