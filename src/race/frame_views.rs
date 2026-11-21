//! The race frame's views and small drawing steps under `race::draw_race_scene`: the full-screen
//! and mirror windows, the flat triangle, the marks' overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;

/// Sets the view window to the whole screen: `view_size` / `SceneDraw.view_size[1]` the screen's
/// size, `view_offset` / `SceneDraw.view_offset[1]` 0, `view_zoom` 0x100. The first view's
/// `draw_mode` is not set to 0 (nothing reads it as anything else). The mirror's own
/// `PolygonWork::view_mirrored` replaces the old shared flag.
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
    // `draw_mode` is not written: it stays 0 (modes 1 and 2 are only read), and the drawing parts
    // leave the light state alone.
}

/// The mirror's grid reach.
const MIRROR_REACH: u32 = 2;

/// Sets the rear-view mirror's window on the mirror's `view` (fog, polygon work, car and view
/// variables): its corner from `FrameWork.mirror_places[1]` (8 bytes each, at `split` + 3 x
/// `drawn_view`, + 6 when `flat` is 0), 800 x 600 scaled to the screen, 0xB3 x 0x40 big (each
/// even), `view_zoom` 0x100, the fog from `race::set_up_fog`, `draw_mode` 0, `view_mirrored` 1 and
/// grid reach 2. Refuses (before anything) a size whose scaling would fault.
pub fn mirror_view_window(
    race: &Race,
    view: &crate::race::view_scratch::ViewScratch,
    part: &crate::scene::scratch_store::PartScratch,
    reads: crate::scene::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let draw = reads.scene_draw;
    let (fog, poly, car, vars) = (&view.fog, &view.polygon, &view.car, &view.vars);
    let drawn_view = view.drawn_view(race);
    // Logic in `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{MirrorRect, mirror_entry, mirror_window};
    let n = mirror_entry(
        reads.vars.split.get(),
        drawn_view,
        reads.panel_and_overlay.panel.flat.get() == 0,
    );
    // mirror_places: 12 entries of two dwords. The entry is the split (0 for one player, 1 or 2 for
    // two: `race_shell::set_start_states` holds it there and the split key turns 1 and 2 round)
    // plus 3 for the second view (0 or 1) plus 6 for the second set: at most 11.
    let places = &reads.frame_work.mirror_places;
    let trap = Trap::new();
    let at = |word: usize| {
        let k = (n as usize).wrapping_mul(2).wrapping_add(word);
        places.get(k).map_or_else(
            || {
                trap.keep(BadInput("a mirror place past the twelve"));
                0
            },
            crate::memory::session::PlainDword::get,
        )
    };
    // The mirror keeps to the part of the screen clear of a phone's notch and home indicator.
    let [inset_x, inset_y, inset_width, inset_height] =
        crate::hud::safe_inset::get((reads.screen.width.get(), reads.screen.height.get()), None);
    let Some(MirrorRect {
        x: place_x,
        y: place_y,
        width: sx,
        height: sy,
    }) = mirror_window(
        (at(0), at(1)),
        (inset_width, inset_height),
        // Two players side by side keep each mirror to their own half.
        if reads.vars.split.get() == 1 {
            rc2k_core::hud::anchor::Rule::Halves
        } else {
            rc2k_core::hud::anchor::Rule::Thirds
        },
        crate::hud::box_ratio::get(),
    )
    else {
        trap.finish()?;
        return Err(BadInput("a mirror place whose scaling overflows").into());
    };
    trap.finish()?;
    let (x, y) = ((place_x + inset_x) & !1, place_y + inset_y);
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
    crate::race::frame_draw::set_up_fog(
        &reads.buffers.engine_tables,
        reads.stage_prefs,
        fog,
        poly,
        car,
        part,
    )?;
    // `draw_mode` is not written here either: see `full_view_window`.
    poly.view_mirrored.set(1);
    Ok(())
}

/// Draws a flat triangle of three vertices at (1, 1, 0) with texture 0 and colour 0
/// (`scene::draw_fan`, `polygon_vertex_count` 3).
pub(super) fn draw_flat_triangle(
    part: &crate::scene::scratch_store::PartScratch,
    scene: &crate::scene::owner::Scene,
) {
    let buffer = &part.vertices;
    // 1.0 at x, y of each of the three vertices; 0.0 at the rest of their floats (z, rhw, u, v) and
    // the specular.
    for vertex in &buffer.vertices[..3] {
        vertex.x.set(1.0f32);
        vertex.y.set(1.0f32);
        vertex.z.set(0.0);
        vertex.u.set(0.0);
        vertex.v.set(0.0);
        vertex.rhw.set(0.0);
        vertex.specular.set(0);
    }
    scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(3);
    crate::scene::draw_fan::draw_fan(part, scene, scene.scratch.front_end_polygon(), buffer);
}

/// The marks' shadow refusals, checked before anything.
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

/// Draws the marks' overlays, then the shadow (`effects::draw_overlays`, `effects::draw_shadow`).
/// Refuses (before anything) the second shadow's refusals.
fn draw_marks_overlays(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    gfx: &crate::gfx::owner::Gfx,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    shadow_refuses(&crate::effects::draw_shadow::first(effects), effects)?;
    // Neither call needs registers.
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

/// Loads car 0 as the current car (`drive::load_car`). Refuses (before anything) car 0 without
/// its block.
pub fn load_first_car(shared: &RaceShared) -> Result<(), Refused> {
    if !shared.work.get().has_car_block(0) {
        return Err(BadInput("car 0 without its block").into());
    }
    shared.work.get().current_car.set(0);
    crate::drive::load_car::load_car(shared)?;
    Ok(())
}

/// Copies the mirror's surface (slot `mirror_picture`) onto the slot
/// `Textures::first_textures[200]`: each pixel goes through `gfx::pixel_to_components`
/// (the screen's format) and back through
/// `gfx::components_to_pixel` (the destination's) with alpha 0xFF, over the destination's width and
/// height. Both surfaces are locked and unlocked (`d3d::lock_texture` / `d3d::unlock_texture`).
/// Refuses (before the copy) a null destination format.
fn copy_mirror_picture(
    frame: &Frame,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let gfx_drawing = &gfx.drawing;
    // Logic in `rc2k_core::race::view_steps::copy_picture` (the last pixel's bits carried from
    // pixel to pixel as the game's; the formats' conversions the gfx module's).
    use crate::d3d::{lock_texture::lock_texture_in, unlock_texture::unlock_texture};
    use rc2k_core::race::view_steps::{PictureCopy, copy_picture};
    let (from, to) = (
        scene.drawing.scene.get().mirror_picture.get(),
        scene.textures.first_textures()[200],
    );
    if !lock_texture_in(gfx, from)? {
        return Ok(());
    }
    // The surfaces' pixels by handle (kept, never read back).
    let from_pixels = crate::gfx::surface::handle(gfx)?;
    frame.frame_work.get().view_kept[0].set(from_pixels);
    let from_pitch = gfx_drawing.draw_surface.pitch.get();
    if lock_texture_in(gfx, to)? {
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
            from_pitch,
            to_pitch: gfx_drawing.draw_surface.pitch.get(),
            to_16_bit: gfx_drawing.draw_surface.bits_per_pixel.get() == 0x10,
        };
        let mut surfaces = GameSurfaces {
            screen: &screen,
            format,
            from: crate::gfx::locked_pixels::Place::start(from_pixels),
            to: crate::gfx::locked_pixels::Place::start(to_pixels),
            gfx,
        };
        // The pixels are places in each surface's bytes, from offset 0.
        copy_picture(&copy, 0, 0, &mut surfaces);
        unlock_texture(gfx, to)?;
    }
    unlock_texture(gfx, from)?;
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

/// Makes the view textures load again this frame: each adds 0x4000 to `texture_changes` and sets
/// a bit in `view_textures_forgotten`: the two views' cars' first texture (slots
/// `KindSlots.kind_first_slots[198]` / `[199]`, bits 1, 2) and their counted-down one
/// (slots `[196]` / `[197]`, bits 4, 8), `forget_flag_0` / `forget_flag_1` (slots `[220]` /
/// `[221]`, bits 0x10, 0x1000). With the mirror on (`mirror_wanted`, not the next flag) bits 0x20
/// and 0x400 are set too, and the mirror's picture is copied (`race::copy_mirror_picture`, then its
/// slot forgotten). Refuses (before anything) a null view block
/// or a slot past 0x3FF.
pub(super) fn forget_view_textures(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    let textures = &scene.textures;
    // Logic in `rc2k_core::race::view_steps::{FORGET_ORDER, forgets, mirror_on, check_slots}`
    // (each switch read where the game reads it).
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
        u32::from(frame.frame_work.get().mirror_wanted.is_set()),
        frame.frame_work.get().mirror_switch.get(),
    );
    let mut slots: Vec<u32> = FORGET_ORDER
        .iter()
        .map(|&(f, _)| source(f))
        .filter(|&(switch, _)| forgets(switch))
        .map(|(_, slot)| slot.get())
        .collect();
    if mirror {
        slots.push(textures.first_textures()[200]);
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
    copy_mirror_picture(frame, scene, system, gfx)?;
    forget_slot_texture(textures.first_textures()[200], scene)?;
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

/// The quad's corners, from (0, 0) to the view window's size.
fn window_corners(view: &crate::race::view_scratch::ViewScratch) {
    view.quad.borrow_mut().corners[0] = 0;
    view.quad.borrow_mut().corners[4] = 0;
    view.quad.borrow_mut().corners[1] = view.polygon.window.view_size[0].get();
    view.quad.borrow_mut().corners[5] = view.polygon.window.view_size[1].get();
}

/// Draws a tinted quad over the view window: `in_credits` 0x283080 half through; else the sun's
/// glare, with the view fog's far value (`fog_far`, held to 0 ..= 0xFF) as the alpha, colour
/// `lamp_table_c` red, `glare_bytes` green and `StagePrefs.glare_bytes[1]` blue, none when 0.
/// `sun_on_screen` is put back afterwards.
pub(super) fn draw_screen_tint(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    fog: &crate::scene::fog::Fog,
) -> Result<(), Refused> {
    let race = ctx.race;
    // Logic in `rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha,
    // glare_colour}`.
    use rc2k_core::race::view_steps::{NIGHT_ALPHA, NIGHT_COLOUR, glare_alpha, glare_colour};
    if ctx.reads.work.in_credits.is_set() {
        ctx.view.quad.borrow_mut().colour[1] = NIGHT_ALPHA;
        window_corners(ctx.view);
        ctx.view.quad.borrow_mut().colour[0] = NIGHT_COLOUR;
        return crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    let sun = ctx.view.polygon.sun.on_screen.get();
    let a = glare_alpha(fog.borrow().far);
    ctx.view.quad.borrow_mut().colour[1] = a;
    let mut done = Ok(());
    if a != 0 {
        window_corners(ctx.view);
        // The two glare bytes (`glare_bytes`, StagePrefs.glare_bytes[1]): the low bytes of two
        // dwords.
        let byte = |k: usize| ctx.reads.stage_prefs.glare_bytes[k].get() as u8;
        ctx.view.quad.borrow_mut().colour[0] =
            glare_colour(ctx.reads.stage_prefs.lamp_table_c.get(), byte(0), byte(1));
        done = crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    race.frame.frame_work.get().sun_on_screen.set(sun);
    done
}

/// With the device on (`d3d_drawing_on`), sets the clip rectangle (`cleared_rectangle` ..) to the
/// whole screen and clears the depth there (`d3d::clear_viewport_unless_held`).
fn clear_view_depth(
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    part: &crate::scene::scratch_store::PartScratch,
    poly: &crate::scene::polygon_work_store::PolygonWork,
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

/// In a race, when the drawn view's car (`drawn_car`) is seen from inside: sets `view_light_base`
/// from the car's block, clears the depth (`race::clear_view_depth`) when the car's block asks for
/// it or `view_cleared` is set, and draws the car (`race::draw_car_shined`) with all parts shown
/// (`group_mask` -1), texture `KindSlots.kind_first_slots[202]`, render state 0x14 = 2,
/// `polygon_vertex_count` 4, then the marks' overlays (`race::draw_marks_overlays`). Refuses
/// (before anything) the car without its block.
pub fn draw_own_car(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    system: &crate::system::owner::System,
    gfx: &crate::gfx::owner::Gfx,
    cheats: &crate::menu::cheats_store::Cheats,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Logic in `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    if ctx.reads.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    let number = ctx.reads.view.drawn_view_car(ctx.view.drawn_view(race))?;
    if !ctx.reads.work.has_car_block(car_number(number) as usize) {
        return Err(BadInput("the drawn car without its block").into());
    }
    let car = ctx.reads.work.block_number(car_number(number) as usize)?;
    ctx.view.vars.drawn_car.set(number);
    ctx.view.state.borrow_mut().drawn_car = number;
    // The state bits are the view's own; the rest, the car as seen.
    if !seen_inside(ctx.view.cars.state_of(car)?.get()) {
        return Ok(());
    }
    let block = ctx.reads.frame_cars.car(car)?;
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
    ctx.scene
        .scratch
        .front_end_polygon()
        .polygon_vertex_count
        .set(4);
    draw_marks_overlays(ctx, gfx, cheats, effects)
}

/// The integer divided by `a`, then multiplied by `b`.
fn int_over_times(
    i: u32,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> f32 {
    use crate::math::x87_float::{int, ld, stf};
    stf(int(i as i32) / ld(a) * ld(b))
}

/// Runs the marks of a view when the marks are on (`view_marks_on`): chooses which views' marks
/// show (`other_marks_shown` the other view's, `own_marks_shown`; none in the replay
/// `scene_switch`, not the other with `still`), sets the view's marks layout from `view_layouts`
/// and the view car's turn into `view_place`, shares the cars (`drive::share_all_cars`), sets
/// every car record's first word, and sets the screen's centre for the layout (width / `k800` x
/// `view_scale_x`, height / `k600` x `view_scale_y`) while it runs `effects::frame_marks` (part 1)
/// and, unless `drawing`, `effects::run_frame` (part 2); the centre is put back after. Refuses
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
    // Logic in `rc2k_core::race::view_steps::{check_view_marks, marks_shown, layout_index,
    // first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    if ctx.reads.switches.view_marks_on.get() == 0 {
        return Ok(());
    }
    let view = ctx.view.drawn_view(race);
    let records = race.car_records()?;
    check_view_marks(
        u32::from(race.shared.view_car(view as usize).is_some()),
        |b| u32::from(records.get(b as usize).is_some()),
        || effects.setup.settings.is_set.get(),
    )?;
    let (other, own) = marks_shown(
        view,
        ctx.reads.vars.scene_switch.get(),
        race.physics.split_tables().still,
    );
    race.effects_settings().other_marks_shown.set(other);
    race.effects_settings().own_marks_shown.set(own);
    let layout_at = layout_index(view, ctx.reads.vars.split.get());
    // The marks' layout number: the two-player layouts' table has none for a larger split's views.
    let layout = race
        .frame
        .frame_work
        .get()
        .view_layouts
        .get(layout_at as usize)
        .map_or(0, crate::memory::session::PlainDword::get);
    race.effects_settings().marks_layout.set(layout);
    crate::math::cells::copy_cells(
        &ctx.reads.car_blocks.cars[ctx.reads.view.view_number(view as usize)?].whole_matrix,
        &race.frame.matrices.get().view_place,
        16,
    );
    crate::drive::share_all_cars::share_all_cars(
        race,
        sound,
        scene,
        crate::drive::set_listener::ListenerView::of(ctx.view, race),
        ctx.reads.net_race,
    )?;
    let first = first_word(ctx.reads.vars.run_kind.get());
    for b in 0..CAR_RECORDS {
        race.car_record(b as usize)?.streaks_on.set(first);
    }
    race.car_record(view as usize)?.streaks_on.set(1);
    let centre = (
        ctx.view.polygon.window.view_centre[0].get(),
        ctx.view.polygon.window.view_centre[1].get(),
    );
    // The effects project through the front end's window (a copy of the view's, made before the
    // view's overlays), so a part's middle goes there.
    let effects_window = &scene.scratch.front_end_polygon().window;
    let effects_centre = (
        effects_window.view_centre[0].get(),
        effects_window.view_centre[1].get(),
    );
    let part_split = ctx.reads.view.players.get() > 2;
    if part_split {
        // A part of a larger split screen: the effects are placed about the part's own middle,
        // in screen pixels, and cut to the part (the two-player layouts' table has no entry for
        // the third and fourth views).
        let window = &ctx.view.polygon.window;
        for axis in 0..2 {
            let middle = window.view_offset[axis].get() + window.view_size[axis].get() / 2;
            effects_window.view_centre[axis].set(middle as f32);
        }
        crate::effects::clip_quad::cut_to_part(effects, true);
    } else {
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
    }
    // No register is read or left.
    let done = crate::effects::frame_marks::frame_marks(race, scene, effects).and_then(|_| {
        if race.frame.frame_work.get().drawing.get() == 0 {
            crate::effects::run_frame::run_frame(
                &ctx.part.vertices,
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
    if part_split {
        crate::effects::clip_quad::cut_to_part(effects, false);
        effects_window.view_centre[0].set(effects_centre.0);
        effects_window.view_centre[1].set(effects_centre.1);
    }
    done
}

/// The same clear as `d3d::clear_viewport` (flag 2) over `cleared_rectangle`, unless
/// `v3_driver_fix` is not 0.
fn clear_viewport_unless_held(
    draw: &crate::scene::draw_store::SceneDraw,
    part: &crate::scene::scratch_store::PartScratch,
    own: &[crate::memory::session::PlainDword; 4],
) {
    if draw.v3_driver_fix.get() == 0 {
        crate::d3d::device::clear_viewport(part, own);
    }
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::asm_test::{Rng, at_each_control_word, same};
    use crate::test_support::vectors::{Recorder, word};

    #[test]
    fn int_over_times_matches() {
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
