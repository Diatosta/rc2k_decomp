//! The race panel: the HUD values, the start lights and the sound spots shared with the audio side.

use crate::errors::{BadInput, Refused};
use crate::race::frame_view::car as car_view;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;

/// `race::car_label` (0x45B1B7): the panel's seven-byte label " CAR nn" for the neighbouring car,
/// `car_number`'s two digits (tens and units, each + '0' as a byte), or seven spaces when
/// `neighbour` is -1 (no car there). The label's eighth byte is kept.
pub fn car_label(label_at: &[Dword; 2], car_number: u32, neighbour: u32) {
    // Converted: `rc2k_core::race::start::car_label`.
    let label = rc2k_core::race::start::car_label(car_number, neighbour == u32::MAX);
    let kept = label_at[1].get().to_le_bytes()[3];
    label_at[0].set(u32::from_le_bytes([label[0], label[1], label[2], label[3]]));
    label_at[1].set(u32::from_le_bytes([label[4], label[5], label[6], kept]));
}

/// `race::split_colours` (0x45B16A): the first `count` of the car's `colours` set to 0xC (not
/// reached); then for each of the car block's split times passed (none when the number passed is 0
/// or negative) the colour 8 when the time is not above the one before (0x63 before the first; the
/// sign of the difference), else 4. Refuses (before anything) a count past the buffer's 0x40
/// dwords, and more splits passed than the block's 16 times (the original reads on into the block's
/// next fields).
pub fn split_colours(car: usize, colours: &[Dword; 64], count: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::frame::{check_split_colours, split_colours}`.
    use rc2k_core::race::frame::{SPLIT_UNREACHED, check_split_colours, split_colours};
    let block = car_view(car)?;
    let n = block.splits_passed.get();
    check_split_colours(count, n)?;
    if n as i32 > 0 && n as usize > block.split_times.len() {
        return Err(BadInput("more splits than a block's times").into());
    }
    for cell in &colours[..count as usize] {
        cell.set(SPLIT_UNREACHED);
    }
    if n as i32 <= 0 {
        return Ok(());
    }
    let splits: Vec<u32> = block.split_times[..n as usize]
        .iter()
        .map(crate::session::Dword::get)
        .collect();
    let wanted = split_colours(&splits);
    for (cell, &c) in colours.iter().zip(wanted.iter()) {
        cell.set(c);
    }
    Ok(())
}

/// `race::panel_players` (0x45ACDB): unless `RACE_VARS.run_kind` is set,
/// `race().frame.panel_and_overlay.panel.view_a.place` and
/// `race().frame.panel_and_overlay.panel.view_b.place` -1; with more than one player also
/// `race().frame.panel_and_overlay.panel.view_a.back` and
/// `race().frame.panel_and_overlay.panel.view_a.front` -1, and
/// `race().frame.panel_and_overlay.panel.view_a.place` /
/// `race().frame.panel_and_overlay.panel.view_b.place` the two view blocks' +0xFA4.
pub fn panel_players(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let frame = &race.frame;
    // Converted: `rc2k_core::race::frame::panel_players`.
    use rc2k_core::race::frame::{PanelPlayers, panel_players};
    {
        let step = panel_players(shared.vars.run_kind.get(), || shared.view.players.get());
        if step == PanelPlayers::Kept {
            return Ok(());
        }
        frame.panel_and_overlay.panel.view_a.place.set(u32::MAX);
        frame.panel_and_overlay.panel.view_b.place.set(u32::MAX);
        if step == PanelPlayers::One {
            return Ok(());
        }
        frame.panel_and_overlay.panel.view_a.back.set(u32::MAX);
        frame.panel_and_overlay.panel.view_a.front.set(u32::MAX);
        // Each view block's +0xFA4.
        let field = |n: usize| -> Result<u32, BadInput> {
            Ok(shared
                .view
                .view_car(n)
                .ok_or(BadInput("a view without its car block"))?
                .two_car_place
                .get())
        };
        frame.panel_and_overlay.panel.view_a.place.set(field(0)?);
        frame.panel_and_overlay.panel.view_b.place.set(field(1)?);
    }
    Ok(())
}

/// `race::lamps_wanted` (0x4536D7): `RACE_VIEW.lamps_wanted` = `g_lampCount`, with bit 0 set when a
/// race car has +0x25C and +0x84 both set. Refuses (before anything) more than eight cars (past
/// `g_carBlocks`).
pub fn lamps_wanted(
    shared: &RaceShared,
    lamps: &crate::race::lamp_work::LampWork,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::frame::{car_lit, lamps_wanted}`.
    use rc2k_core::race::frame::{car_lit, lamps_wanted};
    let cars: u32 = shared.view.race_cars.get();
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let mut any_lit = false;
    for n in 0..cars {
        let this_car = car_view(shared.work.block_number(n as usize)?)?;
        any_lit |= car_lit(this_car.driver_kind.get(), || this_car.lights_on.get());
    }
    lamps
        .lamps_wanted
        .set(lamps_wanted(lamps.lamp_count.get(), any_lit));
    Ok(())
}

/// The number of the view block the drawn view `n` names (`view_blocks`[]`; the original read
/// `RACE_VIEW.drawn_view`, now the view's own: `ViewScratch::drawn_view`); refuses an index
/// past the two and a view without its block.
pub(super) fn drawn_view(shared: &RaceShared, n: u32) -> Result<usize, Refused> {
    if n > 1 {
        return Err(BadInput("a view past the two").into());
    }
    Ok(shared.view.view_number(n as usize)?)
}

/// `race::store_view_value` (0x455528): the drawn view's block +0xFB8 = the view's sun on the
/// screen (`g_sunOnScreen`). Worked out at the scene's end (the block and the value), stored by the
/// game's thread in the views' order (`ViewOutput::store_view_value`).
pub fn view_value(
    race: &Race,
    sun: &crate::scene::polygon_work::ViewSun,
    view: u32,
) -> Result<(usize, u32), Refused> {
    Ok((drawn_view(&race.shared, view)?, sun.on_screen.get()))
}

/// The value into the view's block +0xFB8 (`view_value`).
pub fn store_view_value(race: &Race, block: usize, value: u32) -> Result<(), Refused> {
    race.car_blocks().cars[block].eased_target.set(value);
    Ok(())
}

/// `race::fetch_view_value` (0x45550E): the view's value (`ViewSun::view_out`, the original's
/// `FrameWork.view_out`) = the drawn view's block +0xFBC.
pub fn fetch_view_value(
    race: &Race,
    sun: &crate::scene::polygon_work::ViewSun,
    view: u32,
) -> Result<(), Refused> {
    let b = drawn_view(&race.shared, view)?;
    sun.view_out.set(car_view(b)?.eased_value.get());
    Ok(())
}

/// `race::share_sound_spots` (0x45C7B9): the shared count =
/// `race().frame.sound_spots.sound_spot_count`, and that many 0x14-byte spots copied from 0x6D1AF4
/// to 0x5165F0 (read by the sound side, `sound::match_spot_voices`). Refuses (before anything) more
/// than 36 (past both tables).
pub fn share_sound_spots(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let frame = &race.frame;
    // Converted: `rc2k_core::race::view::shared_spot_bytes`.
    let n: u32 = frame.sound_spots.sound_spot_count.get();
    let bytes = rc2k_core::race::view::shared_spot_bytes(n)?;
    sound.race.stage.get().shared_count.set(n);
    let from = frame.sound_spots.sound_spots.chunks(5);
    for (spot, d) in sound
        .race
        .stage
        .get()
        .shared_spots
        .iter()
        .zip(from)
        .take(bytes as usize / 0x14)
    {
        spot.id.set(d[0].get());
        for (p, c) in spot.place.iter().zip(&d[1..4]) {
            p.set_bits(c.get());
        }
        spot.kind.set(d[4].get());
    }
    Ok(())
}

/// `race::clear_point_records` (0x4500FF), on its variables: the
/// 0x1000 point records of 0x20 bytes at `g_pointRecords`: each first dword -1, or with `g_useSse`
/// set each first 16 bytes `LIGHT_WORK.sse_fill` (`movntps`); `part` the drawing part clearing
/// them (`scene::point_records::first_records_everywhere`).
pub fn clear_point_records(
    scene: &crate::scene::owner::Scene,
    part: &crate::workers::PartScratch,
) -> Result<(), BadInput> {
    let pattern: [u32; 4] = crate::session::read_bits(&scene.lights.work.sse_fill);
    let sse = scene.draw_switches.use_sse.get() != 0;
    for record in crate::scene::point_records::first_records_everywhere(part, 0x1000)? {
        if sse {
            for (cell, v) in record.iter().zip(pattern) {
                cell.set(v);
            }
        } else {
            record[0].set(u32::MAX);
        }
    }
    Ok(())
}

/// `race::object_bytes` (0x455263): for each of the 0x100 objects (records at each
/// `POLYGON_WORK.texture_infos` entry's +0x20, its first dword the kind, +0xE bit 0), out0 = the
/// record's +0xE bit 0, out1 = 1, out2 = the kinds' byte (`TEXTURE_PLACES.place_slot`) of the
/// record's kind, out3 = 0xFF, or with faces (`OBJECT_SLOTS.face_counts`, first
/// `OBJECT_SLOTS.first_face` in `g_faces`) the kinds' byte of the first face's +4 word. Outputs:
/// the view's four face tables (`race::view_course`; the original's `LEVEL_GRID.face_table_a` ...)
pub fn object_bytes(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    mine: &crate::race::view_course::ViewCourse,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::frame::object_bytes`.
    let kind = |n: u32| -> Result<u32, BadInput> {
        scene
            .textures
            .places
            .place_slot
            .get(n as usize)
            .map(crate::session::Dword::get)
            .ok_or(BadInput("an object kind past the kinds' table"))
    };
    for n in 0..0x100usize {
        let record = &race.polygon_tables().texture_infos[n].words[8..];
        let flags = record[3].get().to_le_bytes()[2];
        let record_kind = kind(record[0].get())?;
        let face_kind = if scene.objects.slots.face_counts[n].get() != 0 {
            let first = scene.objects.slots.first_face[n].get() as usize;
            let face = race
                .faces()
                .faces
                .get(first)
                .ok_or(BadInput("an object's first face past the faces"))?
                .get();
            // The face is a place of `scene::object_faces`.
            let word = crate::scene::object_faces::bytes(face.wrapping_add(4), 2)?;
            let word = u16::from_le_bytes([word[0], word[1]]);
            Some(kind(word as u32)?)
        } else {
            None
        };
        let bytes = rc2k_core::race::frame::object_bytes(flags, record_kind, face_kind);
        for (own, b) in mine.tables().into_iter().zip(bytes) {
            own.set(n, b);
        }
    }
    Ok(())
}

/// One view's half of `race::panel_values`: the car block at `car`'s values into the panel
/// fields (the rounded engine speed, the engine sound speed, the gear, the distance, the
/// stage time x 10, the split gaps, and for the first view also its front and back gaps and
/// the two last ones).
pub(super) fn panel_half(race: &Race, car: usize, first: bool) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{distance_shown, shown_whole, travelled};
    let panel = &race.frame.panel_and_overlay.panel;
    // The two views' records have the same fields in two types.
    macro_rules! shown_cells {
        ($v:expr) => {
            [
                &$v.whole,
                &$v.speed,
                &$v.gear,
                &$v.place,
                &$v.time,
                &$v.split2,
                &$v.negated,
            ]
        };
    }
    let (cells, distance) = if first {
        (shown_cells!(panel.view_a), &panel.distance[0])
    } else {
        (shown_cells!(panel.view_b), &panel.distance[1])
    };
    // The shown engine speed is the drawing's own (`race::car_parts`), in the block.
    let whole = shown_whole(race.car_blocks().cars[car].shown_engine_speed.get());
    let car = car_view(car)?;
    cells[0].set(whole as u32);
    cells[1].set(car.engine_sound_speed_int.get());
    cells[2].set(car.gear.get());
    let at =
        |n: u32| crate::race::checkpoints_store::distance(race.checkpoints(), n as usize).get();
    let from: u32 = race.physics.split_tables.checkpoints[0].get();
    let d = travelled(at(car.checkpoint.get()), at(from));
    let (unit, miles) = (
        race.course.numbers.k65536.get(),
        race.course.numbers.metres_in_mile.get(),
    );
    let shown = distance_shown(d as i32, unit, miles).to_bits();
    distance.set(shown);
    cells[4].set(car.stage_time.get().wrapping_mul(10));
    cells[3].set(car.split_gaps[0].get());
    // The negated gap: `-(split_gaps[1])`.
    cells[6].set(car.split_gaps[1].get().wrapping_neg());
    cells[5].set(car.split_gaps[2].get());
    if first {
        panel.view_a.front.set(car.split_gaps[3].get());
        panel.view_a.back.set(car.split_gaps[4].get());
        panel.split_5.set(car.split_gaps[5].get());
        panel.split_6.set(car.split_gaps[6].get());
    }
    Ok(())
}

/// `race::panel_values` (0x45AB5A), converted (`set_up_panel`'s too; the exact x87 version went
/// once its one Rust caller converted): both view blocks' values into the panel (see `panel_half`).
// Converted: `rc2k_core::race::panel::{shown_whole, travelled, distance_shown}`.
pub fn panel_values_plain(race: &Race) -> Result<(), BadInput> {
    {
        panel_half(race, race.shared.view.view_number(0)?, true)?;
        panel_half(race, race.shared.view.view_number(1)?, false)?;
    }
    Ok(())
}

/// `race::wake_part_sounds` (0x45D1EA): for each race car (car 0 at least: the loop tests at its
/// end) and each of its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 = 1; then, only
/// when +0x700 was exactly 10, and when `race().frame.frame_work.listed_car_sounds[g_currentCar]`
/// is not -1, the car's sound object (`g_raceSoundObjects[car & 7]`) +0x264 = 1 and +0x10 = 1.
/// Refuses (before anything) more than eight cars, a null block or object it would write, and
/// `g_currentCar` past 7 when it is read.
pub fn wake_part_sounds(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let cars = shared.view.race_cars.get().max(1);
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let current: u32 = shared.work.current_car.get();
    // The plan: (block slot to set, whether to wake the object) by car.
    let mut plan = Vec::new();
    for car in 0..cars {
        let object = crate::sound::store::SoundObject::number(car & 7)?;
        let mut wake = false;
        let block = shared.work.block_car(car as usize, "a null car block")?;
        // Every flag set: each slot is `Left` whatever its count, nothing to plan (the flags only
        // go 0 -> 1, `part_woken` has no other writer).
        if block.part_woken.iter().all(|flag| flag.get() != 0) {
            continue;
        }
        for slot in 0..6usize {
            // Converted: `rc2k_core::race::lights::part_wake`.
            use rc2k_core::race::lights::{PartWake, part_wake};
            let count = block.part_wake_counts[slot].get();
            let part = part_wake(block.part_woken[slot].get(), count);
            if part == PartWake::Left {
                continue;
            }
            plan.push(&block.part_woken[slot]);
            if part != PartWake::WokenNow {
                continue;
            }
            if current > 7 {
                return Err(BadInput("g_currentCar past the eight").into());
            }
            let view: u32 = race.frame.frame_work.listed_car_sounds[current as usize].get();
            if view != u32::MAX {
                wake = true;
            }
        }
        if wake {
            plan.push(&object.slot_c.start);
            plan.push(&object.new_revs);
        }
    }
    for cell in plan {
        cell.set(1);
    }
    Ok(())
}

/// `race::panel_neighbours` (0x45AD36): in mode 2 (`RACE_VARS.run_kind`), the first view's place
/// (+0x654) and its neighbours into the panel: `race().frame.panel_and_overlay.panel.view_a.place`
/// the place, `race().frame.panel_and_overlay.panel.view_a.time` +0x8C x 10,
/// `race().frame.panel_and_overlay.panel.view_a.back` the car behind's +0x24 (-1 at the last place,
/// `race().frame.panel_and_overlay.panel.view_a.back`), its label at 0x6D1838 (`car_label` of
/// `RACE_TABLES.arcade_car_numbers[place]`), `race().frame.panel_and_overlay.panel.view_a.split2`
/// +0x634, `race().frame.panel_and_overlay.panel.view_a.front` the car ahead's +0x24 (-1 at place
/// 1), its label at 0x6D1844 (`RACE_TABLES.number_before_first[place]`),
/// `race().frame.panel_and_overlay.panel.view_a.negated` -(+0x630). Refuses (before anything) a
/// neighbour's car number past 7 or a null block.
pub fn panel_neighbours(race: &Race) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    if shared.vars.run_kind.get() != 2 {
        return Ok(());
    }
    let a = car_view(shared.view.view_number(0)?)?;
    let place = a.mode2_place.get();
    // The place indexes unchecked, as the original does: the cars by place start at
    // `computer_ease_speed` (slot 0, the float 30.0) and run on into `arcade_car_kinds`; the
    // numbers' `number_before_first` is the entry at place -1.
    let on = |first: &Dword, n: u32| {
        crate::session::read_on_in(
            &shared.tables,
            std::slice::from_ref(first),
            n as usize,
            "a place table",
        )
    };
    let block_of = |car: u32| -> Result<_, Refused> {
        if car > 7 {
            return Err(BadInput("a neighbour's car past the eight").into());
        }
        if !shared.work.has_car_block(car as usize) {
            return Err(BadInput("a neighbour's block is null").into());
        }
        Ok(car_view(shared.work.block_number(car as usize)?)?)
    };
    let behind = block_of(on(
        &shared.tables.computer_ease_speed,
        place.wrapping_add(1),
    )?)?;
    let ahead = block_of(on(&shared.tables.computer_ease_speed, place)?)?;
    frame.panel_and_overlay.panel.view_a.place.set(place);
    frame
        .panel_and_overlay
        .panel
        .view_a
        .time
        .set(a.time_left.get().wrapping_mul(10));
    // Converted: `rc2k_core::race::frame::neighbour_number` (and the labels'
    // `rc2k_core::race::start::car_label`).
    use rc2k_core::race::frame::neighbour_number;
    let number = behind.car_kind.get();
    let back = neighbour_number(place == shared.tables.arcade_cars.get(), number);
    frame.panel_and_overlay.panel.view_a.back.set(back);
    car_label(
        &frame.panel_and_overlay.overlay.label_behind,
        on(&shared.tables.arcade_car_numbers[0], place)?,
        back,
    );
    frame
        .panel_and_overlay
        .panel
        .view_a
        .split2
        .set(a.pace_gaps_eased[1].get());
    let front = neighbour_number(place == 1, ahead.car_kind.get());
    frame.panel_and_overlay.panel.view_a.front.set(front);
    car_label(
        &frame.panel_and_overlay.overlay.label_ahead,
        on(&shared.tables.number_before_first, place)?,
        front,
    );
    frame
        .panel_and_overlay
        .panel
        .view_a
        .negated
        .set(a.pace_gaps_eased[0].get().wrapping_neg());
    Ok(())
}

/// `race::panel_motion` (0x45AE1D), converted (`set_up_panel`'s too): for each view block, +0xAE8
/// and +0xAEC x 0.5 / 65536 (floats) and the heading, (-(+0x9C0)) >> 14 x pi x 0.5 / 65536, into
/// the panel (`race().frame.panel_and_overlay.panel.view_a.motion[0]`.. for the first,
/// `race().frame.panel_and_overlay.panel.view_b.motion[0]`.. for the second). The original's stores
/// to `g_scratch` (the heading turn, `g_sceneClock` less the shown-since clock and 800) are not
/// kept: nothing read them.
// Converted: `rc2k_core::race::panel::{motion, heading_turn, heading}`.
pub fn panel_motion_plain(race: &Race) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{heading, heading_turn, motion};
    {
        // NUMBERS.half (0.5), NUMBERS.one_over_65536, NUMBERS.k65536 (65536.0)
        let (half, small, big) = (
            race.course.numbers.half.get(),
            race.course.numbers.one_over_65536.get(),
            race.course.numbers.k65536.get(),
        );
        for (view, out) in [
            (0, &race.frame.panel_and_overlay.panel.view_a.motion),
            (1, &race.frame.panel_and_overlay.panel.view_b.motion),
        ] {
            let b = race.shared.view.view_number(view)?;
            let car = &race.car_blocks().cars[b];
            out[0].set(motion(car.map_picture_place[0].get() as i32, half, small).to_bits());
            out[1].set(motion(car.map_picture_place[1].get() as i32, half, small).to_bits());
            let turn = heading_turn(car_view(b)?.car_heading.get());
            out[2].set(heading(turn as i32, half, big).to_bits());
        }
    }
    Ok(())
}

/// `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (but 0x6D14B2) 0; for
/// each view block, `race().frame.panel_and_overlay.panel.marks[0].base` /
/// `race().frame.panel_and_overlay.panel.marks[1].base` = 400 + (+0xFA0), and its first +0xF98 (at
/// most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6.
pub fn panel_marks(race: &Race) -> Result<(), BadInput> {
    let frame = &race.frame;
    // Converted: `rc2k_core::race::frame::view_marks`.
    {
        for view in &frame.panel_and_overlay.panel.marks {
            for mark in &view.marks {
                mark.set(0);
            }
        }
        for (view, shown) in frame.panel_and_overlay.panel.marks.iter().enumerate() {
            let b = race.shared.view.view_number(view)?;
            let offset = car_view(b)?.car_event_position.get();
            let count = car_view(b)?.car_event_count.get();
            let codes = &car_view(b)?.car_event_codes;
            let past = std::cell::Cell::new(false);
            let m = rc2k_core::race::frame::view_marks(offset, count, |k| {
                codes.get(k as usize).map_or_else(
                    || {
                        past.set(true);
                        0
                    },
                    crate::session::Dword::get,
                )
            });
            if past.get() {
                return Err(BadInput("a view block's mark past its eight"));
            }
            shown.base.set(m.base);
            if count != 0 {
                for k in 0..m.shown {
                    shown.marks[k as usize].set(m.marks[k as usize]);
                }
            }
        }
    }
    Ok(())
}

/// `race::countdown_lights` (0x45B008): one view's start lights: `view` the view's light fields,
/// `block` the view's car block, `flag` where a "step" flag goes. See the header for the steps.
pub fn countdown_lights(
    frame: &Frame,
    view: &crate::race::panel_store::Lights,
    block: usize,
    flag: &Dword,
) -> Result<(), BadInput> {
    {
        let (lit, width, alpha, drop) = (&view.lit, &view.width, &view.alpha, &view.drop);
        width.set(0);
        drop.set(0);
        flag.set(0);
        if car_view(block)?.false_start_count.get() != 0 {
            return Ok(());
        }
        // Converted: `rc2k_core::race::start::lights`.
        use rc2k_core::race::start::lights;
        let Some(l) = lights(
            car_view(block)?
                .run_ticks
                .get()
                .wrapping_sub(frame.panel_and_overlay.overlay.countdown_start.get()),
        ) else {
            return Ok(());
        };
        frame
            .panel_and_overlay
            .overlay
            .start_lights_lit
            .set(l.shown);
        lit.set(l.lit);
        if l.whole {
            flag.set(1);
        }
        alpha.set(l.alpha);
        width.set(l.width);
        drop.set(l.drop);
    }
    Ok(())
}

/// `race::both_countdown_lights` (0x45AFDB): `countdown_lights` for the first view and the second
/// (fields 0x10, `race().frame.panel_and_overlay.overlay.forget_flag_1`).
pub fn both_countdown_lights(race: &Race) -> Result<(), BadInput> {
    let frame = &race.frame;
    {
        for (view, flag) in [
            (0, &frame.panel_and_overlay.overlay.forget_flag_0),
            (1, &frame.panel_and_overlay.overlay.forget_flag_1),
        ] {
            countdown_lights(
                frame,
                &frame.panel_and_overlay.panel.lights[view],
                race.shared.view.view_number(view)?,
                flag,
            )?;
        }
    }
    Ok(())
}

/// The two views' colour buffers for `split_colours`:
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_a`,
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_b`.
pub fn colour_buffers(frame: &Frame) -> [&'_ [Dword; 64]; 2] {
    [
        &frame.panel_and_overlay.overlay.overlay_part_kinds_a,
        &frame.panel_and_overlay.overlay.overlay_part_kinds_b,
    ]
}

/// `split_colours`' refusal, for both views, before anything is written.
pub fn check_split_colours(race: &Race) -> Result<(), Refused> {
    let fill: u32 = race.physics.split_tables.split_colour_count.get();
    for view in 0..2 {
        let n = car_view(race.shared.view.view_number(view)?)?
            .splits_passed
            .get();
        rc2k_core::race::frame::check_split_colours(fill, n)?;
    }
    Ok(())
}

/// `race::both_split_colours` (0x45B147): `split_colours` for the first view's block into
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_a`, then the second's into
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_b`. Refuses (before anything) as
/// `split_colours` would for either.
pub fn both_split_colours(race: &Race) -> Result<(), Refused> {
    check_split_colours(race)?;
    for (view, buffer) in (0..2).zip(colour_buffers(&race.frame)) {
        split_colours(
            race.shared.view.view_number(view)?,
            buffer,
            race.physics.split_tables.split_colour_count.get(),
        )?;
    }
    Ok(())
}

/// `panel_neighbours`' refusal, before anything is written.
pub(super) fn check_panel_neighbours(shared: &RaceShared) -> Result<(), Refused> {
    if shared.vars.run_kind.get() != 2 {
        return Ok(());
    }
    let place = car_view(shared.view.view_number(0)?)?.mode2_place.get();
    // The cars by place (from `computer_ease_speed`, see `panel_neighbours`) at the place and the one after.
    let first = std::slice::from_ref(&shared.tables.computer_ease_speed);
    for n in [place, place.wrapping_add(1)] {
        let car = crate::session::read_on_in(&shared.tables, first, n as usize, "a place table")?;
        if car > 7 || !shared.work.has_car_block(car as usize) {
            return Err(BadInput("a neighbour's car past the eight or null").into());
        }
    }
    Ok(())
}

/// `race::set_up_panel` (0x45AB36): `panel_values`, `panel_neighbours`, `panel_players`,
/// `panel_motion`, `panel_marks`, `both_countdown_lights`, `both_split_colours`. Refuses (before
/// anything) what `panel_neighbours` or `both_split_colours` would.
pub fn set_up_panel(race: &Race) -> Result<(), Refused> {
    check_panel_neighbours(&race.shared)?;
    check_split_colours(race)?;
    // Converted: its float callees' `_plain` versions.
    panel_values_plain(race)?;
    panel_neighbours(race)?;
    panel_players(race)?;
    panel_motion_plain(race)?;
    panel_marks(race)?;
    both_countdown_lights(race)?;
    both_split_colours(race)
}

/// `race::reset_lamps` (0x45487A): `g_lampCount`, the view's two sort lists' counts and
/// `race().frame.frame_work.flare_count` 0, then `race::object_bytes`.
pub fn reset_lamps(ctx: &crate::race::view_scratch::DrawCtx) -> Result<(), BadInput> {
    let crate::race::view_scratch::DrawCtx {
        race, scene, view, ..
    } = *ctx;
    view.lamps.lamp_count.set(0);
    view.lists.depth_entry_count.set(0);
    view.lists.sort_entry_count.set(0);
    view.lamps.flare_count.set(0);
    object_bytes(race, scene, &view.course)
}

/// `race::hold_to_limit` (0x4650A9): `value` held to `RACE_WORK.patch_settings` (unsigned).
pub fn hold_to_limit(shared: &RaceShared, value: u32) -> u32 {
    // Converted: `rc2k_core::race::frame::hold_to_limit`.
    rc2k_core::race::frame::hold_to_limit(value, shared.work.patch_settings[0].get())
}
