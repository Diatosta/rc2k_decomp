//! The race panel: the HUD values, the start lights and the sound spots shared with the audio side.

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use crate::race::owner_frame::Frame;
use crate::race::owner_shared::RaceShared;
use crate::race::tables_store::Listed;

/// The panel's seven-byte label " CAR nn" for the neighbouring car: `car_number`'s two digits (tens
/// and units, each + '0' as a byte), or seven spaces when `neighbour` is -1 (no car there). The
/// label's eighth byte is kept.
pub fn car_label(label_at: &[PlainDword; 2], car_number: u32, neighbour: u32) {
    let label = rc2k_core::race::start::car_label(car_number, neighbour == u32::MAX);
    let kept = label_at[1].get().to_le_bytes()[3];
    label_at[0].set(u32::from_le_bytes([label[0], label[1], label[2], label[3]]));
    label_at[1].set(u32::from_le_bytes([label[4], label[5], label[6], kept]));
}

/// Sets the first `count` of the car's `colours` to 0xC (not reached); then for each of the car
/// block's split times passed (none when the number passed is 0 or negative) the colour is 8 when
/// the time is not above the one before (0x63 before the first; the sign of the difference), else
/// 4 (rather than 8). Refused (before anything): a count past the buffer's 0x40 dwords, and more
/// splits passed than the block's 16 times.
pub fn split_colours(
    cars: &crate::race::frame_view::FrameCars,
    car: usize,
    colours: &[PlainDword; 64],
    count: u32,
) -> Result<(), Refused> {
    use rc2k_core::race::frame::{SPLIT_UNREACHED, check_split_colours, split_colours};
    let block = cars.car(car)?;
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
        .map(crate::memory::session::Dword::get)
        .collect();
    let wanted = split_colours(&splits);
    for (cell, &c) in colours.iter().zip(wanted.iter()) {
        cell.set(c);
    }
    Ok(())
}

/// Unless `run_kind` is set, sets `race().frame.panel_and_overlay.panel.view_a.place` and
/// `view_b.place` to -1; with more than one player also `view_a.back` and `view_a.front` to -1, and
/// `view_a.place` / `view_b.place` to the two view blocks' +0xFA4.
pub fn panel_players(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let frame = &race.frame;
    use rc2k_core::race::frame::{PanelPlayers, panel_players};
    {
        let step = panel_players(shared.vars.get().run_kind.get(), || {
            shared.view.get().players.get()
        });
        if step == PanelPlayers::Kept {
            return Ok(());
        }
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_a
            .place
            .set(u32::MAX);
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_b
            .place
            .set(u32::MAX);
        if step == PanelPlayers::One {
            return Ok(());
        }
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_a
            .back
            .set(u32::MAX);
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_a
            .front
            .set(u32::MAX);
        // Each view block's +0xFA4.
        let field = |n: usize| -> Result<u32, BadInput> {
            Ok(shared
                .view_car(n)
                .ok_or(BadInput("a view without its car block"))?
                .two_car_place
                .get())
        };
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_a
            .place
            .set(field(0)?);
        frame
            .panel_and_overlay
            .get()
            .panel
            .view_b
            .place
            .set(field(1)?);
    }
    Ok(())
}

/// Sets `lamps_wanted` = `lamp_count`, with bit 0 set when a race car has +0x25C and
/// +0x84 both set. Refused (before anything): more than eight cars (past `car_blocks`).
pub fn lamps_wanted(
    frame_cars: &crate::race::frame_view::FrameCars,
    work: &crate::race::work_store::RaceWork,
    race_view: &crate::race::view_store::RaceView,
    lamps: &crate::race::lamp_work::LampWork,
) -> Result<(), Refused> {
    use rc2k_core::race::frame::{car_lit, lamps_wanted};
    let cars: u32 = race_view.race_cars.get();
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let mut any_lit = false;
    for n in 0..cars {
        let this_car = frame_cars.car(work.block_number(n as usize)?)?;
        any_lit |= car_lit(this_car.driver_kind.get(), || this_car.lights_on.get());
    }
    lamps
        .lamps_wanted
        .set(lamps_wanted(lamps.lamp_count.get(), any_lit));
    Ok(())
}

/// The number of the view block the drawn view `n` names (`view_blocks[]`; the view's own
/// `ViewScratch::drawn_view` is read); refused: an index past the four
/// and a view without its block.
pub(super) fn drawn_view(
    race_view: &crate::race::view_store::RaceView,
    n: u32,
) -> Result<usize, Refused> {
    if n > 3 {
        return Err(BadInput("a view past the four").into());
    }
    Ok(race_view.view_number(n as usize)?)
}

/// Stores the drawn view's block +0xFB8 = the view's sun on the screen (`sun_on_screen`). The block
/// and the value are worked out at the scene's end and stored by the game's thread in the views'
/// order (`ViewOutput::store_view_value`).
pub fn view_value(
    race_view: &crate::race::view_store::RaceView,
    sun: &crate::scene::polygon_work::ViewSun,
    view: u32,
) -> Result<(usize, u32), Refused> {
    Ok((drawn_view(race_view, view)?, sun.on_screen.get()))
}

/// The value into the view's block +0xFB8 (`view_value`).
pub fn store_view_value(race: &Race, block: usize, value: u32) -> Result<(), Refused> {
    race.car_blocks().cars[block].eased_target.set(value);
    Ok(())
}

/// The view's value (`ViewSun::view_out`, `FrameWork.view_out`) = the drawn view's block +0xFBC.
pub fn fetch_view_value(
    cars: &crate::race::frame_view::FrameCars,
    race_view: &crate::race::view_store::RaceView,
    sun: &crate::scene::polygon_work::ViewSun,
    view: u32,
) -> Result<(), Refused> {
    let b = drawn_view(race_view, view)?;
    sun.view_out.set(cars.car(b)?.eased_value.get());
    Ok(())
}

/// Sets the shared count = `race().frame.sound_spots().sound_spot_count` and copies that many
/// 0x14-byte spots (read by the sound side, `sound::match_spot_voices`). Refused (before anything):
/// more than 36 (past both tables).
pub fn share_sound_spots(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let frame = &race.frame;
    let n: u32 = frame.sound_spots().sound_spot_count.get();
    let bytes = rc2k_core::race::view::shared_spot_bytes(n)?;
    sound.race.stage.get().shared_count.set(n);
    let from = frame.sound_spots().sound_spots.chunks(5);
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
            p.set(f32::from_bits(c.get()));
        }
        spot.kind.set(d[4].get());
    }
    Ok(())
}

/// Clears the point records, on its variables: the 0x1000 point records of 0x20 bytes at
/// `point_records`: each first dword -1; `part` is the drawing part clearing them
/// (`scene::point_records::first_records_everywhere`).
pub fn clear_point_records(
    scratch: &crate::scene::workers::DrawingScratch,
    part: &crate::scene::workers::PartScratch,
) -> Result<(), BadInput> {
    for record in crate::scene::point_records::first_records_everywhere(scratch, part, 0x1000)? {
        record[0].set(u32::MAX);
    }
    Ok(())
}

/// For each of the 0x100 objects (records at each `texture_infos` entry's +0x20, its
/// first dword the kind, +0xE bit 0): out0 = the record's +0xE bit 0, out1 = 1, out2 = the kinds'
/// byte (`place_slot`) of the record's kind, out3 = 0xFF, or with faces
/// (`face_counts`, first `first_face` in `faces`) the kinds' byte of the
/// first face's +4 word. Outputs: the view's four face tables (`race::view_course`;
/// `face_table_a` ...)
pub fn object_bytes(
    reads: crate::scene::workers::DrawReads<'_>,
    mine: &crate::race::view_course::ViewCourse,
) -> Result<(), BadInput> {
    let kind = |n: u32| -> Result<u32, BadInput> {
        reads
            .texture_places
            .place_slot
            .get(n as usize)
            .map(crate::memory::session::PlainDword::get)
            .ok_or(BadInput("an object kind past the kinds' table"))
    };
    for n in 0..0x100usize {
        let record = &reads.buffers.polygon_tables.texture_infos[n].words[8..];
        let flags = record[3].get().to_le_bytes()[2];
        let record_kind = kind(record[0].get())?;
        let face_kind = if reads.object_slots.face_counts[n].get() != 0 {
            let first = reads.object_slots.first_face[n].get() as usize;
            let face = reads
                .buffers
                .faces
                .faces
                .get(first)
                .ok_or(BadInput("an object's first face past the faces"))?
                .get();
            // The face is a place of `scene::object_faces`.
            let word = reads.stage.object_faces.bytes(face.wrapping_add(4), 2)?;
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

/// One view's half of `panel_values_plain`: the car block at `car`'s values into the panel fields
/// (the rounded engine speed, the engine sound speed, the gear, the distance, the stage time x
/// 10, the split gaps, and for the first view also its front and back gaps and the two last
/// ones).
pub(super) fn panel_half(race: &Race, car: usize, first: bool) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{distance_shown, shown_whole, travelled};
    let panel = &race.frame.panel_and_overlay.get().panel;
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
    let car = race.frame.cars.car(car)?;
    cells[0].set(whole as u32);
    cells[1].set(car.engine_sound_speed_int.get());
    cells[2].set(car.gear.get());
    let at = |n: u32| {
        crate::race::checkpoints_store::distance_run_on(race.checkpoints(), n as usize).get()
    };
    let from: u32 = race.physics.split_tables().checkpoints[0].get();
    let d = travelled(at(car.checkpoint.get()), at(from));
    let (unit, miles) = (
        race.course.numbers.get().k65536.get(),
        race.course.numbers.get().metres_in_mile.get(),
    );
    let shown = distance_shown(d as i32, unit, miles).to_bits();
    distance.set(shown);
    if first {
        network_car_distances(race, unit, miles, from)?;
        network_disc_cars(race);
    }
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

/// A network race's cars' distances shown, for the stage bar's markers of the other cars (the
/// same figure as the first view's distance).
fn network_car_distances(race: &Race, unit: f32, miles: f32, from: u32) -> Result<(), BadInput> {
    let panel = &race.frame.panel_and_overlay.get().panel;
    let count = panel.network_cars.get().min(8) as usize;
    panel
        .network_own
        .set(race.shared.view.get().view_number(0)? as u32);
    car_distances(race, count, unit, miles, from)
}

/// A network race's other cars that are on the course, as the map disc shows them: where each lies
/// on the map from the first view's car, in map pixels. A car that is out of the race (a player
/// who left) or whose place on the map cannot be worked out is left out.
fn network_disc_cars(race: &Race) {
    let panel = &race.frame.panel_and_overlay.get().panel;
    let count = panel.network_cars.get().min(8) as usize;
    let own = panel.network_own.get() as usize;
    let mut shown = 0u32;
    if count > 1 && race.shared.vars.get().checkpoint_count.get() >= 2 {
        let at = |n: usize| {
            let car = race.shared.race_car(n).ok()?;
            crate::race::map_scroll::map_position(race, car).ok()
        };
        if let Some(from) = at(own) {
            for n in (0..count).filter(|&n| n != own) {
                let in_race = race
                    .shared
                    .race_car(n)
                    .is_ok_and(|car| car.driver_kind.get() != 0);
                let Some(place) = at(n).filter(|_| in_race) else {
                    continue;
                };
                for k in 0..2 {
                    let away = place[k].wrapping_sub(from[k]) as i32;
                    panel.disc_offsets[n][k].set(away as f32 / 65536.0);
                }
                shown |= 1 << n;
            }
        }
    }
    panel.disc_cars.set(shown);
}

/// The players a split screen of three or four shows on every part's stage bar.
fn panel_split_players(race: &Race, players: usize) {
    let shown = if players > 2 { players.min(4) } else { 0 };
    race.frame
        .panel_and_overlay
        .get()
        .panel
        .split_players
        .set(shown as u32);
}

/// A split screen's other players as the map discs show them: for each view, the players' cars
/// other than the view's own with where each lies on the map from the view's car, in map pixels.
/// Computer cars are not shown. With one player the discs keep what `network_disc_cars` made (the
/// first) or show nothing (the others).
fn split_disc_cars(race: &Race) -> Result<(), BadInput> {
    let panel = &race.frame.panel_and_overlay.get().panel;
    let view = race.shared.view.get();
    let players = (view.players.get() as usize).min(4);
    if players < 2 {
        for cars in &panel.disc_cars_more {
            cars.set(0);
        }
        return Ok(());
    }
    let at = |car: usize| {
        let car = race.shared.race_car(car).ok()?;
        crate::race::map_scroll::map_position(race, car).ok()
    };
    let enough = race.shared.vars.get().checkpoint_count.get() >= 2;
    let cars = (0..players)
        .map(|p| view.view_number(p))
        .collect::<Result<Vec<_>, _>>()?;
    for slot in 0..4 {
        let (shown_cell, offsets) = if slot == 0 {
            (&panel.disc_cars, &panel.disc_offsets)
        } else {
            (
                &panel.disc_cars_more[slot - 1],
                &panel.disc_offsets_more[slot - 1],
            )
        };
        let mut shown = 0u32;
        if let Some(own) = cars.get(slot)
            && let Some(from) = enough.then(|| at(*own)).flatten()
        {
            for &n in cars.iter().filter(|&n| n != own && *n < offsets.len()) {
                let Some(place) = at(n) else { continue };
                for k in 0..2 {
                    let away = place[k].wrapping_sub(from[k]) as i32;
                    offsets[n][k].set(away as f32 / 65536.0);
                }
                shown |= 1 << n;
            }
        }
        shown_cell.set(shown);
    }
    Ok(())
}

/// The first `count` cars' distances shown into `car_distances`.
fn car_distances(
    race: &Race,
    count: usize,
    unit: f32,
    miles: f32,
    from: u32,
) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{distance_shown, travelled};
    let panel = &race.frame.panel_and_overlay.get().panel;
    for (n, shown) in panel.car_distances.iter().enumerate().take(count) {
        let car = race.frame.cars.car(n)?;
        let at = |n: u32| {
            crate::race::checkpoints_store::distance_run_on(race.checkpoints(), n as usize).get()
        };
        let d = travelled(at(car.checkpoint.get()), at(from));
        shown.set(distance_shown(d as i32, unit, miles).to_bits());
    }
    Ok(())
}

/// The first view's values copied to `to`.
pub fn copy_view(from: &crate::race::panel_store::ViewA, to: &crate::race::panel_store::ViewA) {
    to.whole.set(from.whole.get());
    to.speed.set(from.speed.get());
    to.gear.set(from.gear.get());
    to.place.set(from.place.get());
    to.time.set(from.time.get());
    to.split2.set(from.split2.get());
    to.back.set(from.back.get());
    to.negated.set(from.negated.get());
    to.front.set(from.front.get());
    for (a, b) in from.motion.iter().zip(&to.motion) {
        b.set(a.get());
    }
}

/// Both view blocks' values into the panel (see `panel_half`).
pub fn panel_values_plain(race: &Race) -> Result<(), BadInput> {
    {
        // A split screen of more than two players: each player's values for the HUD of the
        // player's part, taken the way the first view's are, before the two views' own.
        let players = race.shared.view.get().players.get() as usize;
        if players > 2 {
            let panel = &race.frame.panel_and_overlay.get().panel;
            for (n, part) in panel.parts.iter().enumerate().take(players.min(4)) {
                panel_half(race, n, true)?;
                let place = race.frame.cars.car(n)?.two_car_place.get();
                copy_view(&panel.view_a, part);
                part.place.set(place);
            }
        }
        panel_half(race, race.shared.view.get().view_number(0)?, true)?;
        panel_half(race, race.shared.view.get().view_number(1)?, false)?;
        split_disc_cars(race)?;
        // A split screen of more than two players shows each player's progress on its own part's
        // bar (`hud::layout_many`), from the same distances the network markers use.
        let players = race.shared.view.get().players.get() as usize;
        panel_split_players(race, players);
        if players > 2 {
            let (unit, miles) = (
                race.course.numbers.get().k65536.get(),
                race.course.numbers.get().metres_in_mile.get(),
            );
            let from: u32 = race.physics.split_tables().checkpoints[0].get();
            car_distances(race, players.min(4), unit, miles, from)?;
        }
    }
    Ok(())
}

/// Wakes the part sounds: for each race car (car 0 at least: the loop tests at its end) and each of
/// its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 = 1; then, only when +0x700 was
/// exactly 10, and when `race().frame.frame_work.listed_car_sounds[current_car]` is not -1, the
/// car's sound object (`SoundObject` `car & 7`) +0x264 = 1 and +0x10 = 1. Refused (before
/// anything): more than eight cars, a null block or object it would write, and `current_car` past 7
/// when it is read.
pub fn wake_part_sounds(race: &Race, sound: &crate::sound::owner::Sound) -> Result<(), Refused> {
    let shared = &race.shared;
    let cars = shared.view.get().race_cars.get().max(1);
    if cars > 8 {
        return Err(BadInput("more than eight race cars").into());
    }
    let current: u32 = shared.work.get().current_car.get();
    // The plan: (block slot to set, whether to wake the object) by car.
    let mut plan: Vec<&dyn crate::memory::session::DwordLike> = Vec::new();
    for car in 0..cars {
        let object = crate::sound::store::SoundObject::of(sound, car & 7)?;
        let mut wake = false;
        let block = shared.race_car_or(car as usize, "a null car block")?;
        // Every flag set: each slot is `Left` whatever its count, nothing to plan (the flags only
        // go 0 -> 1, `part_woken` has no other writer).
        if block.part_woken.iter().all(|flag| flag.get() != 0) {
            continue;
        }
        for slot in 0..6usize {
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
            let view: u32 = race.frame.frame_work.get().listed_car_sounds[current as usize].get();
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

/// In mode 2 (`run_kind`), puts the first view's place (+0x654) and its neighbours into
/// the panel: `view_a.place` the place, `view_a.time` +0x8C x 10, `view_a.back` the car behind's
/// +0x24 (-1 at the last place), its label (`car_label` of
/// `arcade_car_numbers[place]`), `view_a.split2` +0x634, `view_a.front` the car ahead's
/// +0x24 (-1 at place 1), its label (`number_before_first[place]`), `view_a.negated`
/// -(+0x630); all fields of `race().frame.panel_and_overlay.panel`. Refused (before anything): a
/// neighbour's car number past 7 or a null block.
pub fn panel_neighbours(race: &Race) -> Result<(), Refused> {
    let frame = &race.frame;
    let shared = &race.shared;
    if shared.vars.get().run_kind.get() != 2 {
        return Ok(());
    }
    let a = race.frame.cars.car(shared.view.get().view_number(0)?)?;
    let place = a.mode2_place.get();
    // The place indexes unchecked: the cars by place start at `computer_ease_speed` (slot 0, the
    // float 30.0) and run on into `arcade_car_kinds`; the numbers' `number_before_first` is the
    // entry at place -1.
    let on = |first: Listed, n: u32| shared.tables().dword_from(first, n as usize);
    let block_of = |car: u32| -> Result<_, Refused> {
        if car > 7 {
            return Err(BadInput("a neighbour's car past the eight").into());
        }
        if !shared.work.get().has_car_block(car as usize) {
            return Err(BadInput("a neighbour's block is null").into());
        }
        Ok(race
            .frame
            .cars
            .car(shared.work.get().block_number(car as usize)?)?)
    };
    let behind = block_of(on(Listed::ComputerEaseSpeed, place.wrapping_add(1))?)?;
    let ahead = block_of(on(Listed::ComputerEaseSpeed, place)?)?;
    frame.panel_and_overlay.get().panel.view_a.place.set(place);
    frame
        .panel_and_overlay
        .get()
        .panel
        .view_a
        .time
        .set(a.time_left.get().wrapping_mul(10));
    use rc2k_core::race::frame::neighbour_number;
    let number = behind.car_kind.get();
    let back = neighbour_number(place == shared.tables().arcade_cars.get(), number);
    frame.panel_and_overlay.get().panel.view_a.back.set(back);
    car_label(
        &frame.panel_and_overlay.get().overlay.label_behind,
        on(Listed::ArcadeCarNumbers, place)?,
        back,
    );
    frame
        .panel_and_overlay
        .get()
        .panel
        .view_a
        .split2
        .set(a.pace_gaps_eased[1].get());
    let front = neighbour_number(place == 1, ahead.car_kind.get());
    frame.panel_and_overlay.get().panel.view_a.front.set(front);
    car_label(
        &frame.panel_and_overlay.get().overlay.label_ahead,
        on(Listed::NumberBeforeFirst, place)?,
        front,
    );
    frame
        .panel_and_overlay
        .get()
        .panel
        .view_a
        .negated
        .set(a.pace_gaps_eased[0].get().wrapping_neg());
    Ok(())
}

/// For each view block, stores +0xAE8 and +0xAEC x 0.5 / 65536 (floats) and the heading,
/// (-(+0x9C0)) >> 14 x pi x 0.5 / 65536, into the panel
/// (`race().frame.panel_and_overlay.panel.view_a.motion[0]`.. for the first, `view_b.motion[0]`..
/// for the second). The heading turn and `scene_clock` less the shown-since clock and 800 are not
/// stored: nothing reads them.
pub fn panel_motion_plain(race: &Race) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{heading, heading_turn, motion};
    {
        // NUMBERS.half (0.5), NUMBERS.one_over_65536, NUMBERS.k65536 (65536.0)
        let (half, small, big) = (
            race.course.numbers.get().half.get(),
            race.course.numbers.get().one_over_65536.get(),
            race.course.numbers.get().k65536.get(),
        );
        for (view, out) in [
            (0, &race.frame.panel_and_overlay.get().panel.view_a.motion),
            (1, &race.frame.panel_and_overlay.get().panel.view_b.motion),
        ] {
            let b = race.shared.view.get().view_number(view)?;
            let car = &race.car_blocks().cars[b];
            out[0].set(motion(car.map_picture_place[0].get() as i32, half, small).to_bits());
            out[1].set(motion(car.map_picture_place[1].get() as i32, half, small).to_bits());
            let turn = heading_turn(race.frame.cars.car(b)?.car_heading.get());
            out[2].set(heading(turn as i32, half, big).to_bits());
        }
        // A split screen of three or four: the further players' maps move with their cars too.
        let panel = &race.frame.panel_and_overlay.get().panel;
        let players = race.shared.view.get().players.get() as usize;
        for (n, part) in panel.parts.iter().enumerate().take(players.min(4)).skip(2) {
            let car = &race.car_blocks().cars[race.shared.view.get().view_number(n)?];
            let out = &part.motion;
            out[0].set(motion(car.map_picture_place[0].get() as i32, half, small).to_bits());
            out[1].set(motion(car.map_picture_place[1].get() as i32, half, small).to_bits());
            let turn = heading_turn(race.frame.cars.car(n)?.car_heading.get());
            out[2].set(heading(turn as i32, half, big).to_bits());
        }
    }
    Ok(())
}

/// Sets the mark dwords 0; for each view block (the third and fourth of a split screen too),
/// `race().frame.panel_and_overlay.panel.marks[0].base` / `marks[1].base` = 400 + (+0xFA0), and its
/// first +0xF98 (at most 4) values at +0xF78, each + 1.
pub fn panel_marks(race: &Race) -> Result<(), BadInput> {
    let frame = &race.frame;
    {
        let panel = &frame.panel_and_overlay.get().panel;
        for view in panel.marks.iter().chain(&panel.marks_more) {
            for mark in &view.marks {
                mark.set(0);
            }
        }
        // The split screen's third and fourth views have signs of their own, when they are shown.
        let views = race.shared.view.get().views.get().clamp(2, 4) as usize;
        for view in 0..views {
            let shown = panel.marks_of_view(view);
            let b = race.shared.view.get().view_number(view)?;
            let offset = race.frame.cars.car(b)?.car_event_position.get();
            let count = race.frame.cars.car(b)?.car_event_count.get();
            let codes = &race.frame.cars.car(b)?.car_event_codes;
            let past = std::cell::Cell::new(false);
            let m = rc2k_core::race::frame::view_marks(offset, count, |k| {
                codes.get(k as usize).map_or_else(
                    || {
                        past.set(true);
                        0
                    },
                    crate::memory::session::Dword::get,
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

/// One view's start lights: `view` the view's light fields, `block` the view's car block, `flag`
/// where a "step" flag goes.
pub fn countdown_lights(
    frame: &Frame,
    view: &crate::race::panel_store::Lights,
    block: usize,
    flag: &PlainDword,
) -> Result<(), BadInput> {
    {
        let (lit, width, alpha, drop) = (&view.lit, &view.width, &view.alpha, &view.drop);
        width.set(0);
        drop.set(0);
        flag.set(0);
        if frame.cars.car(block)?.false_start_count.get() != 0 {
            return Ok(());
        }
        use rc2k_core::race::start::lights;
        let Some(l) = lights(
            frame
                .cars
                .car(block)?
                .run_ticks
                .get()
                .wrapping_sub(frame.panel_and_overlay.get().overlay.countdown_start.get()),
        ) else {
            return Ok(());
        };
        frame
            .panel_and_overlay
            .get()
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

/// Runs `countdown_lights` for the first view and the second (fields 0x10,
/// `race().frame.panel_and_overlay.overlay.forget_flag_1`).
pub fn both_countdown_lights(race: &Race) -> Result<(), BadInput> {
    let frame = &race.frame;
    {
        for (view, flag) in [
            (0, &frame.panel_and_overlay.get().overlay.forget_flag_0),
            (1, &frame.panel_and_overlay.get().overlay.forget_flag_1),
        ] {
            countdown_lights(
                frame,
                &frame.panel_and_overlay.get().panel.lights[view],
                race.shared.view.get().view_number(view)?,
                flag,
            )?;
        }
    }
    Ok(())
}

/// The two views' colour buffers for `split_colours`:
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_a`,
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_b`.
pub fn colour_buffers(frame: &Frame) -> [&'_ [PlainDword; 64]; 2] {
    [
        &frame.panel_and_overlay.get().overlay.overlay_part_kinds_a,
        &frame.panel_and_overlay.get().overlay.overlay_part_kinds_b,
    ]
}

/// `split_colours`' refusal, for both views, before anything is written.
pub fn check_split_colours(race: &Race) -> Result<(), Refused> {
    let fill: u32 = race.physics.split_tables().split_colour_count.get();
    for view in 0..2 {
        let n = race
            .frame
            .cars
            .car(race.shared.view.get().view_number(view)?)?
            .splits_passed
            .get();
        rc2k_core::race::frame::check_split_colours(fill, n)?;
    }
    Ok(())
}

/// Runs `split_colours` for the first view's block into
/// `race().frame.panel_and_overlay.overlay.overlay_part_kinds_a`, then the second's into
/// `overlay_part_kinds_b`. Refused (before anything) as `split_colours` would for either.
pub fn both_split_colours(race: &Race) -> Result<(), Refused> {
    check_split_colours(race)?;
    for (view, buffer) in (0..2).zip(colour_buffers(&race.frame)) {
        split_colours(
            &race.frame.cars,
            race.shared.view.get().view_number(view)?,
            buffer,
            race.physics.split_tables().split_colour_count.get(),
        )?;
    }
    Ok(())
}

/// `panel_neighbours`' refusal, before anything is written.
pub(super) fn check_panel_neighbours(
    cars: &crate::race::frame_view::FrameCars,
    shared: &RaceShared,
) -> Result<(), Refused> {
    if shared.vars.get().run_kind.get() != 2 {
        return Ok(());
    }
    let place = cars
        .car(shared.view.get().view_number(0)?)?
        .mode2_place
        .get();
    // The cars by place (from `computer_ease_speed`, see `panel_neighbours`) at the place and the
    // one after.
    for n in [place, place.wrapping_add(1)] {
        let car = shared
            .tables()
            .dword_from(Listed::ComputerEaseSpeed, n as usize)?;
        if car > 7 || !shared.work.get().has_car_block(car as usize) {
            return Err(BadInput("a neighbour's car past the eight or null").into());
        }
    }
    Ok(())
}

/// Runs `panel_values_plain`, `panel_neighbours`, `panel_players`, `panel_motion_plain`,
/// `panel_marks`, `both_countdown_lights` and `both_split_colours`. Refused (before anything): what
/// `panel_neighbours` or `both_split_colours` would refuse.
pub fn set_up_panel(race: &Race) -> Result<(), Refused> {
    check_panel_neighbours(&race.frame.cars, &race.shared)?;
    check_split_colours(race)?;
    panel_values_plain(race)?;
    panel_neighbours(race)?;
    panel_players(race)?;
    panel_motion_plain(race)?;
    panel_marks(race)?;
    both_countdown_lights(race)?;
    both_split_colours(race)
}

/// Sets `lamp_count`, the view's two sort lists' counts and `race().frame.frame_work.flare_count`
/// to 0, then runs `object_bytes`.
pub fn reset_lamps(ctx: &crate::race::view_scratch::DrawCtx) -> Result<(), BadInput> {
    let crate::race::view_scratch::DrawCtx { view, reads, .. } = *ctx;
    view.lamps.lamp_count.set(0);
    view.lists.depth_entry_count.set(0);
    view.lists.sort_entry_count.set(0);
    view.lamps.flare_count.set(0);
    object_bytes(reads, &view.course)
}

/// Holds `value` to `patch_settings` (unsigned).
pub fn hold_to_limit(work: &crate::race::work_store::RaceWork, value: u32) -> u32 {
    rc2k_core::race::frame::hold_to_limit(value, work.patch_settings()[0].get())
}
