//! The race frame's views and small drawing steps under `race::draw_race_scene` (part R3): the full-screen and mirror windows, the flat triangle, the marks'
//! overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;

/// `race::full_view_window` (0x455ECB) / `race::full_view_window_2` (0x456004): the view
/// window the whole screen (`view_size` / `SceneDraw.view_size[1]` its size, `view_offset` /
/// `SceneDraw.view_offset[1]` 0, `view_zoom` 0x100, `scene::set_view_window`); the original's
/// `view_mirrored` 0 (the mirrored flag, now the mirror's own `PolygonWork::view_mirrored`)
/// and the first's `draw_mode` 0 not kept. Its callers (`race::draw_race_scene`,
/// `race::draw_mirror_view`) read no register it leaves.
pub fn full_view_window(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    window: &crate::scene::view_window::ViewWindow,
) {
    let session = &system.session;
    window.view_size[0].set(session.screen.get().width.get());
    window.view_size[1].set(session.screen.get().height.get());
    window.view_offset[0].set(0);
    window.view_offset[1].set(0);
    window.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window(
        race.course.reads(),
        scene.drawing.scene.get(),
        session.screen.get(),
        window,
        &race.frame.view_and_records.get().view_vars,
        &race.shared.vars.get().scale_factor,
    );
    // The original stores 0 in `g_drawMode` here for the frame's first view: never anything
    // else anywhere (it starts at 0; modes 1 and 2 are only read), so not kept (the drawing
    // parts leave the light state alone).
}

/// The mirror's grid reach.
const MIRROR_REACH: u32 = 2;

/// `race::mirror_view_window` (0x455F17): the rear-view mirror's window: its corner from
/// `mirror_places` / `FrameWork.mirror_places[1]` (8 bytes each, at `split` + 3 x `g_drawnView`, + 6
/// when `flat` is 0), 800 x 600 scaled to the screen, 0xB3 x 0x40 big (each even);
/// its edges `cleared_rectangle` .. `SceneDraw.cleared_rectangle[3]`; `view_zoom` 0x100,
/// `scene::set_view_window`, the fog (`race::set_up_fog`, into the mirror's `fog`), `draw_mode` 0,
/// `view_mirrored` 1, the window with the grid's reach 2 (the original's `g_gridReach`, set
/// by `race::draw_mirror_view`; the fog's `draw_distance_settings` sets the level's again).
/// Refuses (before anything) a size whose scaling would fault. On the mirror's `view` (its
/// fog, polygon work, car and view variables).
pub fn mirror_view_window(
    race: &Race,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::workers::PartScratch,
    reads: crate::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let draw = reads.scene_draw;
    let (fog, poly, car, vars) = (&view.fog, &view.polygon, &view.car, &view.vars);
    let drawn_view = view.drawn_view(race);
    // Converted: `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{MirrorRect, mirror_entry, mirror_window};
    let n = mirror_entry(
        reads.vars.split.get(),
        drawn_view,
        reads.panel_and_overlay.panel.flat.get() == 0,
    );
    // g_mirrorPlaces: 12 entries of two dwords. The entry is the split (0 for one player,
    // 1 or 2 for two: `race_shell::set_start_states` holds it there and the split key turns
    // 1 and 2 round) plus 3 for the second view (0 or 1) plus 6 for the second set: at most
    // 11.
    let places = &reads.frame_work.mirror_places;
    let trap = Trap::new();
    let at = |word: usize| {
        let k = (n as usize).wrapping_mul(2).wrapping_add(word);
        places.get(k).map_or_else(
            || {
                trap.keep(BadInput("a mirror place past the twelve"));
                0
            },
            super::super::session::PlainDword::get,
        )
    };
    let Some(MirrorRect {
        x,
        y,
        width: sx,
        height: sy,
    }) = mirror_window(
        (at(0), at(1)),
        (reads.screen.width.get(), reads.screen.height.get()),
    )
    else {
        trap.finish()?;
        return Err(BadInput("a mirror place whose scaling overflows").into());
    };
    trap.finish()?;
    poly.window.view_offset[0].set(x);
    poly.window.view_offset[1].set(y);
    poly.window.view_size[0].set(sx);
    poly.window.view_size[1].set(sy);
    poly.cleared_rectangle[0].set(x);
    poly.cleared_rectangle[1].set(y);
    poly.cleared_rectangle[2].set(x.wrapping_add(sx));
    poly.cleared_rectangle[3].set(y.wrapping_add(sy));
    poly.window.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window_reaching(
        reads.course(),
        draw,
        reads.screen,
        &poly.window,
        vars,
        &reads.vars.scale_factor,
        MIRROR_REACH,
    );
    crate::race::frame_draw::set_up_fog(race, reads.stage_prefs, fog, poly, car, part)?;
    // (The original's 0 into `g_drawMode`: see `full_view_window`.)
    poly.view_mirrored.set(1);
    Ok(())
}

/// `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0) with texture 0, 0,
/// colour 0 (the original's `fld1` / `fldz` stores), `g_polygonVertexCount` 3, drawn
/// (`scene::draw_fan`). Every register kept.
pub fn draw_flat_triangle(part: &crate::workers::PartScratch, scene: &crate::scene::owner::Scene) {
    let buffer = part.vertices;
    // 1.0 at x, y of each of the three vertices; 0.0 at the rest of their floats (z, rhw,
    // u, v) and the specular.
    for vertex in &buffer.vertices[..3] {
        vertex.x.set(1.0f32);
        vertex.y.set(1.0f32);
        vertex.z.set(0.0);
        vertex.u.set(0.0);
        vertex.v.set(0.0);
        vertex.rhw.set(0.0);
        vertex.specular.set(0);
    }
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(3);
    crate::scene::draw_fan::draw_fan(part, scene, crate::scene::polygon_work::front_end(), buffer);
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
    match sh.angle_refuses(effects) {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// `race::draw_marks_overlays` (0x45632B): `effects::draw_overlays`, then
/// `effects::draw_shadow`. Refuses (before anything) the second shadow's refusals.
pub fn draw_marks_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    gfx: &crate::gfx::owner::Gfx,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    shadow_refuses(&crate::effects::draw_shadow::first(effects), effects)?;
    // Neither reads a register, and the race reads none they leave (only DF, ST).
    {
        crate::effects::draw_overlays::draw_overlays(
            ctx,
            &effects.masks,
            &effects.windscreen_drops,
            gfx,
            cheats,
            effects,
        )?;
        crate::effects::draw_shadow::draw_shadow(ctx, &effects.masks, effects)?;
    }
    Ok(())
}

/// `race::load_first_car` (0x460C7A): `g_currentCar` 0, `drive::load_car`. (Its callers,
/// `race::draw_race_scene` and `drive::finish_run`, read no register it leaves.) Refuses
/// (before anything) car 0 without its block.
pub fn load_first_car(shared: &RaceShared) -> Result<(), Refused> {
    if !shared.work.get().has_car_block(0) {
        return Err(BadInput("car 0 without its block").into());
    }
    shared.work.get().current_car.set(0);
    crate::drive::load_car::load_car(shared)?;
    Ok(())
}

/// `race::copy_mirror_picture` (0x455DAD): the mirror's surface (slot `mirror_picture`) copied
/// onto slot `KindTextureTables.kind_first_textures[200]`'s, each pixel through `gfx::pixel_to_components` (the screen's
/// format, 16 or 32 bits by `g_screenBits`) and back by `gfx::components_to_pixel` (the
/// destination's) with alpha 0xFF; the destination's width and height; each surface locked
/// and unlocked by `d3d::lock_texture` / `d3d::unlock_texture` (carry: the lock failed).
/// The registers it leaves are read by no caller. Refuses (before the copy) a null
/// destination format.
pub fn copy_mirror_picture(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let gfx_drawing = &gfx.drawing;
    // Converted: `rc2k_core::race::view_steps::copy_picture` (the last pixel's bits carried
    // from pixel to pixel as the original's; the formats' conversions the gfx module's).
    use crate::d3d::{lock_texture::lock_texture_in, unlock_texture::unlock_texture};
    use rc2k_core::race::view_steps::{PictureCopy, copy_picture};
    let (from, to) = (
        scene.drawing.scene.get().mirror_picture.get(),
        scene.textures.kinds.get().kind_first_textures[200].get(),
    );
    if lock_texture_in(gfx, from)?.carry {
        return Ok(());
    }
    // The surfaces' pixels by handle (kept, never read back).
    let from_pixels = crate::gfx::surface::handle(gfx)?;
    frame.frame_work.get().view_kept[0].set(from_pixels);
    let ebp = gfx_drawing.draw_surface.pitch.get();
    if !lock_texture_in(gfx, to)?.carry {
        let to_pixels = crate::gfx::surface::handle(gfx)?;
        frame.frame_work.get().view_kept[1].set(to_pixels);
        let format = gfx_drawing.draw_surface.format.get();
        if format == 0 {
            return Err(BadInput("a surface without its format").into());
        }
        let format = crate::gfx::pixel_format::layout(system, gfx, format)?;
        let format = &format;
        let screen = crate::gfx::pixel_format::surface_format(system);
        let copy = PictureCopy {
            width: gfx_drawing.draw_surface.width.get(),
            height: gfx_drawing.draw_surface.height.get(),
            from_pitch: ebp,
            to_pitch: gfx_drawing.draw_surface.pitch.get(),
            from_16_bit: system.session.screen.get().bits.get() == 0x10,
            to_16_bit: gfx_drawing.draw_surface.bits_per_pixel.get() == 0x10,
        };
        let mut surfaces = GameSurfaces {
            screen: &screen,
            format,
            from: crate::gfx::locked_pixels::Place::start(from_pixels),
            to: crate::gfx::locked_pixels::Place::start(to_pixels),
            gfx,
        };
        // What the first pixel's high half is under: the destination's slot, as the lock was
        // given it.
        // The pixels are places in each surface's bytes, from offset 0.
        copy_picture(&copy, 0, 0, to, &mut surfaces);
        unlock_texture(to)?;
    }
    unlock_texture(from)?;
    Ok(())
}

/// The locked surfaces in game memory, with the screen's and the destination's formats.
struct GameSurfaces<'a> {
    screen: &'a crate::gfx::pixel_format::PixelFormat,
    format: &'a crate::gfx::pixel_format::PixelFormat,
    /// The pixels read and the pixels written, reached by offset from their start.
    from: crate::gfx::locked_pixels::Place,
    to: crate::gfx::locked_pixels::Place,
    /// The graphics owner (a pixel's conversion leaves its scratch bytes there).
    gfx: &'a crate::gfx::owner::Gfx,
}

impl rc2k_core::race::view_steps::Surfaces for GameSurfaces<'_> {
    fn read16(&mut self, at: u32) -> u16 {
        self.from.must_read_at(at, "a surface's pixel")
    }
    fn read32(&mut self, at: u32) -> u32 {
        self.from.must_read_at(at, "a surface's pixel")
    }
    fn write16(&mut self, at: u32, v: u16) {
        self.to.must_write_at(at, v, "a surface's pixel");
    }
    fn write32(&mut self, at: u32, v: u32) {
        self.to.must_write_at(at, v, "a surface's pixel");
    }
    fn components(&mut self, pixel: u32) -> u32 {
        crate::gfx::pixel_to_components::pixel_to_components(self.screen, pixel)
    }
    fn pixel(&mut self, components: u32) -> u32 {
        crate::gfx::components_to_pixel::components_to_pixel(self.format, components, self.gfx)
    }
}

/// `race::forget_view_textures` (0x45B200): the textures to load again this frame, their
/// bits in `view_textures_forgotten`, each adding 0x4000 to `texture_changes`: the two views' cars' +0xADC
/// (slots `KindTextureTables.kind_first_slots[198]` / `KindTextureTables.kind_first_slots[199]`, bits 1, 2), their +0x720 counted down (slots
/// `KindTextureTables.kind_first_slots[196]` / `KindTextureTables.kind_first_slots[197]`, bits 4, 8), `forget_flag_0` / `forget_flag_1` (slots
/// `KindTextureTables.kind_first_slots[220]` / `KindTextureTables.kind_first_slots[221]`, bits 0x10, 0x1000); with the mirror on
/// (`mirror_wanted`, not the next flag) bit 0x20 and 0x400 more, its picture copied
/// (`race::copy_mirror_picture`, then the mirror's slot forgotten) or with
/// the mirror table uploaded (the platform's upload, with the picture and the slot). Its one
/// caller, `race::draw_race_scene`, uses no result. Refuses (before anything) a null
/// view block or a slot past 0x3FF.
pub fn forget_view_textures(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
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
    let (Ok(first), Ok(second)) = (
        shared.view.get().view_number(0),
        shared.view.get().view_number(1),
    ) else {
        return Err(BadInput("a view without its car block").into());
    };
    let blocks = [first, second];
    let trap = Trap::new();
    let field = |b: usize, word: fn(&Car) -> u32| word(&race.car_blocks().cars[b]);
    // (the switch, the slot's variable) of each.
    let slot_of = |n: usize| &textures.kinds.get().kind_first_slots[n];
    let source = |f: Forget| match f {
        Forget::Changing(v) => (
            field(blocks[v], |car| car.map_redrawn.get()),
            slot_of(198 + v),
        ),
        Forget::Counted(v) => (
            field(blocks[v], |car| car.notice_counter.get()),
            slot_of(196 + v),
        ),
        Forget::Extra(n) => (
            [
                &frame.panel_and_overlay.get().overlay.forget_flag_0,
                &frame.panel_and_overlay.get().overlay.forget_flag_1,
            ][n]
                .get(),
            slot_of(220 + n),
        ),
    };
    let mirror = mirror_on(
        frame.frame_work.get().mirror_wanted.get(),
        frame.frame_work.get().mirror_switch.get(),
    );
    let uploaded = input.devices.keys_now.get(0x18) != 0; // Devices.keys_now
    let mut slots: Vec<u32> = FORGET_ORDER
        .iter()
        .map(|&(f, _)| source(f))
        .filter(|&(switch, _)| forgets(switch))
        .map(|(_, slot)| slot.get())
        .collect();
    if mirror {
        slots.push(textures.kinds.get().kind_first_textures[200].get());
    }
    trap.finish()?;
    check_slots(&slots)?;
    frame.frame_work.get().view_textures_forgotten.set(0);
    for (f, bit) in FORGET_ORDER {
        let (switch, slot) = source(f);
        if !forgets(switch) {
            continue;
        }
        forget_slot_texture(slot.get(), scene)?;
        frame.panel_and_overlay.get().panel.texture_changes.set(
            frame
                .panel_and_overlay
                .get()
                .panel
                .texture_changes
                .get()
                .wrapping_add(FORGET_COST),
        );
        frame
            .frame_work
            .get()
            .view_textures_forgotten
            .set(frame.frame_work.get().view_textures_forgotten.get() | bit);
        if let Forget::Counted(v) = f {
            let counter = &race.car_blocks().cars[blocks[v]].notice_counter;
            counter.set(counter.get().wrapping_sub(1));
        }
    }
    if !mirror {
        return Ok(());
    }
    frame
        .frame_work
        .get()
        .view_textures_forgotten
        .set(frame.frame_work.get().view_textures_forgotten.get() | MIRROR_BIT);
    if !uploaded {
        copy_mirror_picture(frame, scene, system, gfx)?;
        forget_slot_texture(textures.kinds.get().kind_first_textures[200].get(), scene)?;
    } else {
        // The platform's upload (then the slot forgotten): through the game, with the picture
        // and the slot.
        crate::standalone::not_ported(0x44E828, "race::frame_views");
    }
    frame.panel_and_overlay.get().panel.texture_changes.set(
        frame
            .panel_and_overlay
            .get()
            .panel
            .texture_changes
            .get()
            .wrapping_add(MIRROR_COST),
    );
    Ok(())
}

/// The quad's corners the view window's size from (0, 0).
fn window_corners(view: &crate::race::view_scratch::ViewScratch) {
    view.quad.corners[0].set(0);
    view.quad.corners[4].set(0);
    view.quad.corners[1].set(view.polygon.window.view_size[0].get());
    view.quad.corners[5].set(view.polygon.window.view_size[1].get());
}

/// `race::draw_screen_tint` (0x455A11): a tinted quad over the view window
/// (`scene::draw_tinted_quad`, which reads no register): with `in_credits` 0x283080 half through; else the sun's
/// glare, the view's fog's far value (`fog_far`, held to 0 .. 0xFF) through, colour `lamp_table_c` red,
/// `glare_bytes` green, `StagePrefs.glare_bytes[1]` blue, none when 0; `g_sunOnScreen` put back after.
pub fn draw_screen_tint(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    let race = ctx.race;
    // Converted: `rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha,
    // glare_colour}`.
    use rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha, glare_colour};
    if ctx.reads.work.in_credits.is_set() {
        ctx.view.quad.colour[1].set(NIGHT_ALPHA);
        window_corners(ctx.view);
        ctx.view.quad.colour[0].set(NIGHT_COLOUR);
        return crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    let sun = ctx.view.polygon.sun.on_screen.get();
    let a = glare_alpha(fog.far.get());
    ctx.view.quad.colour[1].set(a);
    let mut done = Ok(());
    if a != 0 {
        window_corners(ctx.view);
        // The two glare bytes (`glare_bytes`, StagePrefs.glare_bytes[1]): the low bytes of two dwords.
        let byte = |k: usize| ctx.reads.stage_prefs.glare_bytes[k].get() as u8;
        ctx.view.quad.colour[0].set(glare_colour(
            ctx.reads.stage_prefs.lamp_table_c.get(),
            byte(0),
            byte(1),
        ));
        done = crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    race.frame.frame_work.get().sun_on_screen.set(sun);
    done
}

/// `race::clear_view_depth` (0x44E9A0): with the device on (`d3d_drawing_on`) the clip
/// rectangle (`cleared_rectangle` ..) the whole screen and the depth cleared there
/// (`d3d::clear_viewport_unless_held`, the platform's `Clear`).
pub fn clear_view_depth(
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    part: &crate::workers::PartScratch,
    poly: &crate::scene::polygon_work::PolygonWork,
) {
    let session = &system.session;
    if scene.objects.tables.get().d3d_drawing_on.get() == 0 {
        return;
    }
    poly.cleared_rectangle[0].set(0);
    poly.cleared_rectangle[1].set(0);
    poly.cleared_rectangle[2].set(session.screen.get().width.get());
    poly.cleared_rectangle[3].set(session.screen.get().height.get());
    clear_viewport_unless_held(scene.drawing.scene.get(), part, &poly.cleared_rectangle);
}

/// `race::draw_own_car` (0x455AD3): in a race, when the drawn view's car (`g_drawnCar`) is
/// seen from inside (+0x6FC bit 0x10): `view_light_base` its +0x240; with +0x900, +0x904 or
/// `view_cleared` the depth cleared (`race::clear_view_depth`) and the depth buffer on;
/// the car drawn (`race::draw_car_shined`), all its parts shown (`group_mask` -1),
/// texture `KindTextureTables.kind_first_slots[202]`, render state 0x14 = 2, `g_polygonVertexCount` 4, the marks'
/// overlays (`race::draw_marks_overlays`).
/// Refuses (before anything) the car without its block.
pub fn draw_own_car(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Converted: `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    if ctx.reads.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    let number = ctx.reads.view.view_cars[ctx.view.drawn_view(race) as usize].get();
    if !ctx.reads.work.has_car_block(car_number(number) as usize) {
        return Err(BadInput("the drawn car without its block").into());
    }
    let car = ctx.reads.work.block_number(car_number(number) as usize)?;
    ctx.view.vars.drawn_car.set(number);
    ctx.view.state.drawn_car.set(number);
    // The state bits are the view's own; the rest, the car as seen.
    if !seen_inside(ctx.view.cars.state_of(car)?.get()) {
        return Ok(());
    }
    let block = crate::race::frame_view::car(car)?;
    ctx.view.light.car_light.set(block.light_colour.get());
    let clears = clears_depth(
        block.surface_noise_a.get(),
        || block.surface_noise_b.get(),
        || ctx.view.object.view_cleared[0].get(),
    );
    if clears {
        clear_view_depth(scene, system, ctx.part, &ctx.view.polygon);
        crate::scene::depth_buffer_on::depth_buffer(ctx.part, true);
    }
    // `race::draw_car_shined` reads only the car's block.
    crate::race::car_draw::draw_car_shined(ctx, car)?;
    ctx.view.cars.group_mask.set(u32::MAX);
    crate::scene::set_texture::set_texture_with(
        ctx.part,
        ctx.reads.object_tables,
        ctx.reads.kind_textures.kind_first_slots[202].get(),
    );
    crate::scene::set_render_state::set_render_state(ctx.part, 2, 0x14);
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(4);
    draw_marks_overlays(ctx, gfx, cheats, effects)
}

/// `fild i; fdiv a; fmul b; fstp`.
fn int_over_times(
    i: u32,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> f32 {
    use crate::math::x87_float::{int, ld, stf};
    stf(int(i as i32) / ld(a) * ld(b))
}

/// `race::run_view_marks` (0x456336): with the marks on (`view_marks_on`): which views' marks
/// show (`other_marks_shown` the other view's, `own_marks_shown`; none in the replay `scene_switch`,
/// not the other with `still`), the view's marks layout (`view_layouts`[view + 2 x
/// `split`]` into `marks_layout`), the view car's turn (+0x8C0) into `view_place`,
/// `drive::share_all_cars`, every car record's first word 1 in modes 0 and 2 (else 0) and
/// the view's 1, the screen's centre for the layout (width / `k800` x
/// `view_scale_x`[]`, height / `k600` x `view_scale_y`[]`) for `effects::frame_marks` (part 1)
/// and, unless `drawing`, `effects::run_frame` (part 2), the centre put back. Refuses
/// (before anything) a null view block or car record, or no marks settings.
pub fn run_view_marks(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    sound: &'static crate::sound::owner::Sound,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let session = &system.session;
    // Converted: `rc2k_core::race::view_steps::{check_view_marks, marks_shown,
    // layout_index, first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    if ctx.reads.switches.view_marks_on.get() == 0 {
        return Ok(());
    }
    let view = ctx.view.drawn_view(race);
    let records = race.shared.car_records()?;
    check_view_marks(
        u32::from(race.shared.view_car(view as usize).is_some()),
        |b| u32::from(records.get(b as usize).is_some()),
        || effects.setup.settings.is_set.get(),
    )?;
    let (other, own) = marks_shown(
        view,
        ctx.reads.vars.scene_switch.get(),
        race.physics.split_tables().still.get(),
    );
    race.effects_settings().other_marks_shown.set(other);
    race.effects_settings().own_marks_shown.set(own);
    let layout_at = layout_index(view, ctx.reads.vars.split.get());
    let layout = race.frame.frame_work.get().view_layouts[layout_at as usize].get();
    race.effects_settings().marks_layout.set(layout);
    crate::math::cells::copy_cells(
        &ctx.reads.car_blocks.cars[ctx.reads.view.view_number(view as usize)?].whole_matrix,
        &race.frame.matrices.view_place,
        16,
    );
    crate::drive::share_all_cars::share_all_cars(
        race,
        sound,
        scene,
        ctx.net,
        crate::drive::set_listener::ListenerView::of(ctx.view, race),
    )?;
    let first = first_word(ctx.reads.vars.run_kind.get());
    for b in 0..CAR_RECORDS {
        race.shared.car_record(b as usize)?.streaks_on.set(first);
    }
    race.shared.car_record(view as usize)?.streaks_on.set(1);
    let centre = (
        ctx.view.polygon.window.view_centre[0].get(),
        ctx.view.polygon.window.view_centre[1].get(),
    );
    let n = race.frame.frame_work.get().view_layouts[layout_at as usize].get() as usize;
    ctx.view.polygon.window.view_centre[0].set(int_over_times(
        session.screen.get().width.get(),
        ctx.reads.numbers.k800.get(),
        race.frame.frame_work.get().view_scale_x[n].get(),
    ));
    ctx.view.polygon.window.view_centre[1].set(int_over_times(
        session.screen.get().height.get(),
        ctx.reads.numbers.k600.get(),
        race.frame.frame_work.get().view_scale_y[n].get(),
    ));
    // Neither reads a register nor leaves one read after it (0x456464, 0x456472).
    let done = crate::effects::frame_marks::frame_marks(race, scene, effects).and_then(|_| {
        if race.frame.frame_work.get().drawing.get() == 0 {
            crate::effects::run_frame::run_frame(
                ctx.part.vertices,
                race,
                scene,
                effects,
                system,
                gfx,
                sound,
                crate::effects::run_frame::EffectsView::of(ctx.race, ctx.view),
            )
        } else {
            Ok(())
        }
    });
    ctx.view.polygon.window.view_centre[1].set(centre.1);
    ctx.view.polygon.window.view_centre[0].set(centre.0);
    done
}

/// `d3d::clear_viewport_unless_held` (0x44E9EE): `d3d::clear_viewport` (0x44E9D9, the same `Clear` of
/// `cleared_rectangle` with flag 2) unless `v3_driver_fix` is not 0.
pub fn clear_viewport_unless_held(
    draw: &crate::scene::draw_store::SceneDraw,
    part: &crate::workers::PartScratch,
    own: &[crate::session::Dword; 4],
) {
    if draw.v3_driver_fix.get() == 0 {
        crate::platform_3d::clear_viewport(part, own);
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, lock, same};
    use crate::test_support::vectors::{Recorder, word};

    #[test]
    fn int_over_times_matches() {
        let _l = lock();
        let mut r = Rng(1);
        let mut file = Recorder::replay("race.frame_views.int_over_times");
        at_each_control_word(|| {
            for _ in 0..20000 {
                let (i, a, b) = (r.int() as u32, r.float(), r.float());
                for w in [i, a, b] {
                    file.input(&w.to_le_bytes());
                }
                let want = word(&mut file);
                let got = int_over_times(i, a, b).to_bits();
                assert!(same(want, got), "{i} {a:#x} {b:#x}: {want:#x} {got:#x}");
            }
        });
        file.finish();
    }
}
