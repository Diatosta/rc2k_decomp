//! `race::draw_race_scene` (0x45429B): the race frame's drawing, between the scene's begin
//! and end (`drive::race_frame` part 3). In turn:
//! - the cars shown chosen (`drive::choose_shown_cars`), 0x20 dwords at `unk_63DBB8` 0; the
//!   two view blocks' +0x220 into `dword_6D14F6` / `dword_6D14FA` (each 0 while its
//!   `dword_63DB74` / `dword_63DB78` is 0); the batches' counts, `dword_639960` / `64`,
//!   `g_carsDrawn` and `g_carsCulled` 0; bit 1 of `dword_609354` into `dword_62DAD0`, bit 0
//!   of `dword_609350` into `dword_62DACC`, bit 6 of the first view's flags into
//!   `dword_6D1502`; `dword_6415E0` the first view block's +0x6F0 when `qword_609344` is not
//!   0, else 0; `dword_5CDA8C` 1;
//! - the sound tick (eax 0xA), `g_shine` from `dword_609324`; inside `pusha` / `pushf` the
//!   filters and five render states (0x15 = 4, 0x1C = 1, 0x13 = 5, 0x14 = 6, 0x1B = 1);
//! - the sound tick (0xB), `dword_63967C` one more, the clock restarted into
//!   `dword_63DC38`, the full view's window, the cars' parts readied, the maps scrolled;
//!   `dword_6D1508` 0, the views' textures forgotten, the mud's frame; `dword_6415F8` 0, the
//!   kinds' textures assigned, the two textures of `unk_6D1482` / `unk_6D1492` (unless -1)
//!   wanted in kind 0xDC's first slot and the next; the places mapped, the slots' textures
//!   loaded; the sound tick (0xC); the panel set up unless `dword_64BB18`;
//! - the views: one (`dword_63DB98` 1: car A's, the scene, ticks 0xD and 0xE around
//!   `sub_455D38`, the overlays, tick 0xF, the mirror if the view's flags have bit 3), or
//!   two halves (`dword_6238A4` 1 side by side, else one above the other), each with its
//!   window (`dword_62408C` .. `dword_624080`, `scene::set_view_window`), its car, the
//!   scene, tick 0xD, the overlays, tick 0xF and its mirror;
//! - the first car loaded, both batches flushed, the sound spots found and shared, the
//!   parts' sounds woken, the sound tick (0x10), render states 0x11 and 0x12 = 2; the HUD's
//!   pass unless `dword_71C3E0` or `dword_64BB18`; the sound tick (0x11), the skidmarks laid,
//!   the name plates; the tyre change screen unless `dword_5C5CE0` is 2.
//!
//! In:  nothing (the original reads no register at entry; nor do its steps read one it
//!      gives them: each sets what it uses)
//! Out: nothing its callers (`drive::race_frame`, `race::show_replay_still`) read
//!
//! Refuses (before anything) a null view block, and a wanted texture's slot past
//! `g_slotWantedTextures`' 0x400; then what its steps refuse.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_shared::Shared;
use crate::scene::owner::scene;
use crate::system::owner::system;
const SLOTS: u32 = 0x400;

/// The two view blocks, or the refusal.
fn view_blocks(shared: &Shared) -> Result<[u32; 2], Refused> {
    let blocks: [u32; 2] = [shared.view.view_block(0), shared.view.view_block(1)];
    if blocks.contains(&0) {
        return Err(BadInput("a null view block").into());
    }
    Ok(blocks)
}

/// One view drawn: `car` (0 or 1: `race::load_car_a` / `_b`), its value fetched, its scene,
/// the sound tick (0xD), the platform's blit (the full view only), the tick (0xE, the full
/// view only), the overlays, the tick (0xF), and with the view's flags' bit 3 its mirror.
fn draw_view(race: &Race, car: usize, flags: usize, blit: bool) -> Result<(), Refused> {
    use crate::race::frame::load_view_car;
    use crate::sound::race_sound_tick::race_sound_tick;
    load_view_car(&race.shared, car)?;
    crate::race::frame_draw::fetch_view_value(race)?;
    crate::race::draw_view_scene::draw_view_scene(race)?;
    // Not the original's: the debug UI's probe points need this view.
    if car == 0 {
        crate::debug::imgui_sdl::view_drawn(&race.frame);
    }
    race_sound_tick(&race.frame)?; // 0xD
    if blit {
        crate::d3d::copy_screen_to_texture::copy_screen_to_texture(&race.frame);
        race_sound_tick(&race.frame)?; // 0xE
    }
    crate::race::draw_view_overlays::draw_view_overlays(race)?;
    race_sound_tick(&race.frame)?; // 0xF
    if race.frame.panel_and_overlay.panel.hud_flags[flags].get() & 8 != 0 {
        load_view_car(&race.shared, car)?;
        crate::race::draw_mirror_view::draw_mirror_view(race)?;
    }
    Ok(())
}

/// The Rust version (see the header).
pub fn draw_race_scene() -> Result<(), Refused> {
    let session = &system().session;
    let scene = scene();
    let race = race();
    let frame = &race.frame;
    let physics = &race.physics;
    use crate::scene::set_render_state::set_render_state;
    use crate::sound::race_sound_tick::race_sound_tick;
    let blocks = view_blocks(&race.shared)?;
    for (k, extra) in race
        .frame
        .panel_and_overlay
        .panel
        .lights
        .iter()
        .map(|l| &l.lit)
        .enumerate()
    {
        if extra.get() != u32::MAX && scene.textures.kinds.kind_first_slots[220 + k].get() >= SLOTS
        {
            return Err(BadInput("a wanted texture's slot past g_slotWantedTextures").into());
        }
    }
    crate::drive::choose_shown_cars::choose_shown_cars()?;
    crate::session::write_bits(&race.shared.view.view_cleared, [0u32; 0x20]);
    for (flags, block) in race
        .frame
        .panel_and_overlay
        .panel
        .hud_flags
        .iter()
        .zip(blocks)
    {
        flags.set(crate::race::trap::car_at(block)?.option_bits.get());
    }
    for (flags, on) in race
        .frame
        .panel_and_overlay
        .panel
        .hud_flags
        .iter()
        .zip(&race.shared.view.view_on)
    {
        if on.get() == 0 {
            flags.set(0);
        }
    }
    race.physics.ground.split_polygon_count.set(0);
    race.physics.ground.whole_polygon_count.set(0);
    for cell in [
        &race.frame.draw_batches.plain_word,
        &race.frame.draw_batches.plain_count,
        &race.frame.draw_batches.textured_word,
        &race.frame.draw_batches.textured_count,
    ] {
        cell.set(0);
    }
    race.frame.frame_work.cars_drawn.set(0); // g_carsDrawn
    race.frame.frame_work.cars_culled.set(0); // g_carsCulled
    scene
        .lights
        .work
        .marks_mask
        .set((race.course.stage_prefs.refresh_flags_a.get() & 2) >> 1);
    scene
        .lights
        .work
        .lamps_mask
        .set(race.course.stage_prefs.refresh_flags_b.get() & 1);
    race.frame
        .panel_and_overlay
        .panel
        .flat
        .set(race.frame.panel_and_overlay.panel.hud_flags[0].get() & 0x40);
    race.frame.frame_work.mirror_wanted.set(0); // dword_6415E0
    if race.course.stage_prefs.blit_cleared[0].get() | race.course.stage_prefs.blit_cleared[1].get()
        != 0
    {
        race.frame.frame_work.mirror_wanted.set(
            crate::race::trap::car_at(blocks[0])?
                .camera_mode_is_2_or_3
                .get(),
        );
    }
    scene.objects.tables.has_3d.set(1);
    race_sound_tick(frame)?; // 0xA
    race.course
        .course
        .shine
        .set(race.course.stage_prefs.shine_from.get());
    // pusha; pushf .. popf; popa
    crate::scene::set_filters::set_filters();
    for (value, state) in [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)] {
        set_render_state(value, state);
    }
    race_sound_tick(frame)?; // 0xB
    race.physics
        .ground
        .frames_drawn
        .set(race.physics.ground.frames_drawn.get().wrapping_add(1));
    race.shared
        .view
        .clock_base
        .set(crate::scene::restart_clock::restart_clock(
            crate::system::tick_count::tick_count(),
        ));
    crate::race::frame_views::full_view_window(physics, true);
    crate::race::car_parts::ready_car_parts(race)?;
    crate::race::map_scroll::scroll_maps(race)?;
    race.frame.panel_and_overlay.panel.texture_changes.set(0);
    crate::race::frame_views::forget_view_textures(race)?;
    crate::race::mud_frame::mud_frame(race)?;
    race.frame.frame_work.kind_textures_done.set(0);
    crate::scene::assign_kind_textures::assign_kind_textures()?;
    for (k, extra) in race
        .frame
        .panel_and_overlay
        .panel
        .lights
        .iter()
        .map(|l| &l.lit)
        .enumerate()
    {
        let texture = extra.get();
        if texture != u32::MAX {
            let slot = scene.textures.kinds.kind_first_slots[220 + k].get();
            crate::scene::texture_places_store::cell_at(
                &scene.textures.places.slot_wanted[0],
                slot,
            )
            .set(texture.wrapping_add(scene.textures.kinds.kind_first_textures[220].get()));
        }
    }
    crate::scene::map_texture_places::map_texture_places();
    crate::scene::load_slot_textures::load_slot_textures_in_game()?;
    race_sound_tick(frame)?; // 0xC
    if race.replay.finish.no_place.get() == 0 {
        crate::race::frame_draw::set_up_panel(race)?;
    }
    if race.shared.view.views.get() == 1 {
        race.shared.view.drawn_view.set(0);
        draw_view(race, 0, 0, true)?;
    } else {
        let side_by_side = race.shared.vars.split.bits() == 1;
        for view in 0..2u32 {
            race.shared.view.drawn_view.set(view);
            let (w, h) = (session.screen.width.get(), session.screen.height.get());
            // (width, x, height, y).
            if side_by_side {
                scene.drawing.scene.view_size[0].set_bits(w >> 1);
                scene.drawing.scene.view_offset[0].set_bits(if view == 0 { 0 } else { w >> 1 });
                scene.drawing.scene.view_size[1].set_bits(h);
                scene.drawing.scene.view_offset[1].set_bits(0);
            } else {
                scene.drawing.scene.view_size[0].set_bits(w);
                scene.drawing.scene.view_offset[0].set_bits(0);
                scene.drawing.scene.view_size[1].set_bits(h >> 1);
                scene.drawing.scene.view_offset[1].set_bits(if view == 0 { 0 } else { h >> 1 });
            }
            scene.drawing.scene.view_zoom.set_bits(0x100);
            crate::scene::set_view_window::set_view_window_plain(race);
            draw_view(race, view as usize, view as usize, false)?;
        }
    }
    crate::race::frame_views::load_first_car()?;
    use crate::race::frame_flush::{flush_batch, plain, textured};
    flush_batch(&plain(), false)?;
    flush_batch(&textured(), true)?;
    crate::race::sound_spots::find_sound_spots(race)?;
    crate::race::frame_draw::share_sound_spots()?;
    crate::race::frame_draw::wake_part_sounds(race)?;
    race_sound_tick(frame)?; // 0x10
    for state in [0x11, 0x12] {
        set_render_state(2, state);
    }
    if race.shared.work.in_credits.bits() == 0 && race.replay.finish.no_place.get() == 0 {
        crate::hud::overlay_pass::overlay_pass(crate::hud::owner::hud(), race)?;
    }
    race_sound_tick(frame)?; // 0x11
    crate::race::skid_laying::lay_skidmarks_if_on(race)?;
    crate::race::name_plates::update_name_plates(race)?;
    if race.course.numbers.replay_mode.bits() != 2 {
        crate::drive::tyre_change_screen::tyre_change_screen()?;
    }
    Ok(())
}
