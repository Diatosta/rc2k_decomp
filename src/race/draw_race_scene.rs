//! Draws the race frame, between the scene's begin and end. In turn: the cars shown are chosen
//! (`drive::choose_shown_cars`), `view_cleared` cleared and the two view blocks' option bits
//! (+0x220) copied into `hud_flags` and the next dword (each 0 while its `view_on` flag is 0); the
//! batches' counts, `whole_polygon_count` and the next, `cars_drawn` and `cars_culled` are set to
//! 0; bit 1 of `refresh_flags_a` goes into `marks_mask`, bit 0 of `refresh_flags_b` into
//! `lamps_mask`, bit 6 of the first view's flags into `flat`; `mirror_wanted` is the first view
//! block's flag (+0x6F0) when `blit_cleared` is not 0, else 0; `d3d_drawing_on` is 1. Then the
//! sound tick 0xA, `shine` from `shine_from`, the filters and five render states (0x15 = 4, 0x1C =
//! 1, 0x13 = 5, 0x14 = 6, 0x1B = 1). Then the sound tick 0xB, one more `frames_drawn`, the clock
//! restarted into `clock_base`, the full view's window, the cars' parts readied, the maps
//! scrolled; `texture_changes` 0, the views' textures forgotten, the mud's frame; the kinds'
//! textures assigned (`kind_textures_done` 0), the two textures of the panel's `lights` entries
//! (unless -1) wanted in kind 0xDC's first slot and the next; the places mapped, the slots'
//! textures loaded; the sound tick 0xC; the panel set up unless `no_place`. Then the views: one
//! (`views` 1: car A's, the scene, ticks 0xD and 0xE around `d3d::copy_screen_to_texture`, the
//! overlays, tick 0xF, the mirror if the view's flags have bit 3), or two halves (`split` 1 side
//! by side, else one above the other), each with its window (`view_size`, `view_zoom`,
//! `scene::set_view_window`), its car, the scene, tick 0xD, the overlays, tick 0xF and its mirror.
//! Last: the first car loaded, both batches flushed, the sound spots found and shared, the parts'
//! sounds woken, the sound tick 0x10, render states 0x11 and 0x12 = 2; the HUD's pass unless
//! `in_credits` or `no_place`; the sound tick 0x11, the skidmarks laid, the name plates; the tyre
//! change screen unless `replay_mode` is 2.
//!
//! Refused (before anything): a null view block, and a wanted texture's slot past
//! the 0x400 slots; then what its steps refuse.

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::view_scratch::{ViewScratch, filter_input};
use crate::scene::owner::Scene;
use crate::system::owner::System;
const SLOTS: u32 = 0x400;

/// The two view blocks, or the refusal.
fn view_blocks(shared: &RaceShared) -> Result<[usize; 2], Refused> {
    let (Ok(first), Ok(second)) = (
        shared.view.get().view_number(0),
        shared.view.get().view_number(1),
    ) else {
        return Err(BadInput("a null view block").into());
    };
    Ok([first, second])
}

/// The frame's owners a view's drawing takes.
#[derive(Clone, Copy)]
struct Owners<'a> {
    race: &'a Race,
    scene: &'a crate::scene::owner::Scene,
    system: &'a crate::system::owner::System,
    net: &'a crate::net::owner::Net,
    sound: &'static crate::sound::owner::Sound,
    audio: &'static crate::audio::owner::Audio,
    input: &'static crate::input::owner::Input,
}

impl<'a> Owners<'a> {
    /// The owners whose drawing stores a view reads (`workers::LentOwners`).
    fn lent(self) -> crate::workers::LentOwners<'a> {
        crate::workers::LentOwners {
            race: self.race,
            scene: self.scene,
            system: self.system,
            net: self.net,
        }
    }
}

/// Whether the view with `flags` has its mirror (the flags' bit 3).
fn mirrored(race: &Race, flags: usize) -> bool {
    race.frame.panel_and_overlay.get().panel.hud_flags[flags].get() & 8 != 0
        || crate::race::draw_mirror_view::always()
}

/// View `car`'s car loaded into its own state (`frame::load_view_car`).
fn load_car(
    race: &Race,
    scratch: &crate::workers::DrawingScratch,
    car: usize,
) -> Result<(), Refused> {
    crate::race::frame::load_view_car(
        race.shared.work.get(),
        race.shared.car_blocks(),
        race.shared.view.get(),
        car,
        scratch.main_view(car),
    )
}

/// A view's value fetched (`race().frame.frame_work.view_out`, the drawn view's): into the polygon
/// work of `view`, a player's scene's slot or its mirror's.
fn fetch_value(
    race: &Race,
    race_view: &crate::race::view_store::RaceView,
    view: &ViewScratch,
) -> Result<(), Refused> {
    crate::race::frame_draw::fetch_view_value(
        &race.frame.cars,
        race_view,
        &view.polygon.sun,
        view.drawn_view(race),
    )
}

/// A player's scene on `run`'s view (its slot's state) and part, with the texture filter's switches
/// it starts from (`filter_input`, taken before the parts), and what it leaves its overlays kept
/// (`race::scene_hand_off`).
fn view_scene(
    o: Owners<'_>,
    run: crate::workers::PartRun<'_>,
    filter: [u32; 3],
    reads: crate::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let crate::workers::PartRun {
        part, view, helper, ..
    } = run;
    view.take_vars(reads);
    let output = crate::race::draw_view_scene::draw_view_scene(
        o.race, o.scene, o.input, view, filter, part, reads, helper,
    )?;
    view.hand_off.keep_output(output);
    view.hand_off.keep(reads.matrices, part);
    Ok(())
}

/// What follows view `car`'s scene: the platform's blit (`blit`, the full view only), the overlays;
/// for the first view its sound spots, from its own camera and grid, and its turn for the race's
/// placed sounds.
fn view_rest(
    o: Owners<'_>,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    cheats: &crate::menu::cheats_store::Cheats,
    car: usize,
    blit: bool,
) -> Result<(), Refused> {
    let view = o.scene.scratch.main_view(car);
    // The overlays (the effects, the HUD's quads) draw in the front end's window: the view's.
    {
        use crate::cells::Cells;
        o.scene
            .scratch
            .front_end_polygon()
            .window
            .copy_from(&view.polygon.window);
    }
    // The debug UI's probe points need this view.
    if car == 0 {
        crate::debug::imgui_sdl::view_drawn(&o.race.frame, o.scene);
    }
    if blit {
        crate::d3d::copy_screen_to_texture::copy_screen_to_texture(
            &o.race.frame,
            o.scene,
            o.system.session.screen.get(),
        );
    }
    let ctx = crate::race::view_scratch::DrawCtx::new(
        o.race,
        o.scene,
        o.input,
        view,
        o.scene.scratch.game_scratch(),
        crate::workers::DrawReads::on_game_thread(o.lent()),
    );
    crate::race::draw_view_overlays::draw_view_overlays(
        &ctx, o.system, o.sound, gfx, cheats, effects,
    )?;
    if car == 0 {
        crate::race::sound_spots::find_sound_spots(o.race, view)?;
        o.race.shared.heard_turn.get().keep(&view.matrices);
    }
    Ok(())
}

/// View `car`'s mirror, on `run`'s view (the mirror's slot) and part.
fn view_mirror(
    o: Owners<'_>,
    car: usize,
    run: crate::workers::PartRun<'_>,
    reads: crate::workers::DrawReads<'_>,
) -> Result<(), Refused> {
    let crate::workers::PartRun {
        part, view: mirror, ..
    } = run;
    crate::race::frame::load_view_car(reads.work, reads.car_blocks, reads.view, car, mirror)?;
    let begun = crate::debug::timing::start();
    let output = crate::race::draw_mirror_view::draw_mirror_view(
        o.race, o.scene, o.input, mirror, part, reads,
    )?;
    mirror.hand_off.keep_mirror_drawn(output.is_some());
    mirror.hand_off.keep_output(output.unwrap_or_default());
    crate::debug::timing::stop(crate::debug::timing::Stage::Mirror, begun);
    Ok(())
}

/// What view `car`'s scene (or with `mirror` its mirror) hands on, applied on the game's thread
/// once it is drawn, in the views' order: its value into its block, and for the first player's
/// scene the race's reverb.
fn view_drawn(
    race: &Race,
    scene: &Scene,
    system: &System,
    audio: &crate::audio::owner::Audio,
    car: usize,
    mirror: bool,
) -> Result<(), Refused> {
    let view = if mirror {
        scene.scratch.mirror(car)
    } else {
        scene.scratch.main_view(car)
    };
    view.hand_on_vars(race);
    let output = view.hand_off.output();
    if output.view_matrices {
        crate::race::frame_draw::hand_on_view_matrices(race, &view.matrices);
    }
    // After a mirror that was drawn the front end's window (what the HUD, the effects and the
    // loading bar draw in) is the full screen again.
    if mirror && view.hand_off.mirror_drawn() {
        crate::race::frame_views::full_view_window(
            race,
            scene,
            system,
            &scene.scratch.front_end_polygon().window,
        );
    }
    output.store_view_value(race)?;
    if car == 0 && !mirror {
        output.publish_environment(audio);
    }
    Ok(())
}

/// One view drawn: `car` (0 or 1: `frame::load_view_car`), its value fetched, its scene, the
/// sound tick (0xD), the platform's blit (the full view only), the tick (0xE, the full view only),
/// the overlays, the tick (0xF), and with the view's flags' bit 3 its mirror.
fn draw_view(
    o: Owners<'_>,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    cheats: &crate::menu::cheats_store::Cheats,
    car: usize,
    flags: usize,
    blit: bool,
) -> Result<(), Refused> {
    load_car(o.race, &o.scene.scratch, car)?;
    fetch_value(
        o.race,
        o.race.shared.view.get(),
        o.scene.scratch.main_view(car),
    )?;
    fetch_value(
        o.race,
        o.race.shared.view.get(),
        o.scene.scratch.mirror(car),
    )?;
    // One after another on the game's thread: on its part's scratch, the stores read from their
    // owners.
    let part = o.scene.scratch.game_scratch();
    let reads = || crate::workers::DrawReads::on_game_thread(o.lent());
    let run = |view| crate::workers::PartRun {
        n: 0,
        part,
        view,
        helper: None,
    };
    let view = || -> Result<(), Refused> {
        view_scene(
            o,
            run(o.scene.scratch.main_view(car)),
            filter_input(&o.scene.scratch, car),
            reads(),
        )?;
        view_drawn(o.race, o.scene, o.system, o.audio, car, false)?;
        view_rest(o, gfx, effects, cheats, car, blit)
    };
    let mirror = || view_mirror(o, car, run(o.scene.scratch.mirror(car)), reads());
    if !mirrored(o.race, flags) {
        if crate::workers::car_part_wanted() {
            // The view as a part of its own, for its car to be drawn aside (`race::car_part`).
            let filter = filter_input(&o.scene.scratch, car);
            return crate::workers::draw_side_by_side(
                o.lent(),
                &[car],
                |run, reads| view_scene(o, run, filter, reads),
                |_| {
                    view_drawn(o.race, o.scene, o.system, o.audio, car, false)?;
                    view_rest(o, gfx, effects, cheats, car, blit)
                },
            );
        }
        return view();
    }
    // The mirror drawn beside its view's scene (`workers::draw_beside`, `workers::mirror_beside`),
    // each on its own state (`race::view_scratch`), the view's overlays once both have ended (they
    // crumple the cars' points, which the mirror reads), the mirror's changes to the stores they
    // share put over theirs as when drawn after them.
    if crate::workers::mirror_beside() {
        // (The front end's filter switches are read before the parts, on the game's thread.)
        let filter = filter_input(&o.scene.scratch, car);
        return crate::workers::draw_beside(
            o.lent(),
            [car, 2 + car],
            |run, reads| view_scene(o, run, filter, reads),
            |run, reads| view_mirror(o, car, run, reads),
            || {
                view_drawn(o.race, o.scene, o.system, o.audio, car, false)?;
                view_rest(o, gfx, effects, cheats, car, blit)
            },
            || view_drawn(o.race, o.scene, o.system, o.audio, car, true),
        );
    }
    view()?;
    mirror()?;
    view_drawn(o.race, o.scene, o.system, o.audio, car, true)
}

/// A piece of the players' views drawn beside each other: a player's scene or its mirror.
#[derive(Clone, Copy)]
enum Piece {
    Scene(usize),
    Mirror(usize),
}

/// The `players` players' views and their mirrors beside each other (`workers::draw_side_by_side`):
/// the first player's scene on the game's thread, its mirror and every other player's scene and
/// mirror each on a drawing thread, every part from the frame's state with its player's part of the
/// screen set up (`player`). Then, once all have ended, in the order the views are drawn in, the
/// parts' changes put back and each player's overlays drawn between its scene and its mirror, on
/// its scene's window and what the scene left (`race::scene_hand_off`): no overlays while a scene
/// draws (they crumple the cars' points). So every view's scene starts from the frame's state, not
/// from what the views before it left (the wake counts, the cars' crumpling and the sun the
/// overlays work out reach the other views a frame later, however many threads draw).
fn draw_players_beside(
    o: Owners<'_>,
    gfx: &crate::gfx::owner::Gfx,
    effects: &crate::effects::owner::Effects,
    cheats: &crate::menu::cheats_store::Cheats,
    players: usize,
    player: &(dyn Fn(u32, &ViewScratch, Option<crate::workers::DrawReads<'_>>) + Sync),
) -> Result<(), Refused> {
    for p in 0..players {
        load_car(o.race, &o.scene.scratch, p)?;
    }
    let own = |p: usize,
               view: &ViewScratch,
               window: Option<crate::workers::DrawReads<'_>>,
               race_view: &crate::race::view_store::RaceView| {
        player(p as u32, view, window);
        fetch_value(o.race, race_view, view)
    };
    let mut pieces = Vec::new();
    for p in 0..players {
        pieces.push(Piece::Scene(p));
        if mirrored(o.race, p) {
            pieces.push(Piece::Mirror(p));
        }
    }
    // The view slot each piece draws, and the filter switches each scene starts from (the front
    // end's, read here on the game's thread, before the parts).
    let slots: Vec<usize> = pieces
        .iter()
        .map(|piece| match *piece {
            Piece::Scene(p) => p,
            Piece::Mirror(p) => 2 + p,
        })
        .collect();
    let filters: Vec<[u32; 3]> = slots
        .iter()
        .map(|&slot| filter_input(&o.scene.scratch, slot))
        .collect();
    let part = |run: crate::workers::PartRun<'_>,
                reads: crate::workers::DrawReads<'_>|
     -> Result<(), Refused> {
        match pieces[run.n] {
            Piece::Scene(p) => {
                own(p, run.view, Some(reads), reads.view)?;
                view_scene(o, run, filters[run.n], reads)
            }
            Piece::Mirror(p) => {
                own(p, run.view, None, reads.view)?;
                view_mirror(o, p, run, reads)
            }
        }
    };
    crate::workers::draw_side_by_side(o.lent(), &slots, part, |n| match pieces[n] {
        Piece::Scene(p) => {
            o.race.shared.view.get().drawn_view.set(p as u32);
            own(
                p,
                o.scene.scratch.main_view(p),
                None,
                o.race.shared.view.get(),
            )?;
            o.scene.scratch.main_view(p).hand_off.put(o.race);
            view_drawn(o.race, o.scene, o.system, o.audio, p, false)?;
            view_rest(o, gfx, effects, cheats, p, false)
        }
        Piece::Mirror(p) => view_drawn(o.race, o.scene, o.system, o.audio, p, true),
    })
}

/// Draws the race frame (see the header).
pub fn draw_race_scene(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (input, system, scene) = (o.input, o.system, o.scene);
    let owners = Owners {
        race,
        scene,
        system,
        net: o.net,
        sound: o.sound,
        audio: o.audio,
        input,
    };
    let frame = &race.frame;
    // Drawn outside the views' parts: on the game's thread's part.
    let part = o.scene.scratch.game_scratch();
    use crate::scene::set_render_state::set_render_state;
    use crate::sound::race_sound_tick::race_sound_tick;
    let _cars = crate::race::frame_view::Held::take(race);
    let blocks = view_blocks(&race.shared)?;
    for (k, extra) in race
        .frame
        .panel_and_overlay
        .get()
        .panel
        .lights
        .iter()
        .map(|l| &l.lit)
        .enumerate()
    {
        if extra.get() != u32::MAX
            && scene.textures.kinds.get().kind_first_slots[220 + k].get() >= SLOTS
        {
            return Err(BadInput("a wanted texture's slot past g_slotWantedTextures").into());
        }
    }
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    crate::session::write_bits(&race.shared.view.get().view_cleared, [0u32; 0x20]);
    for (flags, block) in race
        .frame
        .panel_and_overlay
        .get()
        .panel
        .hud_flags
        .iter()
        .zip(blocks)
    {
        flags.set(race.frame.cars.car(block)?.option_bits.get());
    }
    for (flags, on) in race
        .frame
        .panel_and_overlay
        .get()
        .panel
        .hud_flags
        .iter()
        .zip(&race.shared.view.get().view_on)
    {
        if on.get() == 0 {
            flags.set(0);
        }
    }
    // The polygon counts of the four views' slots (read by none).
    for view in [
        o.scene.scratch.main_view(0),
        o.scene.scratch.main_view(1),
        o.scene.scratch.mirror(0),
        o.scene.scratch.mirror(1),
    ] {
        view.object.split_polygon_count.set(0);
        view.object.whole_polygon_count.set(0);
    }
    for cell in [
        &race.frame.draw_batches().plain_word,
        &race.frame.draw_batches().plain_count,
        &race.frame.draw_batches().textured_word,
        &race.frame.draw_batches().textured_count,
    ] {
        cell.set(0);
    }
    // `cars_drawn`, now counted in each view's own state.
    for slot in [
        o.scene.scratch.main_view(0),
        o.scene.scratch.main_view(1),
        o.scene.scratch.mirror(0),
        o.scene.scratch.mirror(1),
    ] {
        slot.cars.cars_drawn.set(0);
    }
    scene
        .lights
        .work
        .get()
        .marks_mask
        .set((race.course.stage_prefs.get().refresh_flags_a.get() & 2) >> 1);
    scene
        .lights
        .work
        .get()
        .lamps_mask
        .set(race.course.stage_prefs.get().refresh_flags_b.get() & 1);
    race.frame
        .panel_and_overlay
        .get()
        .panel
        .flat
        .set(race.frame.panel_and_overlay.get().panel.hud_flags[0].get() & 0x40);
    race.frame.frame_work.get().mirror_wanted.set(false);
    if race.course.stage_prefs.get().blit_cleared[0].get()
        | race.course.stage_prefs.get().blit_cleared[1].get()
        != 0
    {
        race.frame
            .frame_work
            .get()
            .mirror_wanted
            .set(race.frame.cars.car(blocks[0])?.camera_mode_is_2_or_3.get() != 0);
    }
    scene.objects.tables.get().d3d_drawing_on.set(1);
    // The frame's one sound turn (`sound::race_turn`, when 20 ms of the scene clock have passed
    // since the last): the clock moves only here (`restart_clock` above and in `show_frame` after
    // the present), so it is due at most once a frame. It hears the previous frame's sound spots.
    let begun = crate::debug::timing::start();
    race_sound_tick(frame, o)?;
    crate::debug::timing::stop(crate::debug::timing::Stage::SoundTick, begun);
    // `shine` from `shine_from`: each view's from it now (`race::draw_view_scene`).
    crate::scene::set_filters::set_filters(race, part);
    for (value, state) in [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)] {
        set_render_state(part, value, state);
    }
    race.physics
        .ground()
        .frames_drawn
        .set(race.physics.ground().frames_drawn.get().wrapping_add(1));
    race.shared
        .view
        .get()
        .clock_base
        .set(crate::scene::restart_clock::restart_clock(
            &race.shared,
            crate::system::tick_count::tick_count(&system.session.picture_clock),
        ));
    // The window of the whole screen: the front end's (the loading bar, the HUD, the effects) and
    // the views' (a view's own is set again before it is drawn).
    for window in [
        &o.scene.scratch.front_end_polygon().window,
        &o.scene.scratch.main_view(0).polygon.window,
        &o.scene.scratch.main_view(1).polygon.window,
    ] {
        crate::race::frame_views::full_view_window(race, scene, system, window);
    }
    // The level's draw distance and fog settings, once for the frame: the views and the mirrors
    // only read them.
    crate::race::frame_draw::draw_distance_settings(&race.course);
    crate::race::car_parts::ready_car_parts(race, scene)?;
    crate::race::map_scroll::scroll_maps(race, scene, o.gfx, &o.net.race_slot)?;
    race.frame
        .panel_and_overlay
        .get()
        .panel
        .texture_changes
        .set(0);
    crate::race::frame_views::forget_view_textures(race, scene, input, system, o.gfx)?;
    crate::race::mud_frame::mud_frame(race, scene, o.gfx, o.system)?;
    race.frame.frame_work.get().kind_textures_done.set(0);
    crate::scene::assign_kind_textures::assign_kind_textures(scene)?;
    for (k, extra) in race
        .frame
        .panel_and_overlay
        .get()
        .panel
        .lights
        .iter()
        .map(|l| &l.lit)
        .enumerate()
    {
        let texture = extra.get();
        if texture != u32::MAX {
            let slot = scene.textures.kinds.get().kind_first_slots[220 + k].get();
            crate::scene::texture_places_store::cell_at(
                scene.textures.places.get(),
                crate::scene::texture_places_store::PlacesTable::SlotWanted,
                slot,
            )?
            .set(texture.wrapping_add(scene.textures.kinds.get().kind_first_textures[220].get()));
        }
    }
    crate::scene::map_texture_places::map_texture_places(scene);
    crate::scene::load_slot_textures::load_slot_textures_in_game(&race.frame, scene)?;
    if race.replay.finish.get().no_place.get() == 0 {
        crate::race::frame_draw::set_up_panel(race)?;
    }
    if race.shared.view.get().views.get() == 1 {
        race.shared.view.get().drawn_view.set(0);
        draw_view(
            owners,
            o.gfx,
            o.race.effects.get(),
            &o.menus.play.cheats,
            0,
            0,
            true,
        )?;
    } else {
        let side_by_side = race.shared.vars.get().split.get() == 1;
        // A player's half of the screen. (The store's drawn view is set on the game thread only:
        // the parts read their slot's, `ViewScratch::drawn_view`.) With `reads` (the drawing stores
        // as the caller reads them) the view's window set; without, nothing.
        let player =
            |view: u32, scratch: &ViewScratch, reads: Option<crate::workers::DrawReads<'_>>| {
                // Only the view's own scene sets its window: its mirror and its overlays only read
                // it.
                let Some(reads) = reads else {
                    return;
                };
                let own = &scratch.polygon.window;
                let (w, h) = (reads.screen.width.get(), reads.screen.height.get());
                // (width, x, height, y).
                if side_by_side {
                    own.view_size[0].set(w >> 1);
                    own.view_offset[0].set(if view == 0 { 0 } else { w >> 1 });
                    own.view_size[1].set(h);
                    own.view_offset[1].set(0);
                } else {
                    own.view_size[0].set(w);
                    own.view_offset[0].set(0);
                    own.view_size[1].set(h >> 1);
                    own.view_offset[1].set(if view == 0 { 0 } else { h >> 1 });
                }
                own.view_zoom.set(0x100);
                crate::scene::set_view_window::set_view_window_plain(
                    reads.course(),
                    reads.scene_draw,
                    reads.screen,
                    own,
                    &reads.view_and_records.view_vars,
                    &reads.vars.scale_factor,
                );
            };
        if crate::workers::players_beside() {
            draw_players_beside(
                owners,
                o.gfx,
                o.race.effects.get(),
                &o.menus.play.cheats,
                2,
                &player,
            )?;
        } else {
            for view in 0..2u32 {
                race.shared.view.get().drawn_view.set(view);
                player(
                    view,
                    o.scene.scratch.main_view(view as usize),
                    Some(crate::workers::DrawReads::on_game_thread(owners.lent())),
                );
                draw_view(
                    owners,
                    o.gfx,
                    o.race.effects.get(),
                    &o.menus.play.cheats,
                    view as usize,
                    view as usize,
                    false,
                )?;
            }
        }
    }
    crate::race::frame_views::load_first_car(&race.shared)?;
    use crate::race::frame_flush::{flush_batch, plain, textured};
    flush_batch(o.scene, part, &race.frame, &plain(&race.frame), false)?;
    flush_batch(o.scene, part, &race.frame, &textured(&race.frame), true)?;
    crate::race::frame_draw::share_sound_spots(race, o.sound)?;
    crate::race::frame_draw::wake_part_sounds(race, o.sound)?;
    for state in [0x11, 0x12] {
        set_render_state(part, 2, state);
    }
    if !race.shared.work.get().in_credits.is_set() && race.replay.finish.get().no_place.get() == 0 {
        crate::hud::overlay_pass::overlay_pass(
            race.hud.get(),
            race,
            o.system,
            o.scene,
            o.gfx,
            o.net,
        )?;
    }
    crate::race::skid_laying::lay_skidmarks_if_on(race)?;
    crate::race::name_plates::update_name_plates(
        crate::strings::append_string_by_id::StringOwners::of(o),
        race,
        scene,
        o.gfx,
        o.strings,
    )?;
    if race.course.numbers.get().replay_mode.get() != 2 {
        crate::drive::tyre_change_screen::tyre_change_screen(race, input, system, scene)?;
    }
    Ok(())
}
