//! The race frame's views and small drawing steps under `sub_45429B` (part R3, IDA
//! 2026-10-18): the full-screen and mirror windows, the flat triangle, the marks'
//! overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::input::store::DeviceSlot;
use crate::session::SCREEN;
use crate::store::{game_addr, game_read, game_write};

fn k(a: u32) -> u32 {
    unsafe { game_read(rva(a as usize)) }
}

fn w(a: u32, v: u32) {
    unsafe { game_write(rva(a as usize), v) }
}

/// `race::full_view_window` (0x455ECB) / `race::full_view_window_2` (0x456004): the view
/// window the whole screen (`dword_62408C` / `dword_624090` its size, `dword_623FB0` /
/// `dword_623FB4` 0, `dword_624080` 0x100, `scene::set_view_window`), `dword_63993C` 0; the
/// first also `dword_6365C4` 0. Its callers (`race::draw_race_scene`,
/// `race::draw_mirror_view`) read no register it leaves.
pub unsafe fn full_view_window(first: bool) {
    unsafe {
        w(0x62408C, SCREEN.width.get());
        w(0x624090, SCREEN.height.get());
        w(0x623FB0, 0);
        w(0x623FB4, 0);
        w(0x624080, 0x100);
        crate::scene::set_view_window::set_view_window();
        if first {
            w(0x6365C4, 0);
        }
        w(0x63993C, 0);
    }
}

/// `race::mirror_view_window` (0x455F17): the rear-view mirror's window: its corner from
/// `dword_6415FC` / `dword_641600` (8 bytes each, at `dword_6238A4` + 3 x `g_drawnView`, + 6
/// when `dword_6D1502` is 0), 800 x 600 scaled to the screen, 0xB3 x 0x40 big (each even);
/// its edges `dword_624138` .. `dword_624144`; `dword_624080` 0x100,
/// `scene::set_view_window`, the fog (`race::set_up_fog`), `dword_6365C4` 0,
/// `dword_63993C` 1. Refuses (before anything) a size whose scaling would fault.
pub unsafe fn mirror_view_window() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{Window, mirror_entry, mirror_window};
    unsafe {
        let n = mirror_entry(
            k(0x6238A4),
            k(0x63DB9C),
            ((crate::store::game_addr(rva(0x6D1502))) as *const u32).read_unaligned() == 0,
        );
        let at = |base: u32| k(base.wrapping_add(n.wrapping_mul(8)));
        let Some(Window {
            x,
            y,
            width: sx,
            height: sy,
        }) = mirror_window(
            (at(0x6415FC), at(0x641600)),
            (SCREEN.width.get(), SCREEN.height.get()),
        )
        else {
            return Err(BadInput("a mirror place whose scaling overflows").into());
        };
        w(0x623FB0, x);
        w(0x623FB4, y);
        w(0x62408C, sx);
        w(0x624090, sy);
        w(0x624138, x);
        w(0x62413C, y);
        w(0x624140, x.wrapping_add(sx));
        w(0x624144, y.wrapping_add(sy));
        w(0x624080, 0x100);
        crate::scene::set_view_window::set_view_window();
        crate::race::frame_draw::set_up_fog()?;
        w(0x6365C4, 0);
        w(0x63993C, 1);
        Ok(())
    }
}

/// The vertices' floats `race::draw_flat_triangle` sets: 1.0 at these, 0.0 at the rest of
/// its list (`g_polygonVertices` 0x9605A0, 0x20 bytes a vertex).
const ONES: [u32; 6] = [0x9605A0, 0x9605C0, 0x9605E0, 0x9605A4, 0x9605C4, 0x9605E4];
const ZEROS: [u32; 15] = [
    0x9605A8, 0x9605C8, 0x9605E8, 0x9605B8, 0x9605D8, 0x9605F8, 0x9605BC, 0x9605DC, 0x9605FC,
    0x9605B0, 0x9605D0, 0x9605F0, 0x9605B4, 0x9605D4, 0x9605F4,
];

/// `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0) with texture 0, 0,
/// colour 0 (the original's `fld1` / `fldz` stores), `g_polygonVertexCount` 3, drawn
/// (`scene::draw_fan`). Every register kept (`pusha` .. `popa`).
pub unsafe fn draw_flat_triangle() {
    for a in ONES {
        w(a, 1.0f32.to_bits());
    }
    for a in ZEROS {
        w(a, 0);
    }
    w(0x6240F8, 3);
    crate::scene::draw_fan::draw_fan();
}

/// The marks' shadow refusals (`marks::draw_shadow`'s first and second shadows), checked
/// before anything.
fn shadow_refuses(sh: &crate::marks::draw_shadow::Shadow) -> Result<(), Refused> {
    let corner = unsafe { crate::marks::store::game_read::<u32>(sh.corner()) };
    if corner == u32::MAX {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    if unsafe { crate::marks::store::game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0 {
        return Err(BadInput("no marks settings").into());
    }
    match sh.angle_refuses() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// `race::draw_marks_overlays` (0x45632B): `marks::draw_overlays`, then
/// `marks::draw_shadow`. Refuses (before anything) the second shadow's refusals.
pub unsafe fn draw_marks_overlays() -> Result<(), Refused> {
    shadow_refuses(&crate::marks::draw_shadow::FIRST)?;
    // Neither reads a register, and the race reads none they leave (only DF, ST).
    unsafe {
        crate::marks::draw_overlays::draw_overlays()?;
        crate::marks::draw_shadow::draw_shadow()?;
    }
    Ok(())
}

/// `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car`. (Its callers,
/// `race::draw_race_scene` and `drive::finish_run`, read no register it leaves.) Refuses
/// (before anything) car 0 without its block.
pub unsafe fn load_first_car() -> Result<(), Refused> {
    unsafe {
        if k(0x71B3B8) == 0 {
            return Err(BadInput("car 0 without its block").into());
        }
        w(0x71B3F8, 0);
        crate::drive::load_car::load_car()?;
        Ok(())
    }
}

const SURFACE_PIXELS: u32 = 0x4F49C0; // g_surface.pixels
const SURFACE_PITCH: u32 = 0x4F49BC; // g_surface.pitch
const SURFACE_WIDTH: u32 = 0x4F49A0; // g_surface.width
const SURFACE_HEIGHT: u32 = 0x4F49A4; // g_surface.height
const SURFACE_BITS: u32 = 0x4F49A8; // g_surface.bits_per_pixel
const SURFACE_FORMAT: u32 = 0x4F49B8; // g_surface.format (a pointer)

/// `race::copy_mirror_picture` (0x455DAD): the mirror's surface (slot `dword_624234`) copied
/// onto slot `dword_5D0E08`'s, each pixel through `gfx::pixel_to_components` (the screen's
/// format, 16 or 32 bits by `g_screenBits`) and back by `gfx::components_to_pixel` (the
/// destination's) with alpha 0xFF; the destination's width and height; each surface locked
/// and unlocked by `d3d::lock_texture` / `d3d::unlock_texture` (carry: the lock failed).
/// The registers it leaves are read by no caller. Refuses (before the copy) a null
/// destination format.
pub unsafe fn copy_mirror_picture() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{copy_picture, copied_any}` (eax carried
    // from pixel to pixel as the original's; the formats' conversions the gfx module's).
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    use crate::gfx::pixel_format::PixelFormat;
    use rc2k_core::race::view_steps::{PictureCopy, copy_picture};
    unsafe {
        let (from, to) = (k(0x624234), k(0x5D0E08));
        if lock_texture(from)?.carry {
            return Ok(());
        }
        let esi = k(SURFACE_PIXELS);
        w(0x6415EC, esi);
        let ebp = k(SURFACE_PITCH);
        if !lock_texture(to)?.carry {
            let edi = k(SURFACE_PIXELS);
            w(0x6415F0, edi);
            let format = k(SURFACE_FORMAT);
            if format == 0 {
                return Err(BadInput("a surface without its format").into());
            }
            let format = &*(format as *const PixelFormat);
            let screen = crate::gfx::pixel_format::surface_format();
            let copy = PictureCopy {
                width: k(SURFACE_WIDTH),
                height: k(SURFACE_HEIGHT),
                from_pitch: ebp,
                to_pitch: k(SURFACE_PITCH),
                from_16: SCREEN.bits.get() == 0x10,
                to_16: k(SURFACE_BITS) == 0x10,
            };
            let mut surfaces = GameSurfaces {
                screen: &screen,
                format,
            };
            // eax at the copy: the destination's slot, as the lock was given it.
            copy_picture(&copy, esi, edi, to, &mut surfaces);
            unlock_texture(to)?;
        }
        unlock_texture(from)?;
        Ok(())
    }
}

/// The locked surfaces in game memory, with the screen's and the destination's formats.
struct GameSurfaces<'a> {
    screen: &'a crate::gfx::pixel_format::PixelFormat,
    format: &'a crate::gfx::pixel_format::PixelFormat,
}

impl rc2k_core::race::view_steps::Surfaces for GameSurfaces<'_> {
    fn read16(&mut self, at: u32) -> u16 {
        unsafe { (at as *const u16).read_unaligned() }
    }
    fn read32(&mut self, at: u32) -> u32 {
        unsafe { (at as *const u32).read_unaligned() }
    }
    fn write16(&mut self, at: u32, v: u16) {
        unsafe { (at as *mut u16).write_unaligned(v) }
    }
    fn write32(&mut self, at: u32, v: u32) {
        unsafe { (at as *mut u32).write_unaligned(v) }
    }
    fn components(&mut self, pixel: u32) -> u32 {
        crate::gfx::pixel_to_components::pixel_to_components(self.screen, pixel)
    }
    fn pixel(&mut self, components: u32) -> u32 {
        unsafe { crate::gfx::components_to_pixel::components_to_pixel(self.format, components) }
    }
}

/// `race::forget_view_textures` (0x45B200): the textures to load again this frame, their
/// bits in `dword_6415F4`, each adding 0x4000 to `dword_6D1508`: the two views' cars' +0xADC
/// (slots `dword_5D1E00` / `dword_5D1E04`, bits 1, 2), their +0x720 counted down (slots
/// `dword_5D1DF8` / `dword_5D1DFC`, bits 4, 8), `dword_6D1830` / `dword_6D1834` (slots
/// `dword_5D1E58` / `dword_5D1E5C`, bits 0x10, 0x1000); with the mirror on
/// (`dword_6415E0`, not `dword_6415E8`) bit 0x20 and 0x400 more, its picture copied
/// (`race::copy_mirror_picture`, then slot `dword_5D0E08` forgotten) or with
/// `byte_4F6B74` uploaded (`sub_44E828`, esi `dword_A12AE4`, eax the slot). Its one caller,
/// `race::draw_race_scene`, reads no register it leaves. Refuses (before anything) a null
/// view block or a slot past 0x3FF.
pub unsafe fn forget_view_textures() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{FORGET_ORDER, forgets, mirror_on,
    // check_slots}` (each switch read where the original reads it).
    use crate::scene::forget_slot_texture::forget_slot_texture;
    use rc2k_core::race::view_steps::{
        FORGET_COST, FORGET_ORDER, Forget, MIRROR_BIT, MIRROR_COST, check_slots, forgets, mirror_on,
    };
    unsafe {
        let blocks = [k(0x63DBB0), k(0x63DBB4)];
        if blocks.contains(&0) {
            return Err(BadInput("a view without its car block").into());
        }
        let field = |b: u32, o: u32| ((b + o) as *const u32).read_unaligned();
        // (the switch, the slot's variable) of each.
        let source = |f: Forget| match f {
            Forget::Changing(v) => (field(blocks[v], 0xADC), [0x5D1E00u32, 0x5D1E04][v]),
            Forget::Counted(v) => (field(blocks[v], 0x720), [0x5D1DF8, 0x5D1DFC][v]),
            Forget::Extra(n) => (k([0x6D1830u32, 0x6D1834][n]), [0x5D1E58, 0x5D1E5C][n]),
        };
        let mirror = mirror_on(k(0x6415E0), k(0x6415E8));
        let uploaded =
            ((crate::store::game_addr(DeviceSlot::at(0x4F6B74))) as *const u8).read() != 0;
        let mut slots: Vec<u32> = FORGET_ORDER
            .iter()
            .map(|&(f, _)| source(f))
            .filter(|&(switch, _)| forgets(switch))
            .map(|(_, slot)| k(slot))
            .collect();
        if mirror {
            slots.push(k(0x5D0E08));
        }
        check_slots(&slots)?;
        w(0x6415F4, 0);
        for (f, bit) in FORGET_ORDER {
            let (switch, slot) = source(f);
            if !forgets(switch) {
                continue;
            }
            forget_slot_texture(k(slot))?;
            w(0x6D1508, k(0x6D1508).wrapping_add(FORGET_COST));
            w(0x6415F4, k(0x6415F4) | bit);
            if let Forget::Counted(v) = f {
                let at = (blocks[v] + 0x720) as *mut u32;
                at.write_unaligned(at.read_unaligned().wrapping_sub(1));
            }
        }
        if !mirror {
            return Ok(());
        }
        w(0x6415F4, k(0x6415F4) | MIRROR_BIT);
        if !uploaded {
            copy_mirror_picture()?;
            forget_slot_texture(k(0x5D0E08))?;
        } else {
            // `sub_44E828`, the platform's upload (then the slot forgotten): through the
            // game, esi the picture, eax the slot (the only registers it reads).
            crate::standalone::not_ported(0x44E828, "race::frame_views");
        }
        w(0x6D1508, k(0x6D1508).wrapping_add(MIRROR_COST));
        Ok(())
    }
}

/// The quad's corners the view window's size from (0, 0).
fn window_corners() {
    w(0x623E10, 0);
    w(0x623E20, 0);
    w(0x623E14, k(0x62408C));
    w(0x623E24, k(0x624090));
}

/// `race::draw_screen_tint` (0x455A11): a tinted quad over the view window
/// (`scene::draw_tinted_quad`, which reads no register): with `dword_71C3E0` 0x283080 half through; else the sun's
/// glare, `dword_6365F4` (held to 0 .. 0xFF) through, colour `dword_60A570` red,
/// `byte_60A574` green, `byte_60A578` blue, none when 0; `g_sunOnScreen` put back after.
pub unsafe fn draw_screen_tint() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha,
    // glare_colour}`.
    use rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha, glare_colour};
    if k(0x71C3E0) != 0 {
        w(0x623DEC, NIGHT_ALPHA);
        window_corners();
        w(0x623DE8, NIGHT_COLOUR);
        return unsafe { crate::scene::draw_tinted_quad::draw_tinted_quad() };
    }
    let sun = k(0x6415C4);
    let a = glare_alpha(k(0x6365F4));
    w(0x623DEC, a);
    let mut done = Ok(());
    if a != 0 {
        window_corners();
        let byte = |x: u32| unsafe { (game_addr(rva(x as usize)) as *const u8).read() };
        w(
            0x623DE8,
            glare_colour(k(0x60A570), byte(0x60A574), byte(0x60A578)),
        );
        done = unsafe { crate::scene::draw_tinted_quad::draw_tinted_quad() };
    }
    w(0x6415C4, sun);
    done
}

/// `race::clear_view_depth` (0x44E9A0): with the device on (`dword_5CDA8C`) the clip
/// rectangle (`dword_624138` ..) the whole screen and the depth cleared there
/// (`d3d::clear_viewport_unless_held`, the platform's `Clear`).
pub unsafe fn clear_view_depth() {
    if k(0x5CDA8C) == 0 {
        return;
    }
    w(0x624138, 0);
    w(0x62413C, 0);
    w(0x624140, SCREEN.width.get());
    w(0x624144, SCREEN.height.get());
    clear_viewport_unless_held();
}

/// `race::draw_own_car` (0x455AD3): in a race, when the drawn view's car (`g_drawnCar`) is
/// seen from inside (+0x6FC bit 0x10): `dword_62DAD4` its +0x240; with +0x900, +0x904 or
/// `dword_635014` the depth cleared (`race::clear_view_depth`) and the depth buffer on;
/// the car drawn (`race::draw_car_shined`), all its parts shown (`dword_606280` -1),
/// texture `dword_5D1E10`, render state 0x14 = 2, `g_polygonVertexCount` 4, the marks'
/// overlays (`race::draw_marks_overlays`).
/// Refuses (before anything) the car without its block.
pub unsafe fn draw_own_car() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    unsafe {
        if k(0x621480) == 0 {
            return Ok(());
        }
        let number = k(0x63DBA8 + 4 * k(0x63DB9C));
        let car = k(0x71B3B8 + 4 * car_number(number));
        if car == 0 {
            return Err(BadInput("the drawn car without its block").into());
        }
        w(0x60627C, number);
        let field = |o: u32| ((car + o) as *const u32).read_unaligned();
        if !seen_inside(field(0x6FC)) {
            return Ok(());
        }
        w(0x62DAD4, field(0x240));
        if clears_depth(field(0x900), || field(0x904), || k(0x635014)) {
            clear_view_depth();
            crate::scene::depth_buffer_on::depth_buffer(true);
        }
        // `race::draw_car_shined` reads only esi, the car's block.
        crate::race::car_draw::draw_car_shined(car)?;
        w(0x606280, u32::MAX);
        crate::scene::set_texture::set_texture(k(0x5D1E10));
        crate::scene::set_render_state::set_render_state(2, 0x14);
        w(0x6240F8, 4);
        draw_marks_overlays()
    }
}

/// `fild i; fdiv a; fmul b; fstp`.
fn int_over_times(i: u32, a: u32, b: u32) -> u32 {
    let (v, mut out) = ([i, a, b], 0u32);
    unsafe {
        core::arch::asm!(
            "fild dword ptr [{v}]", "fdiv dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
            "fstp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `race::run_view_marks` (0x456336): with the marks on (`dword_621890`): which views' marks
/// show (`dword_5C2F94` the other view's, `dword_5C2F90`; none in the replay `dword_6235E8`,
/// not the other with `dword_601024`), the view's marks layout (`dword_6416A0[view + 2 x
/// dword_6238A4]` into `dword_5C2F8C`), the view car's turn (+0x8C0) into `flt_5EA3CC`,
/// `drive::share_all_cars`, every car record's first word 1 in modes 0 and 2 (else 0) and
/// the view's 1, the screen's centre for the layout (width / `flt_5C5D6C` x
/// `flt_6416C0[]`, height / `flt_5C5D70` x `flt_6416D4[]`) for `marks::frame_marks` (part 1)
/// and, unless `dword_6416B8`, `marks::run_frame` (part 2), the centre put back. Refuses
/// (before anything) a null view block or car record, or no marks settings.
pub unsafe fn run_view_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{check_view_marks, marks_shown,
    // layout_index, first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    unsafe {
        if k(0x621890) == 0 {
            return Ok(());
        }
        let view = k(0x63DB9C);
        let block = k(0x63DBB0 + 4 * view);
        check_view_marks(
            block,
            |b| k(0x71B3D8 + 4 * b),
            || crate::marks::store::game_read(crate::marks::setup::SETTINGS_RVA),
        )?;
        let (other, own) = marks_shown(view, k(0x6235E8), k(0x601024));
        w(0x5C2F94, other);
        w(0x5C2F90, own);
        let layout_at = layout_index(view, k(0x6238A4));
        let layout = k(0x6416A0 + 4 * layout_at);
        w(0x5C2F8C, layout);
        std::ptr::copy(
            (block + 0x8C0) as *const u8,
            game_addr(rva(0x5EA3CC)) as *mut u8,
            0x40,
        );
        crate::drive::share_all_cars::share_all_cars()?;
        let first = first_word(k(0x62389C));
        for b in 0..CAR_RECORDS {
            ((k(0x71B3D8 + 4 * b)) as *mut u32).write_unaligned(first);
        }
        ((k(0x71B3D8 + 4 * view)) as *mut u32).write_unaligned(1);
        let centre = (k(0x62409C), k(0x6240A0));
        let n = k(0x6416A0 + 4 * layout_at);
        w(
            0x62409C,
            int_over_times(SCREEN.width.get(), k(0x5C5D6C), k(0x6416C0 + 4 * n)),
        );
        w(
            0x6240A0,
            int_over_times(SCREEN.height.get(), k(0x5C5D70), k(0x6416D4 + 4 * n)),
        );
        // Neither reads a register nor leaves one read after it (0x456464, 0x456472).
        let done = crate::marks::frame_marks::frame_marks().and_then(|_| {
            if k(0x6416B8) == 0 {
                crate::marks::run_frame::run_frame()
            } else {
                Ok(())
            }
        });
        w(0x6240A0, centre.1);
        w(0x62409C, centre.0);
        done
    }
}

/// `d3d::clear_viewport_unless_held` (0x44E9EE, was `sub_44E9EE`; ported 2026-10-23, the
/// game's was called before): `d3d::clear_viewport` (0x44E9D9, the same `Clear` of
/// `dword_624138` with flag 2) unless `dword_624238` is not 0.
pub fn clear_viewport_unless_held() {
    if unsafe { crate::store::game_read::<u32>(rva(0x624238)) } == 0 {
        crate::platform_3d::clear_viewport();
    }
}
