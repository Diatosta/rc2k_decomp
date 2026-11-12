//! The race frame's views and small drawing steps under `race::draw_race_scene` (part R3): the full-screen and mirror windows, the flat triangle, the marks'
//! overlays, the first car loaded.

use crate::errors::{BadInput, Refused};
use crate::gfx::owner::gfx;
use crate::physics::car_store::Car;
use crate::race::effects_settings_store::EFFECTS_SETTINGS;
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
    window.view_size[0].set(session.screen.width.get());
    window.view_size[1].set(session.screen.height.get());
    window.view_offset[0].set(0);
    window.view_offset[1].set(0);
    window.view_zoom.set(0x100);
    crate::scene::set_view_window::set_view_window(
        race,
        scene,
        system,
        window,
        &race.frame.view_and_records.view_vars,
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
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    let (fog, poly, car, vars) = (&view.fog, &view.polygon, &view.car, &view.vars);
    let drawn_view = view.drawn_view(race);
    let session = &system.session;
    // Converted: `rc2k_core::race::view::{mirror_entry, mirror_window}`.
    use rc2k_core::race::view::{MirrorRect, mirror_entry, mirror_window};
    let n = mirror_entry(
        race.shared.vars.split.get(),
        drawn_view,
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
                    &race.frame.frame_work,
                    places,
                    k,
                    "the mirror places",
                ))
            },
            super::super::session::Dword::get,
        )
    };
    let Some(MirrorRect {
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
        race,
        scene,
        system,
        &poly.window,
        vars,
        MIRROR_REACH,
    );
    crate::race::frame_draw::set_up_fog(race, fog, poly, car)?;
    // (The original's 0 into `g_drawMode`: see `full_view_window`.)
    poly.view_mirrored.set(1);
    Ok(())
}

/// `race::draw_flat_triangle` (0x455912): three vertices at (1, 1, 0) with texture 0, 0,
/// colour 0 (the original's `fld1` / `fldz` stores), `g_polygonVertexCount` 3, drawn
/// (`scene::draw_fan`). Every register kept.
pub fn draw_flat_triangle(scene: &crate::scene::owner::Scene) {
    // 1.0 at x, y of each of the three vertices; 0.0 at the rest of their floats (z, rhw,
    // u, v) and the specular.
    for vertex in &crate::d3d::vertices_store::VERTICES.vertices[..3] {
        vertex.x.set(1.0f32);
        vertex.y.set(1.0f32);
        vertex.z.set_bits(0);
        vertex.u.set_bits(0);
        vertex.v.set_bits(0);
        vertex.rhw.set_bits(0);
        vertex.specular.set(0);
    }
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(3);
    crate::scene::draw_fan::draw_fan(scene, crate::scene::polygon_work::front_end());
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
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    shadow_refuses(&crate::effects::draw_shadow::first(effects), effects)?;
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
pub fn load_first_car(shared: &RaceShared) -> Result<(), Refused> {
    if !shared.work.has_car_block(0) {
        return Err(BadInput("car 0 without its block").into());
    }
    shared.work.current_car.set(0);
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
    // The surfaces' pixels by handle (kept, never read back).
    let from_pixels = crate::gfx::surface::handle()?;
    frame.frame_work.view_kept[0].set(from_pixels);
    let ebp = gfx_drawing.draw_surface.pitch.get();
    if !lock_texture(to)?.carry {
        let to_pixels = crate::gfx::surface::handle()?;
        frame.frame_work.view_kept[1].set(to_pixels);
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
            from_16_bit: system.session.screen.bits.get() == 0x10,
            to_16_bit: gfx_drawing.draw_surface.bits_per_pixel.get() == 0x10,
        };
        let mut surfaces = GameSurfaces {
            screen: &screen,
            format,
            from: crate::gfx::locked_pixels::Place::start(from_pixels),
            to: crate::gfx::locked_pixels::Place::start(to_pixels),
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
}

impl rc2k_core::race::view_steps::Surfaces for GameSurfaces<'_> {
    fn read16(&mut self, at: u32) -> u16 {
        self.from.read_or_end_at(at, "a surface's pixel")
    }
    fn read32(&mut self, at: u32) -> u32 {
        self.from.read_or_end_at(at, "a surface's pixel")
    }
    fn write16(&mut self, at: u32, v: u16) {
        self.to.write_or_end_at(at, v, "a surface's pixel");
    }
    fn write32(&mut self, at: u32, v: u32) {
        self.to.write_or_end_at(at, v, "a surface's pixel");
    }
    fn components(&mut self, pixel: u32) -> u32 {
        crate::gfx::pixel_to_components::pixel_to_components(self.screen, pixel)
    }
    fn pixel(&mut self, components: u32) -> u32 {
        crate::gfx::components_to_pixel::components_to_pixel(self.format, components)
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
    let (Ok(first), Ok(second)) = (shared.view.view_number(0), shared.view.view_number(1)) else {
        return Err(BadInput("a view without its car block").into());
    };
    let blocks = [first, second];
    let trap = Trap::new();
    let field = |b: usize, word: fn(&Car) -> u32| word(&race.car_blocks().cars[b]);
    // (the switch, the slot's variable) of each.
    let slot_of = |n: usize| &textures.kinds.kind_first_slots[n];
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
    let uploaded = input.devices.keys_now.get(0x18) != 0; // Devices.keys_now
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
            let counter = &race.car_blocks().cars[blocks[v]].notice_counter;
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
        // The platform's upload (then the slot forgotten): through the game, with the picture
        // and the slot.
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
    if race.shared.work.in_credits.is_set() {
        race.shared.vars.quad_colour[1].set(NIGHT_ALPHA);
        window_corners(ctx.view);
        race.shared.vars.quad_colour[0].set(NIGHT_COLOUR);
        return crate::scene::draw_tinted_quad::draw_tinted_quad(ctx);
    }
    let sun = ctx.view.polygon.sun.on_screen.get();
    let a = glare_alpha(fog.far.get());
    race.shared.vars.quad_colour[1].set(a);
    let mut done = Ok(());
    if a != 0 {
        window_corners(ctx.view);
        // The two glare bytes (`glare_bytes`, StagePrefs.glare_bytes[1]): the low bytes of two dwords.
        let byte = |k: usize| race.course.stage_prefs.glare_bytes[k].get() as u8;
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

/// `race::clear_view_depth` (0x44E9A0): with the device on (`d3d_drawing_on`) the clip
/// rectangle (`cleared_rectangle` ..) the whole screen and the depth cleared there
/// (`d3d::clear_viewport_unless_held`, the platform's `Clear`).
pub fn clear_view_depth(
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
    poly: &crate::scene::polygon_work::PolygonWork,
) {
    let session = &system.session;
    if scene.objects.tables.d3d_drawing_on.get() == 0 {
        return;
    }
    poly.cleared_rectangle[0].set(0);
    poly.cleared_rectangle[1].set(0);
    poly.cleared_rectangle[2].set(session.screen.width.get());
    poly.cleared_rectangle[3].set(session.screen.height.get());
    clear_viewport_unless_held(scene, &poly.cleared_rectangle);
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
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    // Converted: `rc2k_core::race::view_steps::{car_number, seen_inside, clears_depth}`.
    use rc2k_core::race::view_steps::{car_number, clears_depth, seen_inside};
    if race.shared.switches.scene_set_up.get() == 0 {
        return Ok(());
    }
    let number = race.shared.view.view_cars[ctx.view.drawn_view(race) as usize].get();
    if !race.shared.work.has_car_block(car_number(number) as usize) {
        return Err(BadInput("the drawn car without its block").into());
    }
    let car = race.shared.work.block_number(car_number(number) as usize)?;
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
        clear_view_depth(scene, system, &ctx.view.polygon);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    // `race::draw_car_shined` reads only the car's block.
    crate::race::car_draw::draw_car_shined(ctx, car)?;
    ctx.view.cars.group_mask.set(u32::MAX);
    crate::scene::set_texture::set_texture(scene, scene.textures.kinds.kind_first_slots[202].get());
    crate::scene::set_render_state::set_render_state(2, 0x14);
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set(4);
    draw_marks_overlays(ctx, effects)
}

/// `fild i; fdiv a; fmul b; fstp`.
fn int_over_times(
    i: u32,
    a: impl crate::math::x87_float::Loadable,
    b: impl crate::math::x87_float::Loadable,
) -> u32 {
    use crate::math::x87_float::{int, ld, st};
    st(int(i as i32) / ld(a) * ld(b))
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
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let (race, scene) = (ctx.race, ctx.scene);
    let session = &system.session;
    // Converted: `rc2k_core::race::view_steps::{check_view_marks, marks_shown,
    // layout_index, first_word}` (the centre's floats the x87's).
    use rc2k_core::race::view_steps::{
        CAR_RECORDS, check_view_marks, first_word, layout_index, marks_shown,
    };
    if race.shared.switches.view_marks_on.get() == 0 {
        return Ok(());
    }
    let view = ctx.view.drawn_view(race);
    let records = race.shared.work.car_records_resolved()?;
    check_view_marks(
        u32::from(race.shared.view.view_car(view as usize).is_some()),
        |b| u32::from(records.get(b as usize).is_some()),
        || effects.setup.settings.is_set.get(),
    )?;
    let (other, own) = marks_shown(
        view,
        race.shared.vars.scene_switch.get(),
        race.physics.split_tables.still.get(),
    );
    EFFECTS_SETTINGS.other_marks_shown.set(other);
    EFFECTS_SETTINGS.own_marks_shown.set(own);
    let layout_at = layout_index(view, race.shared.vars.split.get());
    let layout = race.frame.frame_work.view_layouts[layout_at as usize].get();
    EFFECTS_SETTINGS.marks_layout.set(layout);
    crate::math::cells::copy_cells(
        &crate::race::blocks_store::CAR_BLOCKS.cars[race.shared.view.view_number(view as usize)?]
            .whole_matrix,
        &race.frame.matrices.view_place,
        16,
    );
    crate::drive::share_all_cars::share_all_cars(
        race,
        scene,
        crate::drive::set_listener::ListenerView::of(ctx.view, race),
    )?;
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
        ctx.view.polygon.window.view_centre[0].bits(),
        ctx.view.polygon.window.view_centre[1].bits(),
    );
    let n = race.frame.frame_work.view_layouts[layout_at as usize].get() as usize;
    ctx.view.polygon.window.view_centre[0].set_bits(int_over_times(
        session.screen.width.get(),
        race.course.numbers.k800.get(),
        race.frame.frame_work.view_scale_x[n].get(),
    ));
    ctx.view.polygon.window.view_centre[1].set_bits(int_over_times(
        session.screen.height.get(),
        race.course.numbers.k600.get(),
        race.frame.frame_work.view_scale_y[n].get(),
    ));
    // Neither reads a register nor leaves one read after it (0x456464, 0x456472).
    let done = crate::effects::frame_marks::frame_marks(race, scene, effects).and_then(|_| {
        if race.frame.frame_work.drawing.get() == 0 {
            crate::effects::run_frame::run_frame(
                race,
                scene,
                effects,
                crate::effects::run_frame::EffectsView::of(ctx.view),
            )
        } else {
            Ok(())
        }
    });
    ctx.view.polygon.window.view_centre[1].set_bits(centre.1);
    ctx.view.polygon.window.view_centre[0].set_bits(centre.0);
    done
}

/// `d3d::clear_viewport_unless_held` (0x44E9EE): `d3d::clear_viewport` (0x44E9D9, the same `Clear` of
/// `cleared_rectangle` with flag 2) unless `v3_driver_fix` is not 0.
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
