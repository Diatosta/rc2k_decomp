//! The race frame's views and small drawing steps under `sub_45429B` (part R3, IDA
//! 2026-10-18): the full-screen and mirror windows, the flat triangle, the marks'
//! overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::physics::ground_store::GROUND_WORK;
use crate::pointers::GamePtr;
use crate::race::frame_store::FRAME_WORK;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::overlay_store::RACE_OVERLAY;
use crate::race::panel_store::PANEL;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::trap::{Trap, car_at};
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::kind_texture_store::KIND_TEXTURES;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::session::SCREEN;

/// `race::full_view_window` (0x455ECB) / `race::full_view_window_2` (0x456004): the view
/// window the whole screen (`dword_62408C` / `dword_624090` its size, `dword_623FB0` /
/// `dword_623FB4` 0, `dword_624080` 0x100, `scene::set_view_window`), `dword_63993C` 0; the
/// first also `dword_6365C4` 0. Its callers (`race::draw_race_scene`,
/// `race::draw_mirror_view`) read no register it leaves.
pub fn full_view_window(first: bool) {
    DRAW.view_size[0].set(SCREEN.width.get());
    DRAW.view_size[1].set(SCREEN.height.get());
    DRAW.view_offset[0].set(0);
    DRAW.view_offset[1].set(0);
    DRAW.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window();
    if first {
        LIGHT_STATE.draw_mode.set(0);
    }
    GROUND_WORK.view_mirrored.set(0);
}

/// `race::mirror_view_window` (0x455F17): the rear-view mirror's window: its corner from
/// `dword_6415FC` / `dword_641600` (8 bytes each, at `dword_6238A4` + 3 x `g_drawnView`, + 6
/// when `dword_6D1502` is 0), 800 x 600 scaled to the screen, 0xB3 x 0x40 big (each even);
/// its edges `dword_624138` .. `dword_624144`; `dword_624080` 0x100,
/// `scene::set_view_window`, the fog (`race::set_up_fog`), `dword_6365C4` 0,
/// `dword_63993C` 1. Refuses (before anything) a size whose scaling would fault.
pub fn mirror_view_window() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{Window, mirror_entry, mirror_window};
    let n = mirror_entry(
        RACE_VARS.split.get(),
        RACE_VIEW.drawn_view.get(),
        PANEL.flat.get() == 0,
    );
    // g_mirrorPlaces: 12 entries of two dwords; an entry past them reads on, as the
    // original.
    let places = &FRAME_WORK.mirror_places;
    let trap = Trap::new();
    let at = |word: usize| {
        let k = (n as usize).wrapping_mul(2).wrapping_add(word);
        places.get(k).map_or_else(
            || {
                trap.keep_read(crate::session::read_on_in(
                    &FRAME_WORK,
                    places,
                    k,
                    "the mirror places",
                ))
            },
            super::super::session::Dword::get,
        )
    };
    let Some(Window {
        x,
        y,
        width: sx,
        height: sy,
    }) = mirror_window((at(0), at(1)), (SCREEN.width.get(), SCREEN.height.get()))
    else {
        trap.finish()?;
        return Err(BadInput("a mirror place whose scaling overflows").into());
    };
    trap.finish()?;
    DRAW.view_offset[0].set(x);
    DRAW.view_offset[1].set(y);
    DRAW.view_size[0].set(sx);
    DRAW.view_size[1].set(sy);
    DRAW.cleared_rectangle[0].set(x);
    DRAW.cleared_rectangle[1].set(y);
    DRAW.cleared_rectangle[2].set(x.wrapping_add(sx));
    DRAW.cleared_rectangle[3].set(y.wrapping_add(sy));
    DRAW.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window();
    crate::race::frame_draw::set_up_fog()?;
    LIGHT_STATE.draw_mode.set(0);
    GROUND_WORK.view_mirrored.set(1);
    Ok(())
}

/// `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0) with texture 0, 0,
/// colour 0 (the original's `fld1` / `fldz` stores), `g_polygonVertexCount` 3, drawn
/// (`scene::draw_fan`). Every register kept (`pusha` .. `popa`).
pub fn draw_flat_triangle() {
    // 1.0 at x, y of each of the three vertices; 0.0 at the rest of their floats (z, rhw,
    // u, v) and the specular.
    for vertex in &crate::d3d::vertices_store::VERTICES.vertices[..3] {
        vertex.x.set_bits(1.0f32.to_bits());
        vertex.y.set_bits(1.0f32.to_bits());
        vertex.z.set_bits(0);
        vertex.u.set_bits(0);
        vertex.v.set_bits(0);
        vertex.rhw.set_bits(0);
        vertex.specular.set(0);
    }
    DRAW.polygon_vertex_count.set(3);
    crate::scene::draw_fan::draw_fan();
}

/// The marks' shadow refusals (`effects::draw_shadow`'s first and second shadows), checked
/// before anything.
fn shadow_refuses(sh: &crate::effects::draw_shadow::Shadow<'_>) -> Result<(), Refused> {
    let corner = sh.corner();
    if corner == u32::MAX {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    if crate::effects::setup::SETTINGS.pointer.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    match sh.angle_refuses() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// `race::draw_marks_overlays` (0x45632B): `effects::draw_overlays`, then
/// `effects::draw_shadow`. Refuses (before anything) the second shadow's refusals.
pub fn draw_marks_overlays() -> Result<(), Refused> {
    shadow_refuses(&crate::effects::draw_shadow::FIRST)?;
    // Neither reads a register, and the race reads none they leave (only DF, ST).
    {
        crate::effects::draw_overlays::draw_overlays()?;
        crate::effects::draw_shadow::draw_shadow()?;
    }
    Ok(())
}

/// `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car`. (Its callers,
/// `race::draw_race_scene` and `drive::finish_run`, read no register it leaves.) Refuses
/// (before anything) car 0 without its block.
pub fn load_first_car() -> Result<(), Refused> {
    if RACE_WORK.car_block(0) == 0 {
        return Err(BadInput("car 0 without its block").into());
    }
    RACE_WORK.current_car.set(0);
    crate::drive::load_car::load_car()?;
    Ok(())
}

/// `race::copy_mirror_picture` (0x455DAD): the mirror's surface (slot `dword_624234`) copied
/// onto slot `dword_5D0E08`'s, each pixel through `gfx::pixel_to_components` (the screen's
/// format, 16 or 32 bits by `g_screenBits`) and back by `gfx::components_to_pixel` (the
/// destination's) with alpha 0xFF; the destination's width and height; each surface locked
/// and unlocked by `d3d::lock_texture` / `d3d::unlock_texture` (carry: the lock failed).
/// The registers it leaves are read by no caller. Refuses (before the copy) a null
/// destination format.
pub fn copy_mirror_picture() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{copy_picture, copied_any}` (eax carried
    // from pixel to pixel as the original's; the formats' conversions the gfx module's).
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    use rc2k_core::race::view_steps::{PictureCopy, copy_picture};
    let (from, to) = (
        DRAW.mirror_picture.get(),
        KIND_TEXTURES.kind_first_textures[200].get(),
    );
    if lock_texture(from)?.carry {
        return Ok(());
    }
    let esi = crate::gfx::surface::address()?;
    FRAME_WORK.view_kept[0].set(esi);
    let ebp = DRAW_SURFACE.pitch.get();
    if !lock_texture(to)?.carry {
        let edi = crate::gfx::surface::address()?;
        FRAME_WORK.view_kept[1].set(edi);
        let format = DRAW_SURFACE.format.get();
        if format == 0 {
            return Err(BadInput("a surface without its format").into());
        }
        let format = crate::gfx::pixel_format::layout(format)?;
        let format = &format;
        let screen = crate::gfx::pixel_format::surface_format()?;
        let copy = PictureCopy {
            width: DRAW_SURFACE.width.get(),
            height: DRAW_SURFACE.height.get(),
            from_pitch: ebp,
            to_pitch: DRAW_SURFACE.pitch.get(),
            from_16: SCREEN.bits.get() == 0x10,
            to_16: DRAW_SURFACE.bits_per_pixel.get() == 0x10,
        };
        let mut surfaces = GameSurfaces {
            screen: &screen,
            format,
            reads: crate::pointers::Window::new(),
            writes: crate::pointers::Window::new(),
        };
        // eax at the copy: the destination's slot, as the lock was given it.
        copy_picture(&copy, esi, edi, to, &mut surfaces);
        unlock_texture(to)?;
    }
    unlock_texture(from)?;
    Ok(())
}

/// The locked surfaces in game memory, with the screen's and the destination's formats.
struct GameSurfaces<'a> {
    screen: &'a crate::gfx::pixel_format::PixelFormat,
    format: &'a crate::gfx::pixel_format::PixelFormat,
    /// The surfaces' pixels checked (the screen's and the textures' buffers are regions),
    /// a window for the one read and one for the one written.
    reads: crate::pointers::Window,
    writes: crate::pointers::Window,
}

impl rc2k_core::race::view_steps::Surfaces for GameSurfaces<'_> {
    fn read16(&mut self, at: u32) -> u16 {
        self.reads.read_or_end(at, "a surface's pixel")
    }
    fn read32(&mut self, at: u32) -> u32 {
        self.reads.read_or_end(at, "a surface's pixel")
    }
    fn write16(&mut self, at: u32, v: u16) {
        self.writes.write_or_end(at, v, "a surface's pixel");
    }
    fn write32(&mut self, at: u32, v: u32) {
        self.writes.write_or_end(at, v, "a surface's pixel");
    }
    fn components(&mut self, pixel: u32) -> u32 {
        crate::gfx::pixel_to_components::pixel_to_components(self.screen, pixel)
    }
    fn pixel(&mut self, components: u32) -> u32 {
        crate::gfx::components_to_pixel::components_to_pixel(self.format, components)
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
pub fn forget_view_textures() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{FORGET_ORDER, forgets, mirror_on,
    // check_slots}` (each switch read where the original reads it).
    use crate::scene::forget_slot_texture::forget_slot_texture;
    use rc2k_core::race::view_steps::{
        FORGET_COST, FORGET_ORDER, Forget, MIRROR_BIT, MIRROR_COST, check_slots, forgets, mirror_on,
    };
    let blocks = [RACE_VIEW.view_block(0), RACE_VIEW.view_block(1)];
    if blocks.contains(&0) {
        return Err(BadInput("a view without its car block").into());
    }
    let trap = Trap::new();
    let field = |b: u32, o: u32| trap.read::<u32>(b + o, "a view car block's word");
    // (the switch, the slot's variable) of each.
    let slot_of = |n: usize| &KIND_TEXTURES.kind_first_slots[n];
    let source = |f: Forget| match f {
        Forget::Changing(v) => (field(blocks[v], 0xADC), slot_of(198 + v)),
        Forget::Counted(v) => (field(blocks[v], 0x720), slot_of(196 + v)),
        Forget::Extra(n) => (
            [&RACE_OVERLAY.forget_flag_0, &RACE_OVERLAY.forget_flag_1][n].get(),
            slot_of(220 + n),
        ),
    };
    let mirror = mirror_on(
        FRAME_WORK.mirror_wanted.get(),
        FRAME_WORK.mirror_switch.get(),
    );
    let uploaded = crate::input::store::DEVICES.keys_now.get(0x18) != 0; // byte_4F6B74
    let mut slots: Vec<u32> = FORGET_ORDER
        .iter()
        .map(|&(f, _)| source(f))
        .filter(|&(switch, _)| forgets(switch))
        .map(|(_, slot)| slot.get())
        .collect();
    if mirror {
        slots.push(KIND_TEXTURES.kind_first_textures[200].get());
    }
    trap.finish()?;
    check_slots(&slots)?;
    FRAME_WORK.view_textures_forgotten.set(0);
    for (f, bit) in FORGET_ORDER {
        let (switch, slot) = source(f);
        if !forgets(switch) {
            continue;
        }
        forget_slot_texture(slot.get())?;
        PANEL
            .texture_changes
            .set(PANEL.texture_changes.get().wrapping_add(FORGET_COST));
        FRAME_WORK
            .view_textures_forgotten
            .set(FRAME_WORK.view_textures_forgotten.get() | bit);
        if let Forget::Counted(v) = f {
            // +0x720: the notice counter.
            let counter = &crate::race::trap::car_at(blocks[v])?.notice_unknown_6f8;
            counter.set(counter.get().wrapping_sub(1));
        }
    }
    if !mirror {
        return Ok(());
    }
    FRAME_WORK
        .view_textures_forgotten
        .set(FRAME_WORK.view_textures_forgotten.get() | MIRROR_BIT);
    if !uploaded {
        copy_mirror_picture()?;
        forget_slot_texture(KIND_TEXTURES.kind_first_textures[200].get())?;
    } else {
        // `sub_44E828`, the platform's upload (then the slot forgotten): through the
        // game, esi the picture, eax the slot (the only registers it reads).
        crate::standalone::not_ported(0x44E828, "race::frame_views");
    }
    PANEL
        .texture_changes
        .set(PANEL.texture_changes.get().wrapping_add(MIRROR_COST));
    Ok(())
}

/// The quad's corners the view window's size from (0, 0).
fn window_corners() {
    RACE_VARS.quad_corners[0].set(0);
    RACE_VARS.quad_corners[4].set(0);
    RACE_VARS.quad_corners[1].set(DRAW.view_size[0].get());
    RACE_VARS.quad_corners[5].set(DRAW.view_size[1].get());
}

/// `race::draw_screen_tint` (0x455A11): a tinted quad over the view window
/// (`scene::draw_tinted_quad`, which reads no register): with `dword_71C3E0` 0x283080 half through; else the sun's
/// glare, `dword_6365F4` (held to 0 .. 0xFF) through, colour `dword_60A570` red,
/// `byte_60A574` green, `byte_60A578` blue, none when 0; `g_sunOnScreen` put back after.
pub fn draw_screen_tint() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha,
    // glare_colour}`.
    use rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha, glare_colour};
    if RACE_WORK.in_credits.get() != 0 {
        RACE_VARS.quad_colour[1].set(NIGHT_ALPHA);
        window_corners();
        RACE_VARS.quad_colour[0].set(NIGHT_COLOUR);
        return crate::scene::draw_tinted_quad::draw_tinted_quad();
    }
    let sun = FRAME_WORK.sun_on_screen.get();
    let a = glare_alpha(LIGHT_STATE.fog_far.get());
    RACE_VARS.quad_colour[1].set(a);
    let mut done = Ok(());
    if a != 0 {
        window_corners();
        // The two glare bytes (byte_60A574, byte_60A578): the low bytes of two dwords.
        let byte = |k: usize| STAGE_PREFS.unknown_18c0[k].get() as u8;
        RACE_VARS.quad_colour[0].set(glare_colour(
            STAGE_PREFS.lamp_table_c.get(),
            byte(0),
            byte(1),
        ));
        done = crate::scene::draw_tinted_quad::draw_tinted_quad();
    }
    FRAME_WORK.sun_on_screen.set(sun);
    done
}

/// `race::clear_view_depth` (0x44E9A0): with the device on (`dword_5CDA8C`) the clip
/// rectangle (`dword_624138` ..) the whole screen and the depth cleared there
/// (`d3d::clear_viewport_unless_held`, the platform's `Clear`).
pub fn clear_view_depth() {
    if OBJECT_TABLES.has_3d.get() == 0 {
        return;
    }
    DRAW.cleared_rectangle[0].set(0);
    DRAW.cleared_rectangle[1].set(0);
    DRAW.cleared_rectangle[2].set(SCREEN.width.get());
    DRAW.cleared_rectangle[3].set(SCREEN.height.get());
    clear_viewport_unless_held();
}

/// `race::draw_own_car` (0x455AD3): in a race, when the drawn view's car (`g_drawnCar`) is
/// seen from inside (+0x6FC bit 0x10): `dword_62DAD4` its +0x240; with +0x900, +0x904 or
/// `dword_635014` the depth cleared (`race::clear_view_depth`) and the depth buffer on;
/// the car drawn (`race::draw_car_shined`), all its parts shown (`dword_606280` -1),
/// texture `dword_5D1E10`, render state 0x14 = 2, `g_polygonVertexCount` 4, the marks'
/// overlays (`race::draw_marks_overlays`).
/// Refuses (before anything) the car without its block.
pub fn draw_own_car() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    if FRAME_SWITCHES.other_way.get() == 0 {
        return Ok(());
    }
    let number = RACE_VIEW.view_cars[RACE_VIEW.drawn_view.get() as usize].get();
    let car = RACE_WORK.car_block(car_number(number) as usize);
    if car == 0 {
        return Err(BadInput("the drawn car without its block").into());
    }
    VIEW_VARS.drawn_car.set(number);
    let block = car_at(car)?;
    if !seen_inside(block.unknown_6f4[2].get()) {
        return Ok(());
    }
    LIGHT_WORK.view_light_base.set(block.light_colour.get());
    let clears = clears_depth(
        block.surface_noise_a.get(),
        || block.surface_noise_b.get(),
        || OBJECT_WORK.view_cleared[0].get(),
    );
    if clears {
        clear_view_depth();
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    // `race::draw_car_shined` reads only esi, the car's block.
    crate::race::car_draw::draw_car_shined(car)?;
    VIEW_VARS.group_mask.set(u32::MAX);
    crate::scene::set_texture::set_texture(KIND_TEXTURES.kind_first_slots[202].get());
    crate::scene::set_render_state::set_render_state(2, 0x14);
    DRAW.polygon_vertex_count.set(4);
    draw_marks_overlays()
}

/// `fild i; fdiv a; fmul b; fstp`.
fn int_over_times(i: u32, a: u32, b: u32) -> u32 {
    use crate::math::x87_float::{int, ld, st};
    st(int(i as i32) / ld(a) * ld(b))
}

/// `race::run_view_marks` (0x456336): with the marks on (`dword_621890`): which views' marks
/// show (`dword_5C2F94` the other view's, `dword_5C2F90`; none in the replay `dword_6235E8`,
/// not the other with `dword_601024`), the view's marks layout (`dword_6416A0[view + 2 x
/// dword_6238A4]` into `dword_5C2F8C`), the view car's turn (+0x8C0) into `flt_5EA3CC`,
/// `drive::share_all_cars`, every car record's first word 1 in modes 0 and 2 (else 0) and
/// the view's 1, the screen's centre for the layout (width / `flt_5C5D6C` x
/// `flt_6416C0[]`, height / `flt_5C5D70` x `flt_6416D4[]`) for `effects::frame_marks` (part 1)
/// and, unless `dword_6416B8`, `effects::run_frame` (part 2), the centre put back. Refuses
/// (before anything) a null view block or car record, or no marks settings.
pub fn run_view_marks() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::view_steps::{check_view_marks, marks_shown,
    // layout_index, first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    if FRAME_SWITCHES.view_marks_on.get() == 0 {
        return Ok(());
    }
    let view = RACE_VIEW.drawn_view.get();
    let block = RACE_VIEW.view_block(view as usize);
    let records = RACE_WORK.car_records_resolved()?;
    check_view_marks(
        block,
        |b| GamePtr::to(records[b as usize]).addr(),
        || crate::effects::setup::SETTINGS.pointer.get(),
    )?;
    let (other, own) = marks_shown(
        view,
        RACE_VARS.scene_switch.get(),
        crate::physics::split_tables_store::SPLIT_TABLES.still.get(),
    );
    MARKS_SETTINGS.other_marks_shown.set(other);
    MARKS_SETTINGS.own_marks_shown.set(own);
    let layout_at = layout_index(view, RACE_VARS.split.get());
    let layout = FRAME_WORK.view_layouts[layout_at as usize].get();
    MARKS_SETTINGS.marks_layout.set(layout);
    crate::math::cells::copy_cells(&car_at(block)?.whole_matrix, &MATRICES.view_place, 16);
    crate::drive::share_all_cars::share_all_cars()?;
    let first = first_word(RACE_VARS.run_kind.get());
    for b in 0..CAR_RECORDS {
        RACE_WORK
            .car_record(b as usize)?
            .at(0)
            .write(first, "a car record's first word")?;
    }
    RACE_WORK
        .car_record(view as usize)?
        .at(0)
        .write(1u32, "a car record's first word")?;
    let centre = (DRAW.view_centre[0].bits(), DRAW.view_centre[1].bits());
    let n = FRAME_WORK.view_layouts[layout_at as usize].get() as usize;
    DRAW.view_centre[0].set_bits(int_over_times(
        SCREEN.width.get(),
        NUMBERS.k800.bits(),
        FRAME_WORK.view_scale_x[n].bits(),
    ));
    DRAW.view_centre[1].set_bits(int_over_times(
        SCREEN.height.get(),
        NUMBERS.k600.bits(),
        FRAME_WORK.view_scale_y[n].bits(),
    ));
    // Neither reads a register nor leaves one read after it (0x456464, 0x456472).
    let done = crate::effects::frame_marks::frame_marks().and_then(|_| {
        if FRAME_WORK.drawing.get() == 0 {
            crate::effects::run_frame::run_frame()
        } else {
            Ok(())
        }
    });
    DRAW.view_centre[1].set_bits(centre.1);
    DRAW.view_centre[0].set_bits(centre.0);
    done
}

/// `d3d::clear_viewport_unless_held` (0x44E9EE, was `sub_44E9EE`; ported 2026-10-23, the
/// game's was called before): `d3d::clear_viewport` (0x44E9D9, the same `Clear` of
/// `dword_624138` with flag 2) unless `dword_624238` is not 0.
pub fn clear_viewport_unless_held() {
    if crate::scene::draw_store::DRAW.v3_driver_fix.get() == 0 {
        crate::platform_3d::clear_viewport();
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};

    /// The original block.
    fn int_over_times_asm(i: u32, a: u32, b: u32) -> u32 {
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

    #[test]
    fn int_over_times_matches() {
        let _l = lock();
        let mut r = Rng(1);
        at_each_control_word(|| {
            for _ in 0..20000 {
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                let (want, got) = (int_over_times_asm(i, a, b), int_over_times(i, a, b));
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
    }
}
