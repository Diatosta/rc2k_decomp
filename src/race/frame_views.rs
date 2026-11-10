//! The race frame's views and small drawing steps under `race::draw_race_scene` (part R3, IDA
//! 2026-10-18): the full-screen and mirror windows, the flat triangle, the marks'
//! overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::pointers::GamePtr;
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::Shared;
use crate::race::trap::{Trap, car_at};
use crate::scene::owner::scene;

/// `race::full_view_window` (0x455ECB) / `race::full_view_window_2` (0x456004): the view
/// window the whole screen (`dword_62408C` / `dword_624090` its size, `dword_623FB0` /
/// `dword_623FB4` 0, `dword_624080` 0x100, `scene::set_view_window`), `dword_63993C` 0; the
/// first also `dword_6365C4` 0. Its callers (`race::draw_race_scene`,
/// `race::draw_mirror_view`) read no register it leaves.
pub fn full_view_window(race: &Race, first: bool) {
    let system = crate::system::owner::system();
    let physics = &race.physics;
    let session = &system.session;
    let scene = scene();
    scene.drawing.scene.view_size[0].set(session.screen.width.get());
    scene.drawing.scene.view_size[1].set(session.screen.height.get());
    scene.drawing.scene.view_offset[0].set(0);
    scene.drawing.scene.view_offset[1].set(0);
    scene.drawing.scene.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window(race, scene);
    if first {
        scene.lights.state.draw_mode.set(0);
    }
    physics.ground.view_mirrored.set(0);
}

/// The mirror's grid reach.
const MIRROR_REACH: u32 = 2;

/// `race::mirror_view_window` (0x455F17): the rear-view mirror's window: its corner from
/// `dword_6415FC` / `dword_641600` (8 bytes each, at `dword_6238A4` + 3 x `g_drawnView`, + 6
/// when `dword_6D1502` is 0), 800 x 600 scaled to the screen, 0xB3 x 0x40 big (each even);
/// its edges `dword_624138` .. `dword_624144`; `dword_624080` 0x100,
/// `scene::set_view_window`, the fog (`race::set_up_fog`, into the mirror's `fog`), `dword_6365C4` 0,
/// `dword_63993C` 1, the window with the grid's reach 2 (the original's `g_gridReach`, set
/// by `race::draw_mirror_view`; the fog's `draw_distance_settings` sets the level's again).
/// Refuses (before anything) a size whose scaling would fault.
pub fn mirror_view_window(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    fog: &crate::scene::fog::Fog,
    poly: &crate::scene::polygon_work::PolygonWork,
    car: &crate::physics::car_store::Car,
) -> Result<(), Refused> {
    let session = &system.session;
    // Converted: `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{Window, mirror_entry, mirror_window};
    let n = mirror_entry(
        race.shared.vars.split.get(),
        race.shared.view.drawn_view.get(),
        race.frame.panel_and_overlay.panel.flat.get() == 0,
    );
    // g_mirrorPlaces: 12 entries of two dwords; an entry past them reads on, as the
    // original.
    let places = &race.frame.frame_work.mirror_places;
    let trap = Trap::new();
    let at = |word: usize| {
        let k = (n as usize).wrapping_mul(2).wrapping_add(word);
        places.get(k).map_or_else(
            || {
                trap.keep_read(crate::session::read_on_in(
                    &*race.frame.frame_work,
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
    }) = mirror_window(
        (at(0), at(1)),
        (session.screen.width.get(), session.screen.height.get()),
    )
    else {
        trap.finish()?;
        return Err(BadInput("a mirror place whose scaling overflows").into());
    };
    trap.finish()?;
    scene.drawing.scene.view_offset[0].set(x);
    scene.drawing.scene.view_offset[1].set(y);
    scene.drawing.scene.view_size[0].set(sx);
    scene.drawing.scene.view_size[1].set(sy);
    poly.cleared_rectangle[0].set(x);
    poly.cleared_rectangle[1].set(y);
    poly.cleared_rectangle[2].set(x.wrapping_add(sx));
    poly.cleared_rectangle[3].set(y.wrapping_add(sy));
    scene.drawing.scene.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window_reaching(race, scene, MIRROR_REACH);
    crate::race::frame_draw::set_up_fog(race, fog, poly, car)?;
    scene.lights.state.draw_mode.set(0);
    race.physics.ground.view_mirrored.set(1);
    Ok(())
}

/// `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0) with texture 0, 0,
/// colour 0 (the original's `fld1` / `fldz` stores), `g_polygonVertexCount` 3, drawn
/// (`scene::draw_fan`). Every register kept.
pub fn draw_flat_triangle(_scene: &crate::scene::owner::Scene) {
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
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(3);
    crate::scene::draw_fan::draw_fan(crate::scene::polygon_work::front_end());
}

/// The marks' shadow refusals (`effects::draw_shadow`'s first and second shadows), checked
/// before anything.
fn shadow_refuses(
    sh: &crate::effects::draw_shadow::Shadow<'_>,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let corner = sh.corner();
    if corner == u32::MAX {
        return Ok(());
    }
    if corner >= 0x100 {
        return Err(BadInput("a shadow corner past g_marksCorners").into());
    }
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    match sh.angle_refuses() {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// `race::draw_marks_overlays` (0x45632B): `effects::draw_overlays`, then
/// `effects::draw_shadow`. Refuses (before anything) the second shadow's refusals.
pub fn draw_marks_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    shadow_refuses(&crate::effects::draw_shadow::FIRST, effects)?;
    // Neither reads a register, and the race reads none they leave (only DF, ST).
    {
        crate::effects::draw_overlays::draw_overlays(
            ctx,
            &effects.masks,
            &effects.windscreen_drops,
            gfx,
            effects,
        )?;
        crate::effects::draw_shadow::draw_shadow(ctx, &effects.masks, effects)?;
    }
    Ok(())
}

/// `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car`. (Its callers,
/// `race::draw_race_scene` and `drive::finish_run`, read no register it leaves.) Refuses
/// (before anything) car 0 without its block.
pub fn load_first_car(shared: &Shared) -> Result<(), Refused> {
    if shared.work.car_block(0) == 0 {
        return Err(BadInput("car 0 without its block").into());
    }
    shared.work.current_car.set(0);
    crate::drive::load_car::load_car(shared)?;
    Ok(())
}

/// `race::copy_mirror_picture` (0x455DAD): the mirror's surface (slot `dword_624234`) copied
/// onto slot `dword_5D0E08`'s, each pixel through `gfx::pixel_to_components` (the screen's
/// format, 16 or 32 bits by `g_screenBits`) and back by `gfx::components_to_pixel` (the
/// destination's) with alpha 0xFF; the destination's width and height; each surface locked
/// and unlocked by `d3d::lock_texture` / `d3d::unlock_texture` (carry: the lock failed).
/// The registers it leaves are read by no caller. Refuses (before the copy) a null
/// destination format.
pub fn copy_mirror_picture(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let gfx_drawing = &gfx().drawing;
    // Converted: `rc2k_core::race::view_steps::copy_picture` (the last pixel's bits carried
    // from pixel to pixel as the original's; the formats' conversions the gfx module's).
    use crate::d3d::{lock_texture::lock_texture, unlock_texture::unlock_texture};
    use rc2k_core::race::view_steps::{PictureCopy, copy_picture};
    let (from, to) = (
        scene.drawing.scene.mirror_picture.get(),
        scene.textures.kinds.kind_first_textures[200].get(),
    );
    if lock_texture(from)?.carry {
        return Ok(());
    }
    let esi = crate::gfx::surface::address()?;
    frame.frame_work.view_kept[0].set(esi);
    let ebp = gfx_drawing.draw_surface.pitch.get();
    if !lock_texture(to)?.carry {
        let edi = crate::gfx::surface::address()?;
        frame.frame_work.view_kept[1].set(edi);
        let format = gfx_drawing.draw_surface.format.get();
        if format == 0 {
            return Err(BadInput("a surface without its format").into());
        }
        let format = crate::gfx::pixel_format::layout(format)?;
        let format = &format;
        let screen = crate::gfx::pixel_format::surface_format()?;
        let copy = PictureCopy {
            width: gfx_drawing.draw_surface.width.get(),
            height: gfx_drawing.draw_surface.height.get(),
            from_pitch: ebp,
            to_pitch: gfx_drawing.draw_surface.pitch.get(),
            from_16: system.session.screen.bits.get() == 0x10,
            to_16: gfx_drawing.draw_surface.bits_per_pixel.get() == 0x10,
        };
        let mut surfaces = GameSurfaces {
            screen: &screen,
            format,
            reads: crate::pointers::Window::new(),
            writes: crate::pointers::Window::new(),
        };
        // What the first pixel's high half is under: the destination's slot, as the lock was
        // given it.
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
/// `byte_4F6B74` uploaded (0x44E828, esi `dword_A12AE4`, eax the slot). Its one caller,
/// `race::draw_race_scene`, reads no register it leaves. Refuses (before anything) a null
/// view block or a slot past 0x3FF.
pub fn forget_view_textures(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    let textures = &scene.textures;
    // Converted: `rc2k_core::race::view_steps::{FORGET_ORDER, forgets, mirror_on,
    // check_slots}` (each switch read where the original reads it).
    use crate::scene::forget_slot_texture::forget_slot_texture;
    use rc2k_core::race::view_steps::{
        FORGET_COST, FORGET_ORDER, Forget, MIRROR_BIT, MIRROR_COST, check_slots, forgets, mirror_on,
    };
    let blocks = [shared.view.view_block(0), shared.view.view_block(1)];
    if blocks.contains(&0) {
        return Err(BadInput("a view without its car block").into());
    }
    let trap = Trap::new();
    let field = |b: u32, o: u32| trap.read::<u32>(b + o, "a view car block's word");
    // (the switch, the slot's variable) of each.
    let slot_of = |n: usize| &textures.kinds.kind_first_slots[n];
    let source = |f: Forget| match f {
        Forget::Changing(v) => (field(blocks[v], 0xADC), slot_of(198 + v)),
        Forget::Counted(v) => (field(blocks[v], 0x720), slot_of(196 + v)),
        Forget::Extra(n) => (
            [
                &frame.panel_and_overlay.overlay.forget_flag_0,
                &frame.panel_and_overlay.overlay.forget_flag_1,
            ][n]
                .get(),
            slot_of(220 + n),
        ),
    };
    let mirror = mirror_on(
        frame.frame_work.mirror_wanted.get(),
        frame.frame_work.mirror_switch.get(),
    );
    let uploaded = input.devices.keys_now.get(0x18) != 0; // byte_4F6B74
    let mut slots: Vec<u32> = FORGET_ORDER
        .iter()
        .map(|&(f, _)| source(f))
        .filter(|&(switch, _)| forgets(switch))
        .map(|(_, slot)| slot.get())
        .collect();
    if mirror {
        slots.push(textures.kinds.kind_first_textures[200].get());
    }
    trap.finish()?;
    check_slots(&slots)?;
    frame.frame_work.view_textures_forgotten.set(0);
    for (f, bit) in FORGET_ORDER {
        let (switch, slot) = source(f);
        if !forgets(switch) {
            continue;
        }
        forget_slot_texture(slot.get(), scene)?;
        frame.panel_and_overlay.panel.texture_changes.set(
            frame
                .panel_and_overlay
                .panel
                .texture_changes
                .get()
                .wrapping_add(FORGET_COST),
        );
        frame
            .frame_work
            .view_textures_forgotten
            .set(frame.frame_work.view_textures_forgotten.get() | bit);
        if let Forget::Counted(v) = f {
            let counter = &crate::race::trap::car_at(blocks[v])?.notice_counter;
            counter.set(counter.get().wrapping_sub(1));
        }
    }
    if !mirror {
        return Ok(());
    }
    frame
        .frame_work
        .view_textures_forgotten
        .set(frame.frame_work.view_textures_forgotten.get() | MIRROR_BIT);
    if !uploaded {
        copy_mirror_picture(frame, scene, system)?;
        forget_slot_texture(textures.kinds.kind_first_textures[200].get(), scene)?;
    } else {
        // 0x44E828, the platform's upload (then the slot forgotten): through the
        // game, esi the picture, eax the slot (the only registers it reads).
        crate::standalone::not_ported(0x44E828, "race::frame_views");
    }
    frame.panel_and_overlay.panel.texture_changes.set(
        frame
            .panel_and_overlay
            .panel
            .texture_changes
            .get()
            .wrapping_add(MIRROR_COST),
    );
    Ok(())
}

/// The quad's corners the view window's size from (0, 0).
fn window_corners(
    scene: &crate::scene::owner::Scene,
    view: &crate::race::view_scratch::ViewScratch,
) {
    let drawing = &scene.drawing;
    view.quad.corners[0].set(0);
    view.quad.corners[4].set(0);
    view.quad.corners[1].set(drawing.scene.view_size[0].get());
    view.quad.corners[5].set(drawing.scene.view_size[1].get());
}

/// `race::draw_screen_tint` (0x455A11): a tinted quad over the view window
/// (`scene::draw_tinted_quad`, which reads no register): with `dword_71C3E0` 0x283080 half through; else the sun's
/// glare, the view's fog's far value (`dword_6365F4`, held to 0 .. 0xFF) through, colour `dword_60A570` red,
/// `byte_60A574` green, `byte_60A578` blue, none when 0; `g_sunOnScreen` put back after.
pub fn draw_screen_tint(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Converted: `rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha,
    // glare_colour}`.
    use rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha, glare_colour};
    if race.shared.work.in_credits.get() != 0 {
        race.shared.vars.quad_colour[1].set(NIGHT_ALPHA);
        window_corners(scene, ctx.view);
        race.shared.vars.quad_colour[0].set(NIGHT_COLOUR);
        return crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    let sun = race.frame.frame_work.sun_on_screen.get();
    let a = glare_alpha(fog.far.get());
    race.shared.vars.quad_colour[1].set(a);
    let mut done = Ok(());
    if a != 0 {
        window_corners(scene, ctx.view);
        // The two glare bytes (byte_60A574, byte_60A578): the low bytes of two dwords.
        let byte = |k: usize| race.course.stage_prefs.unknown_18c0[k].get() as u8;
        race.shared.vars.quad_colour[0].set(glare_colour(
            race.course.stage_prefs.lamp_table_c.get(),
            byte(0),
            byte(1),
        ));
        done = crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    race.frame.frame_work.sun_on_screen.set(sun);
    done
}

/// `race::clear_view_depth` (0x44E9A0): with the device on (`dword_5CDA8C`) the clip
/// rectangle (`dword_624138` ..) the whole screen and the depth cleared there
/// (`d3d::clear_viewport_unless_held`, the platform's `Clear`).
pub fn clear_view_depth(
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    poly: &crate::scene::polygon_work::PolygonWork,
) {
    let session = &system.session;
    if scene.objects.tables.has_3d.get() == 0 {
        return;
    }
    poly.cleared_rectangle[0].set(0);
    poly.cleared_rectangle[1].set(0);
    poly.cleared_rectangle[2].set(session.screen.width.get());
    poly.cleared_rectangle[3].set(session.screen.height.get());
    clear_viewport_unless_held(scene, &poly.cleared_rectangle);
}

/// `race::draw_own_car` (0x455AD3): in a race, when the drawn view's car (`g_drawnCar`) is
/// seen from inside (+0x6FC bit 0x10): `dword_62DAD4` its +0x240; with +0x900, +0x904 or
/// `dword_635014` the depth cleared (`race::clear_view_depth`) and the depth buffer on;
/// the car drawn (`race::draw_car_shined`), all its parts shown (`dword_606280` -1),
/// texture `dword_5D1E10`, render state 0x14 = 2, `g_polygonVertexCount` 4, the marks'
/// overlays (`race::draw_marks_overlays`).
/// Refuses (before anything) the car without its block.
pub fn draw_own_car(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let frame = &race.frame;
    // Converted: `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    if race.shared.switches.other_way.get() == 0 {
        return Ok(());
    }
    let number = race.shared.view.view_cars[race.shared.view.drawn_view.get() as usize].get();
    let car = race.shared.work.car_block(car_number(number) as usize);
    if car == 0 {
        return Err(BadInput("the drawn car without its block").into());
    }
    frame.view_and_records.view_vars.drawn_car.set(number);
    ctx.view.state.drawn_car.set(number);
    // The state bits are the view's own; the rest, the car as seen.
    if !seen_inside(ctx.view.cars.state_of(car)?.get()) {
        return Ok(());
    }
    let block = crate::race::frame_view::car_at(car)?;
    ctx.view.light.car_light.set(block.light_colour.get());
    let clears = clears_depth(
        block.surface_noise_a.get(),
        || block.surface_noise_b.get(),
        || scene.drawing.object_work.view_cleared[0].get(),
    );
    if clears {
        clear_view_depth(scene, system, &ctx.view.polygon);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    // `race::draw_car_shined` reads only esi, the car's block.
    crate::race::car_draw::draw_car_shined(ctx, car)?;
    ctx.view.cars.group_mask.set(u32::MAX);
    crate::scene::set_texture::set_texture(scene.textures.kinds.kind_first_slots[202].get());
    crate::scene::set_render_state::set_render_state(2, 0x14);
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(4);
    draw_marks_overlays(ctx, effects)
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
pub fn run_view_marks(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    effects: &crate::effects::owner::Effects,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let session = &system.session;
    let drawing = &scene.drawing;
    // Converted: `rc2k_core::race::view_steps::{check_view_marks, marks_shown,
    // layout_index, first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    if race.shared.switches.view_marks_on.get() == 0 {
        return Ok(());
    }
    let view = race.shared.view.drawn_view.get();
    let block = race.shared.view.view_block(view as usize);
    let records = race.shared.work.car_records_resolved()?;
    check_view_marks(
        block,
        |b| GamePtr::to(records[b as usize]).addr(),
        || effects.setup.settings.is_set.get(),
    )?;
    let (other, own) = marks_shown(
        view,
        race.shared.vars.scene_switch.get(),
        race.physics.split_tables.still.get(),
    );
    MARKS_SETTINGS.other_marks_shown.set(other);
    MARKS_SETTINGS.own_marks_shown.set(own);
    let layout_at = layout_index(view, race.shared.vars.split.get());
    let layout = race.frame.frame_work.view_layouts[layout_at as usize].get();
    MARKS_SETTINGS.marks_layout.set(layout);
    crate::math::cells::copy_cells(
        &car_at(block)?.whole_matrix,
        &race.frame.matrices.view_place,
        16,
    );
    crate::drive::share_all_cars::share_all_cars(race, scene)?;
    let first = first_word(race.shared.vars.run_kind.get());
    for b in 0..CAR_RECORDS {
        race.shared
            .work
            .car_record(b as usize)?
            .streaks_on
            .set(first);
    }
    race.shared
        .work
        .car_record(view as usize)?
        .streaks_on
        .set(1);
    let centre = (
        drawing.scene.view_centre[0].bits(),
        drawing.scene.view_centre[1].bits(),
    );
    let n = race.frame.frame_work.view_layouts[layout_at as usize].get() as usize;
    drawing.scene.view_centre[0].set_bits(int_over_times(
        session.screen.width.get(),
        race.course.numbers.k800.bits(),
        race.frame.frame_work.view_scale_x[n].bits(),
    ));
    drawing.scene.view_centre[1].set_bits(int_over_times(
        session.screen.height.get(),
        race.course.numbers.k600.bits(),
        race.frame.frame_work.view_scale_y[n].bits(),
    ));
    // Neither reads a register nor leaves one read after it (0x456464, 0x456472).
    let done = crate::effects::frame_marks::frame_marks(race, effects).and_then(|_| {
        if race.frame.frame_work.drawing.get() == 0 {
            crate::effects::run_frame::run_frame(race, fog)
        } else {
            Ok(())
        }
    });
    drawing.scene.view_centre[1].set_bits(centre.1);
    drawing.scene.view_centre[0].set_bits(centre.0);
    done
}

/// `d3d::clear_viewport_unless_held` (0x44E9EE, was 0x44E9EE; ported 2026-10-23, the
/// game's was called before): `d3d::clear_viewport` (0x44E9D9, the same `Clear` of
/// `dword_624138` with flag 2) unless `dword_624238` is not 0.
pub fn clear_viewport_unless_held(
    scene: &crate::scene::owner::Scene,
    own: &[crate::session::Dword; 4],
) {
    if scene.drawing.scene.v3_driver_fix.get() == 0 {
        crate::platform_3d::clear_viewport(own);
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
