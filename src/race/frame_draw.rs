//! The race's drawing (`sub_4541BF`'s tree, IDA 2026-10-17): its leaves that are game logic,
//! not platform. The panel's fields at 0x6D14xx are a reading.

use crate::audio::store::ENVIRONMENT;
use crate::errors::{BadInput, Refused};
use crate::net::store::NET;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::split_tables_store::SPLIT_TABLES;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::checkpoints_store::CHECKPOINTS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::flares_store::FLARE_TABLES;
use crate::race::frame_store::FRAME_WORK;
use crate::race::kind_store::KIND_TABLES;
use crate::race::level_grid_store::LEVEL_GRID;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::matrices_store::{MATRICES, SceneMatrices};
use crate::race::numbers_store::NUMBERS;
use crate::race::overlay_store::RACE_OVERLAY;
use crate::race::panel_clock_store::PANEL_CLOCK;
use crate::race::panel_store::PANEL;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::sound_spots_store::RACE_SOUND_SPOTS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::tables_store::RACE_TABLES;
use crate::race::trap::Trap;
use crate::race::trap::car_at;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::view_vars_store::VIEW_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_store::DRAW;
use crate::scene::draw_switches_store::DRAW_SWITCHES;
use crate::scene::engine_tables_store::ENGINE_TABLES;
use crate::scene::faces_store::FACES;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_tables_store::LIGHT_TABLES;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_slots_store::OBJECT_SLOTS;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::polygon_store::POLYGON_WORK;
use crate::scene::texture_places_store::TEXTURE_PLACES;
use crate::session::Dword;
use crate::session::MACHINE;
use crate::session::SCREEN;
use crate::sound::store::RACE_SOUNDS;
use crate::sound::store::STAGE_SOUNDS;

/// A dword at game address `a` (frame words and matrices).
fn get(a: u32) -> Result<u32, BadInput> {
    GamePtr::new(a).read("a frame word")
}

/// Field `offset` of course face `face`.
fn face_field<T: crate::race::course_places::Whole>(face: u32, offset: u32) -> Result<T, BadInput> {
    crate::race::course_places::face_field(face, offset, "a course face")
}

/// Field `offset` of course record `record`.
fn record_field(record: u32, offset: u32) -> Result<u32, BadInput> {
    crate::race::course_places::record_field(record, offset, "a course record")
}

/// `race::car_label` (0x45B1B7): at edi seven bytes, " CAR nn" with nn eax's two digits
/// (eax / 10 and eax % 10, each + '0' as a byte), or seven spaces when ecx is -1. Every
/// register kept.
pub fn car_label(label_at: &[Dword; 2], eax: u32, ecx: u32) {
    // Converted: `rc2k_core::race::start::car_label`. Seven bytes: the eighth is kept.
    let label = rc2k_core::race::start::car_label(eax, ecx == u32::MAX);
    let kept = label_at[1].get().to_le_bytes()[3];
    label_at[0].set(u32::from_le_bytes([label[0], label[1], label[2], label[3]]));
    label_at[1].set(u32::from_le_bytes([label[4], label[5], label[6], kept]));
}

/// `race::split_colours` (0x45B16A): `dword_601830` dwords at edi set to 0xC; then for each
/// of the block esi's +0x90 values at +0x140 (none when +0x90 is 0 or negative), edi's
/// dword 8 when it is not above the one before (0x63 before the first; the sign of the
/// difference, as `cmp`/`jns`), else 4. (The registers it leaves, ecx the count, eax the
/// last colour, edx the last value, are read by no caller.) Refuses (before anything) a
/// count past the buffers' 0x40 dwords.
/// Refuses also more splits than the block's 16 times (the original reads on into the
/// block's next fields).
pub fn split_colours(esi: u32, edi: &[Dword; 64], fill: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::frame::{check_split_colours, split_colours}`.
    use rc2k_core::race::frame::{SPLIT_UNREACHED, check_split_colours, split_colours};
    let block = car_at(esi)?;
    let n = block.splits_passed.get();
    check_split_colours(fill, n)?;
    if n as i32 > 0 && n as usize > block.split_times.len() {
        return Err(BadInput("more splits than a block's times").into());
    }
    for cell in &edi[..fill as usize] {
        cell.set(SPLIT_UNREACHED);
    }
    if n as i32 <= 0 {
        return Ok(());
    }
    let splits: Vec<u32> = block.split_times[..n as usize]
        .iter()
        .map(super::super::session::Dword::get)
        .collect();
    let colours = split_colours(&splits);
    for (cell, &c) in edi.iter().zip(colours.iter()) {
        cell.set(c);
    }
    Ok(())
}

/// `race::panel_players` (0x45ACDB): unless `dword_62389C` is set, `dword_6D141A` and
/// `dword_6D144A` -1; with more than one player also `dword_6D1426` and `dword_6D142E` -1,
/// and `dword_6D141A` / `dword_6D144A` the two view blocks' +0xFA4.
pub fn panel_players() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::frame::panel_players`.
    use rc2k_core::race::frame::{PanelPlayers, panel_players};
    {
        let step = panel_players(RACE_VARS.run_kind.bits(), || RACE_VIEW.players.bits());
        if step == PanelPlayers::Kept {
            return Ok(());
        }
        PANEL.view_a.place.set(u32::MAX);
        PANEL.view_b.place.set(u32::MAX);
        if step == PanelPlayers::One {
            return Ok(());
        }
        PANEL.view_a.back.set(u32::MAX);
        PANEL.view_a.front.set(u32::MAX);
        let a = get(RACE_VIEW.view_block(0) + 0xFA4)?;
        PANEL.view_a.place.set(a);
        let b = get(RACE_VIEW.view_block(1) + 0xFA4)?;
        PANEL.view_b.place.set(b);
    }
    Ok(())
}

/// `race::lamps_wanted` (0x4536D7): `dword_63DB58` = `g_lampCount`, with bit 0 set when a
/// race car has +0x25C and +0x84 both set. (The registers it leaves, eax that, ecx the
/// cars, esi the last block, are read by no caller: `race::draw_view_scene`'s next
/// steps read none.) Refuses (before anything) more than eight cars (past `g_carBlocks`).
pub fn lamps_wanted() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::frame::{car_lit, lamps_wanted}`.
    use rc2k_core::race::frame::{car_lit, lamps_wanted};
    {
        let cars: u32 = RACE_VIEW.race_cars.bits();
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let mut any_lit = false;
        for n in 0..cars {
            let b: u32 = RACE_WORK.car_block(n as usize);
            let this_car = car_at(b)?;
            any_lit |= car_lit(this_car.driver_kind.bits(), || this_car.lights_on.bits());
        }
        RACE_VIEW
            .lamps_wanted
            .set_bits(lamps_wanted(RACE_VIEW.lamp_count.bits(), any_lit));
        Ok(())
    }
}

/// The view block `dword_63DB9C` names (`off_63DBB0[]`); refuses an index past the two.
fn drawn_view() -> Result<u32, Refused> {
    let n: u32 = RACE_VIEW.drawn_view.bits();
    if n > 1 {
        return Err(BadInput("a view past the two").into());
    }
    Ok(RACE_VIEW.view_block(n as usize))
}

/// `race::store_view_value` (0x455528): the drawn view's block +0xFB8 = `dword_6415C4`.
/// (It leaves eax that and esi the block; its one caller, `race::draw_view_scene`, reads
/// neither.)
pub fn store_view_value() -> Result<(), Refused> {
    {
        let b = drawn_view()?;
        car_at(b)?
            .eased_target
            .set_bits(FRAME_WORK.sun_on_screen.bits());
        Ok(())
    }
}

/// `race::fetch_view_value` (0x45550E): `dword_6415D8` = the drawn view's block +0xFBC.
/// Every register kept.
pub fn fetch_view_value() -> Result<(), Refused> {
    {
        let b = drawn_view()?;
        FRAME_WORK.view_out.set_bits(car_at(b)?.eased_value.bits());
        Ok(())
    }
}

/// `race::share_sound_spots` (0x45C7B9): `dword_5165EC` = `dword_6D1DC4`, and that many
/// 0x14-byte spots copied from 0x6D1AF4 to 0x5165F0 (read by the sound side,
/// `sub_40E642`). (It leaves ecx 0, esi and edi past the spots: its one caller,
/// `race::draw_race_scene`, reads none.) Refuses (before anything) more than 36 (past both
/// tables).
pub fn share_sound_spots() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::view::shared_spot_bytes`.
        let n: u32 = RACE_SOUND_SPOTS.sound_spot_count.bits();
        let bytes = rc2k_core::race::view::shared_spot_bytes(n)?;
        STAGE_SOUNDS.shared_count.set(n);
        let from = RACE_SOUND_SPOTS.sound_spots.chunks(5);
        for (spot, d) in STAGE_SOUNDS
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
}

/// `race::clear_point_records` (0x4500FF), on its variables: `dword_5C6FC4` 0, then the
/// 0x1000 point records of 0x20 bytes at `g_pointRecords`: each first dword -1, or with
/// `g_useSse` set each first 16 bytes `xmmword_62DAE0` (`movntps`). (It leaves ecx 0 and
/// edi past the records: its one caller, `race::draw_view_scene`, reads neither.)
pub fn clear_point_records() -> Result<(), BadInput> {
    COURSE_LISTS.clear_count.set(0u32);
    let pattern: [u32; 4] = crate::session::read_bits(&LIGHT_WORK.sse_fill);
    let sse = DRAW_SWITCHES.use_sse.get() != 0;
    for record in crate::scene::point_records::first_records(0x1000)? {
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

/// `race::object_bytes` (0x455263): for each of the 0x100 objects (records at
/// `dword_979F50`: +0x20 of each `texture_infos` entry, its first dword the kind, +0xE
/// bit 0), out0 = the record's +0xE bit 0, out1 = 1, out2 = the kinds' byte (`dword_5D3AE8`)
/// of the record's kind, out3 = 0xFF, or with faces (`dword_632F70`, first `dword_632B70`
/// in `g_faces`) the kinds' byte of the first face's +4 word. Outputs: the four face tables
/// `byte_63DC5C` ...
pub fn object_bytes() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::frame::object_bytes`.
    let kind = |n: u32| -> Result<u32, BadInput> {
        TEXTURE_PLACES
            .place_slot
            .get(n as usize)
            .map(super::super::session::Dword::get)
            .ok_or(BadInput("an object kind past the kinds' table"))
    };
    let outputs = face_tables();
    for n in 0..0x100usize {
        let record = &POLYGON_WORK.texture_infos[n].words[8..];
        let flags = record[3].get().to_le_bytes()[2];
        let record_kind = kind(record[0].get())?;
        let face_kind = if OBJECT_SLOTS.face_counts[n].get() != 0 {
            let first = OBJECT_SLOTS.first_face[n].get() as usize;
            let face = FACES
                .faces
                .get(first)
                .ok_or(BadInput("an object's first face past the faces"))?
                .get();
            // The face is a place of `scene::object_faces` (phase 13).
            let word = crate::scene::object_faces::bytes(face.wrapping_add(4), 2)?;
            let word = u16::from_le_bytes([word[0], word[1]]);
            Some(kind(word as u32)?)
        } else {
            None
        };
        let bytes = rc2k_core::race::frame::object_bytes(flags, record_kind, face_kind);
        for (out, b) in outputs.iter().zip(bytes) {
            out.set(n, b);
        }
    }
    Ok(())
}

/// One view's half of `race::panel_values`: block `b`'s values into the panel fields
/// (+0x718 rounded, +0xAB0, +0x1D4, the distance, +0x88 x 10, +0xA0C, -(+0xA10),
/// +0xA14, and for the first view also +0xA18, +0xA1C, +0xA20, +0xA24).
fn panel_half(b: u32, first: bool) -> Result<(), BadInput> {
    use rc2k_core::race::panel::{distance_shown, shown_whole, travelled};
    {
        let panel = &*PANEL;
        let (cells, distance) = if first {
            let v = &panel.view_a;
            (
                [
                    &v.whole, &v.speed, &v.gear, &v.place, &v.time, &v.split2, &v.negated,
                ],
                &panel.distance[0],
            )
        } else {
            let v = &panel.view_b;
            (
                [
                    &v.whole, &v.speed, &v.gear, &v.place, &v.time, &v.split2, &v.negated,
                ],
                &panel.distance[1],
            )
        };
        let whole = shown_whole(f32::from_bits(car_at(b)?.shown_engine_speed.bits()));
        cells[0].set(whole as u32);
        cells[1].set(car_at(b)?.engine_sound_speed_int.bits());
        cells[2].set(car_at(b)?.gear.bits());
        let at = |n: u32| {
            crate::race::checkpoints_store::cell_in(&CHECKPOINTS.distances, n as usize).get()
        };
        let from: u32 = SPLIT_TABLES.checkpoints[0].get();
        let d = travelled(at(car_at(b)?.checkpoint.bits()), at(from));
        DRAW.scratch.set_bits(d);
        let (unit, miles): (u32, u32) = (NUMBERS.k65536.bits(), NUMBERS.metres_in_mile.bits());
        let shown = distance_shown(d as i32, f32::from_bits(unit), f32::from_bits(miles)).to_bits();
        distance.set(shown);
        cells[4].set(car_at(b)?.stage_time.bits().wrapping_mul(10));
        cells[3].set(car_at(b)?.split_gaps[0].bits());
        // -(+0xA10): `dword_6D142A` for the first view, `dword_6D1456` for the second.
        cells[6].set(car_at(b)?.split_gaps[1].bits().wrapping_neg());
        cells[5].set(car_at(b)?.split_gaps[2].bits());
        if first {
            panel.view_a.front.set(car_at(b)?.split_gaps[3].bits());
            panel.view_a.back.set(car_at(b)?.split_gaps[4].bits());
            panel.split_5.set(car_at(b)?.split_gaps[5].bits());
            panel.split_6.set(car_at(b)?.split_gaps[6].bits());
        }
    }
    Ok(())
}

/// `race::panel_values` (0x45AB5A), converted (`set_up_panel`'s too; the
/// exact x87 version went once its one Rust caller converted): both view blocks' values
/// into the panel (see `panel_half`).
// Converted: `rc2k_core::race::panel::{shown_whole, travelled, distance_shown}`.
pub fn panel_values_plain() -> Result<(), BadInput> {
    {
        let a: u32 = RACE_VIEW.view_block(0);
        panel_half(a, true)?;
        let b: u32 = RACE_VIEW.view_block(1);
        panel_half(b, false)?;
    }
    Ok(())
}

/// `race::wake_part_sounds` (0x45D1EA): for each race car (car 0 at least: the loop tests
/// at its end) and each of its six slots, when +0x48 is 0 and +0x700 at least 10, +0x48 =
/// 1; then, only when +0x700 was exactly 10 (the `jnz` reads the `cmp`'s flags), and when
/// `dword_641680[g_currentCar]` is not -1, the car's sound object (`g_raceSoundObjects[car
/// & 7]`) +0x264 = 1 and +0x10 = 1. (The registers it leaves are read by no caller.)
/// Refuses (before anything) more than eight cars, a null block or object it would write,
/// and `g_currentCar` past 7 when it is read.
pub fn wake_part_sounds() -> Result<(), Refused> {
    {
        let cars = RACE_VIEW.race_cars.bits().max(1);
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let current: u32 = RACE_WORK.current_car.get();
        // The plan: (block slot to set, whether to wake the object) by car.
        let mut plan = Vec::new();
        for car in 0..cars {
            let b: u32 = RACE_WORK.car_block(car as usize);
            let o: u32 = RACE_SOUNDS.table[(car & 7) as usize].get();
            let mut wake = false;
            for slot in 0..6usize {
                if b == 0 {
                    return Err(BadInput("a null car block").into());
                }
                // Converted: `rc2k_core::race::lights::part_wake`.
                use rc2k_core::race::lights::{PartWake, part_wake};
                let block = car_at(b)?;
                let count = block.part_wake_counts[slot].get();
                let part = part_wake(block.unknown_48[slot].get(), count);
                if part == PartWake::Left {
                    continue;
                }
                plan.push(&block.unknown_48[slot]);
                if part != PartWake::WokenNow {
                    continue;
                }
                if current > 7 {
                    return Err(BadInput("g_currentCar past the eight").into());
                }
                let view: u32 = FRAME_WORK.listed_car_sounds[current as usize].bits();
                if view != u32::MAX {
                    if o == 0 {
                        return Err(BadInput("a null sound object").into());
                    }
                    wake = true;
                }
            }
            if wake {
                let object = crate::sound::store::SoundObject::at(o)?;
                plan.push(&object.slot_c.start);
                plan.push(&object.new_revs);
            }
        }
        for cell in plan {
            cell.set(1);
        }
        Ok(())
    }
}

/// `race::panel_neighbours` (0x45AD36): in mode 2 (`dword_62389C`), the first view's place
/// (+0x654) and its neighbours into the panel: `dword_6D141A` the place, `dword_6D141E`
/// +0x8C x 10, `dword_6D1426` the car behind's +0x24 (-1 at the last place,
/// `dword_6D1426`), its label at 0x6D1838 (`car_label` of `dword_605034[place]`),
/// `dword_6D1422` +0x634, `dword_6D142E` the car ahead's +0x24 (-1 at place 1), its label at
/// 0x6D1844 (`dword_605030[place]`), `dword_6D142A` -(+0x630). Refuses (before anything)
/// a neighbour's car number past 7 or a null block.
pub fn panel_neighbours() -> Result<(), Refused> {
    {
        if RACE_VARS.run_kind.bits() != 2 {
            return Ok(());
        }
        let a = car_at(RACE_VIEW.view_block(0))?;
        let place = a.mode2_place.get();
        // The place indexes unchecked, as the original does: `dword_604EC8[]`, the car at
        // each place, is `unknown_88c` run on into `arcade_car_kinds`; the numbers'
        // `number_before_first` is the entry at place -1.
        let on = |first: &Dword, n: u32| {
            crate::session::read_on_in(
                &RACE_TABLES,
                std::slice::from_ref(first),
                n as usize,
                "a place table",
            )
        };
        let block_of = |car: u32| -> Result<_, Refused> {
            if car > 7 {
                return Err(BadInput("a neighbour's car past the eight").into());
            }
            let b: u32 = RACE_WORK.car_block(car as usize);
            if b == 0 {
                return Err(BadInput("a neighbour's block is null").into());
            }
            Ok(car_at(b)?)
        };
        let behind = block_of(on(&RACE_TABLES.unknown_88c, place.wrapping_add(1))?)?;
        let ahead = block_of(on(&RACE_TABLES.unknown_88c, place)?)?;
        PANEL.view_a.place.set(place);
        PANEL.view_a.time.set(a.time_left.get().wrapping_mul(10));
        // Converted: `rc2k_core::race::frame::neighbour_number` (and the labels'
        // `rc2k_core::race::start::car_label`).
        use rc2k_core::race::frame::neighbour_number;
        let number = behind.car_kind.get();
        let back = neighbour_number(place == RACE_TABLES.arcade_cars.bits(), number);
        PANEL.view_a.back.set(back);
        car_label(
            &RACE_OVERLAY.label_behind,
            on(&RACE_TABLES.arcade_car_numbers[0], place)?,
            back,
        );
        PANEL.view_a.split2.set(a.pace_gaps_eased[1].get());
        let front = neighbour_number(place == 1, ahead.car_kind.get());
        PANEL.view_a.front.set(front);
        car_label(
            &RACE_OVERLAY.label_ahead,
            on(&RACE_TABLES.number_before_first, place)?,
            front,
        );
        PANEL
            .view_a
            .negated
            .set(a.pace_gaps_eased[0].get().wrapping_neg());
        Ok(())
    }
}

/// `race::panel_motion` (0x45AE1D), converted (`set_up_panel`'s too): for
/// each view block, +0xAE8 and +0xAEC x 0.5 / 65536 (floats) and the heading,
/// (-(+0x9C0)) >> 14 x pi x 0.5 / 65536, into the panel (`flt_6D1432`.. for the first,
/// `flt_6D145A`.. for the second); then `dword_62400C` = `g_sceneClock` - `dword_62290C` -
/// 800, 0 when negative.
// Converted: `rc2k_core::race::panel::{motion, heading_turn, heading, shown_for}`.
pub fn panel_motion_plain() -> Result<(), BadInput> {
    use rc2k_core::race::panel::{heading, heading_turn, motion, shown_for};
    {
        // flt_5C5E40 (0.5), flt_5C5DA0, flt_5C5D98 (65536.0)
        let (half, small, big) = (
            NUMBERS.half.get(),
            NUMBERS.one_over_65536.get(),
            NUMBERS.k65536.get(),
        );
        for (view, out) in [(0, &PANEL.view_a.motion), (1, &PANEL.view_b.motion)] {
            let b = RACE_VIEW.view_block(view);
            out[0]
                .set(motion(car_at(b)?.map_picture_place[0].bits() as i32, half, small).to_bits());
            out[1]
                .set(motion(car_at(b)?.map_picture_place[1].bits() as i32, half, small).to_bits());
            let turn = heading_turn(car_at(b)?.car_heading.bits());
            DRAW.scratch.set_bits(turn);
            out[2].set(heading(turn as i32, half, big).to_bits());
        }
        let shown = shown_for(PHYSICS.scene_clock.bits(), PANEL_CLOCK.shown_since.get());
        DRAW.scratch.set_bits(shown);
    }
    Ok(())
}

/// `race::panel_marks` (0x45AEFE): the eight dwords at 0x6D14A2 .. 0x6D14C2 (but 0x6D14B2)
/// 0; for each view block, `dword_6D14B2` / `dword_6D14C6` = 400 + (+0xFA0), and its first
/// +0xF98 (at most 4) values at +0xF78, each + 1, at 0x6D14A2 / 0x6D14B6.
pub fn panel_marks() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::frame::view_marks`.
    {
        for view in &PANEL.marks {
            for mark in &view.marks {
                mark.set(0);
            }
        }
        for (view, shown) in PANEL.marks.iter().enumerate() {
            let b: u32 = RACE_VIEW.view_block(view);
            let offset = car_at(b)?.car_event_position.bits();
            let count = car_at(b)?.car_event_count.bits();
            let codes = &car_at(b)?.car_event_codes;
            let past = std::cell::Cell::new(false);
            let m = rc2k_core::race::frame::view_marks(offset, count, |k| {
                codes.get(k as usize).map_or_else(
                    || {
                        past.set(true);
                        0
                    },
                    super::super::session::Dword::get,
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

/// `race::countdown_lights` (0x45B008): one view's start lights: `fields` 0 or 0x10 (edi:
/// the view's fields at 0x6D1482 + it), `block` (esi) the view's block, `flag` (ebp) where
/// a "step" flag goes. See the header for the steps. (The registers its divisions leave
/// are read by no caller.)
pub fn countdown_lights(
    view: &crate::race::panel_store::Lights,
    block: u32,
    flag: &Dword,
) -> Result<(), BadInput> {
    {
        let (lit, fade, size, shift) = (&view.lit, &view.fade, &view.size, &view.shift);
        fade.set(0);
        shift.set(0);
        flag.set(0);
        if car_at(block)?.false_start_count.bits() != 0 {
            return Ok(());
        }
        // Converted: `rc2k_core::race::start::lights`.
        use rc2k_core::race::start::lights;
        let Some(l) = lights(
            car_at(block)?
                .run_ticks
                .bits()
                .wrapping_sub(RACE_OVERLAY.countdown_start.bits()),
        ) else {
            return Ok(());
        };
        RACE_OVERLAY.start_lights_lit.set_bits(l.shown);
        lit.set(l.lit);
        if l.whole {
            flag.set(1);
        }
        size.set(l.size);
        fade.set(l.fade);
        shift.set(l.shift);
    }
    Ok(())
}

/// `race::both_countdown_lights` (0x45AFDB): `countdown_lights` for the first view (fields
/// 0, flag `dword_6D1830`) and the second (fields 0x10, `dword_6D1834`).
pub fn both_countdown_lights() -> Result<(), BadInput> {
    {
        for (view, flag) in [
            (0, &RACE_OVERLAY.forget_flag_0),
            (1, &RACE_OVERLAY.forget_flag_1),
        ] {
            countdown_lights(&PANEL.lights[view], RACE_VIEW.view_block(view), flag)?;
        }
    }
    Ok(())
}

/// The two views' colour buffers for `split_colours`: `dword_6D160C`, `dword_6D170C`.
pub fn colour_buffers() -> [&'static [Dword; 64]; 2] {
    [
        &RACE_OVERLAY.overlay_part_kinds_a,
        &RACE_OVERLAY.overlay_part_kinds_b,
    ]
}

/// `split_colours`' refusal, for both views, before anything is written.
pub fn check_split_colours() -> Result<(), Refused> {
    {
        let fill: u32 = SPLIT_TABLES.split_colour_count.get();
        for view in 0..2 {
            let n = car_at(RACE_VIEW.view_block(view))?.splits_passed.get();
            rc2k_core::race::frame::check_split_colours(fill, n)?;
        }
        Ok(())
    }
}

/// `race::both_split_colours` (0x45B147): `split_colours` for the first view's block into
/// `dword_6D160C`, then the second's into `dword_6D170C`. Refuses (before anything) as
/// `split_colours` would for either.
pub fn both_split_colours() -> Result<(), Refused> {
    {
        check_split_colours()?;
        for (view, buffer) in (0..2).zip(colour_buffers()) {
            split_colours(
                RACE_VIEW.view_block(view),
                buffer,
                SPLIT_TABLES.split_colour_count.get(),
            )?;
        }
        Ok(())
    }
}

/// `panel_neighbours`' refusal, before anything is written.
fn check_panel_neighbours() -> Result<(), Refused> {
    {
        if RACE_VARS.run_kind.bits() != 2 {
            return Ok(());
        }
        let place = car_at(RACE_VIEW.view_block(0))?.mode2_place.get();
        // `dword_604EC8[]` (see `panel_neighbours`) at the place and the one after.
        let first = std::slice::from_ref(&RACE_TABLES.unknown_88c);
        for n in [place, place.wrapping_add(1)] {
            let car = crate::session::read_on_in(&RACE_TABLES, first, n as usize, "a place table")?;
            if car > 7 || RACE_WORK.car_block(car as usize) == 0 {
                return Err(BadInput("a neighbour's car past the eight or null").into());
            }
        }
        Ok(())
    }
}

/// `race::set_up_panel` (0x45AB36): `panel_values`, `panel_neighbours`, `panel_players`,
/// `panel_motion`, `panel_marks`, `both_countdown_lights`, `both_split_colours`. Refuses
/// (before anything) what `panel_neighbours` or `both_split_colours` would. None of them
/// reads a register at entry (each sets what it uses), and its one caller,
/// `race::draw_race_scene`, reads none it leaves.
pub fn set_up_panel() -> Result<(), Refused> {
    {
        check_panel_neighbours()?;
        check_split_colours()?;
        // Converted: its float callees' `_plain` versions.
        panel_values_plain()?;
        panel_neighbours()?;
        panel_players()?;
        panel_motion_plain()?;
        panel_marks()?;
        both_countdown_lights()?;
        both_split_colours()
    }
}

/// `race::reset_lamps` (0x45487A): `g_lampCount`, `dword_63DC4C`, `dword_63DC50` (the sort
/// entries' count) and `dword_64158C` 0, then `race::object_bytes`.
pub fn reset_lamps() -> Result<(), BadInput> {
    {
        RACE_VIEW.lamp_count.set_bits(0);
        RACE_VIEW.depth_entry_count.set_bits(0);
        RACE_VIEW.sort_entry_count.set_bits(0);
        FRAME_WORK.flare_count.set_bits(0);
        object_bytes()
    }
}

/// `race::hold_to_limit` (0x4650A9): eax held to `dword_71C6D4` (unsigned).
pub fn hold_to_limit(eax: u32) -> u32 {
    // Converted: `rc2k_core::race::frame::hold_to_limit`.
    rc2k_core::race::frame::hold_to_limit(eax, RACE_WORK.patch_settings[0].bits())
}

// dword_9DB730: 0x1000 of 8 bytes
pub const SORT_MAX: u32 = 0x1000;

/// `race::add_sort_entry` (0x454D57): entry `dword_63DC50` of `dword_9DB730`: +4 ebx, +0
/// 0, or with (ebx & 0x7FFFFFFF) below 0x1000000 (eax << 16) | bx; the count + 1. (The
/// registers and carry it leaves are read by no caller.) Refuses (before anything) an
/// entry past the 0x1000.
pub fn add_sort_entry(eax: u32, ebx: u32) -> Result<(), Refused> {
    {
        let n: u32 = RACE_VIEW.sort_entry_count.bits();
        if n >= SORT_MAX {
            return Err(BadInput("a sort entry past the 0x1000").into());
        }
        // Converted: `rc2k_core::race::frame::sort_entry`.
        let entry = rc2k_core::race::frame::sort_entry(eax, ebx);
        let at = &FACES.sort_entries[2 * n as usize..2 * n as usize + 2];
        at[0].set(entry.first.unwrap_or(0));
        at[1].set(ebx);
        RACE_VIEW.sort_entry_count.set_bits(n.wrapping_add(1));
        Ok(())
    }
}

/// `race::checkpoint_colours` (0x4548E3): the current checkpoint's (`dword_71A070`) face
/// (`g_checkpointFaces`, records of 0x40 at `g_courseFaces`): its +0x10 byte's low and high
/// nibbles (0 read as 0x10) to `dword_614D08` / `dword_614D0C`; `dword_614D10` =
/// `dword_9ED808[checkpoint]` - `dword_71A998`; `dword_63F17C` the low (twice the distance
/// below 0x40000000 as signed), the high (from 0xC0000000), else the larger. The tables are
/// read unchecked, as the original.
pub fn checkpoint_colours() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::frame::checkpoint_colours`.
    {
        let n: u32 = CAR.checkpoint.get();
        let face =
            crate::race::checkpoints_store::cell_in(&CHECKPOINTS.checkpoint_faces, n as usize)
                .get();
        let byte: u8 = face_field(face, 0x10)?;
        let along =
            crate::race::checkpoints_store::cell_in(&CHECKPOINTS.angles[2..], n as usize).get();
        let c = rc2k_core::race::frame::checkpoint_colours(byte, along, CAR.car_heading.bits());
        PHYSICS.checkpoint_colours[0].set(c.near);
        PHYSICS.checkpoint_colours[1].set(c.far);
        PHYSICS.checkpoint_colours[2].set(c.distance);
        LEVEL_GRID.far_cell_limit.set(c.level);
    }
    Ok(())
}

/// A scene point's record (`sub_44CF33`, inline in the callers): the record at
/// `g_pointRecords` + (point & 0xFFF) x 0x20, made anew (`scene::make_scene_record`) when
/// its +0 is not the point.
pub fn point_record(point: u32) -> Result<u32, Refused> {
    let record = crate::scene::point_records::of_scene_point(point);
    if crate::scene::point_records::field(record, 0)? != point {
        crate::scene::project_point::make_scene_record(point, record)?;
    }
    Ok(record)
}

/// What `point_record` needs to make a record: the records and the points.
pub fn points_there() -> Result<(), Refused> {
    {
        if !crate::scene::point_records::held() || COURSE.scene_points.bits() == 0 {
            return Err(BadInput("no point records or no points").into());
        }
        Ok(())
    }
}

/// `race::face_hidden` (0x454D92), face record `esi` (read): carry (hidden) when the shown
/// view's `dword_606478` entry is 0, `dword_621480` is set and the face's +2 word has a
/// bit of `dword_64167C` (eax left that); else, inside `pusha`, its first point's record
/// (+4): `dword_635004` = |x| and `dword_635010` = depth + `flt_63F188` as whole numbers,
/// carry when the depth is below |x| (the sign of the difference). Gives eax and the carry.
/// Refuses (before anything) no point records or points.
pub fn face_hidden(face: &rc2k_core::race::faces::Face, eax: u32) -> Result<(u32, bool), Refused> {
    // Converted: `rc2k_core::race::faces::{hidden_by_switches, hidden_by_depth}`; the
    // depths' roundings the x87's.
    use rc2k_core::race::faces::{hidden_by_depth, hidden_by_switches};
    let mut eax = eax;
    let view: u32 = CAR.camera_view.bits();
    // dword_606478[]: by camera view. Its 34 entries cover views 0 .. 0x21; the views up
    // to 0x33 (the viewer's cameras) read on into the variables after it, as the original
    // (notes/bugs.md); 0x33 is still inside `VIEW_VARS`.
    let shows: u32 = crate::session::read_on_in(
        &*VIEW_VARS,
        &VIEW_VARS.view_hides,
        view as usize,
        "the view hides",
    )?;
    let hiding = if shows == 0 {
        FRAME_SWITCHES.other_way.bits()
    } else {
        0
    };
    let (bits, hidden) = hidden_by_switches(shows, hiding, || {
        face.switches() as u32 & FRAME_WORK.face_hide_mask.bits()
    });
    if let Some(bits) = bits {
        eax = bits;
    }
    if hidden {
        return Ok((eax, true));
    }
    points_there()?;
    let record = point_record(face.point(0))?;
    let x = crate::math::x87_load::abs_to_int(crate::scene::point_records::field(record, 0x10)?);
    OBJECT_WORK.object_position[0].set_bits(x as u32);
    let depth = crate::math::x87_load::add_to_int(
        crate::scene::point_records::field(record, 0x18)?,
        LEVEL_GRID.depth_shift.bits(),
    );
    OBJECT_WORK.face_depth.set_bits(depth as u32);
    Ok((eax, hidden_by_depth(depth as u32, x as u32)))
}

/// `race::face_depth` (0x4552D8): the depth key of face `ebx` (records of 0x40 at
/// `g_courseFaces`) into `dword_63DC40`: for a kind (+0) of 0x10 or more, its first point's
/// depth (`fistp`; and `dword_63DC44` its x); else, with its object's (+1) record +0xE bit
/// 1, the largest of its four points' depths (+4 .. +0x10, each `fistp`, through
/// `dword_62400C`), otherwise their float sum `fistp`'d and shifted right 2. Then + 0x2000000,
/// shifted right 10 (logical). Every register kept. Refuses (before anything) no point
/// records or points.
pub fn face_depth(ebx: u32) -> Result<(), Refused> {
    use crate::math::x87::{add, to_int};
    points_there()?;
    let face = ebx;
    let depth = |k: u32| -> Result<u32, Refused> {
        let record = point_record(face_field(face, 4 + 4 * k)?)?;
        Ok(crate::scene::point_records::field(record, 0x18)?)
    };
    // Converted: `rc2k_core::race::faces::{depth_by, deeper, average_key, depth_key}`;
    // the roundings and the sum the x87's.
    use rc2k_core::race::faces::{DepthBy, average_key, deeper, depth_by};
    let kind: u8 = face_field(face, 0)?;
    let trap = Trap::new();
    let by = depth_by(kind, || {
        let object = trap.keep_read(face_field::<u8>(face, 1)) as u32;
        // +0x2E of the object's 0x40-byte info (`object` a byte: inside the 0x100).
        POLYGON_WORK.texture_infos[object as usize].words[11]
            .get()
            .to_le_bytes()[2]
    });
    trap.finish()?;
    let key = match by {
        DepthBy::First => {
            let record = point_record(face_field(face, 4)?)?;
            let key = to_int(crate::scene::point_records::field(record, 0x18)?) as u32;
            RACE_VIEW.face_depth_key.set_bits(key);
            RACE_VIEW
                .face_side
                .set_bits(to_int(crate::scene::point_records::field(record, 0x10)?) as u32);
            key
        }
        DepthBy::Deepest => {
            let mut key = to_int(depth(0)?) as u32;
            for k in 1..4 {
                let d = to_int(depth(k)?) as u32;
                DRAW.scratch.set_bits(d);
                key = deeper(key, d);
            }
            key
        }
        DepthBy::Average => {
            let mut sum = depth(0)?;
            for k in 1..3 {
                sum = add(depth(k)?, sum);
            }
            average_key(crate::math::x87_load::add_to_int(depth(3)?, sum))
        }
    };
    RACE_VIEW
        .face_depth_key
        .set_bits(rc2k_core::race::faces::depth_key(key));
    Ok(())
}

// dword_9D3730: 0x1000 of 8 bytes

/// `race::add_depth_entry` (0x454D2A): `face_depth` of face ebx, then entry
/// `dword_63DC4C` of `dword_9D3730`: +0 the key, +4 ebx; the count + 1. (The registers and
/// carry it leaves are read by no caller.) Refuses (before anything) an entry past the
/// 0x1000, or what `face_depth` refuses.
pub fn add_depth_entry(ebx: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::faces::depth_entry_room`.
    let n: u32 = RACE_VIEW.depth_entry_count.bits();
    rc2k_core::race::faces::depth_entry_room(n)?;
    face_depth(ebx)?;
    let key: u32 = RACE_VIEW.face_depth_key.bits();
    let at = depth_entry(n)?;
    at[0].set(key);
    at[1].set(ebx);
    RACE_VIEW.depth_entry_count.set_bits(n.wrapping_add(1));
    Ok(())
}
// dword_63E17C[0x400]: 32 x 32 course records

/// `race::course_grid` (0x45497E): the course records around the view in a 32 x 32 grid.
/// The half size `dword_63E15C` = `hold_to_limit(dword_60A87C)`, the span `dword_63E160`
/// twice that; the corner is the view's place (`dword_6062B0` / `dword_6062B4` >> 21,
/// arithmetic) less the half. Unchanged corner: nothing more. Else the corner kept, the
/// grid cleared to 0x0FFFFFFF, and each record (0x20 bytes at `g_courseRecords`, +0 two
/// 11-bit cell numbers, -1 none; each into `dword_5CCBF0` / `dword_5CCBF4`) whose cell less
/// the corner (& 0x7FF) is within the span on both axes goes in at row z, column x.
/// Refuses (before anything is written) no records when the grid
/// is to be made (the original's loop would run on: it tests `jnz` after the first).
pub fn course_grid() -> Result<(), Refused> {
    {
        // Converted: `rc2k_core::race::frame::{hold_to_limit, grid_corner, record_cell,
        // grid_cell}`.
        use rc2k_core::race::frame::{GRID_EMPTY, GRID_SIDE, grid_cell, grid_corner, record_cell};
        let half = hold_to_limit(STAGE_PREFS.grid_reach.bits());
        let span = half.wrapping_add(half);
        let (gx, gz) = grid_corner(VIEW_VARS.view_x.bits(), VIEW_VARS.view_z.bits(), half);
        let unchanged = gx == LEVEL_GRID.grid_x.bits() && gz == LEVEL_GRID.grid_z.bits();
        let count: u32 = COURSE.course_record_count.bits();
        if !unchanged && count == 0 {
            return Err(BadInput("no course records (the loop would not end)?").into());
        }
        LEVEL_GRID.grid_half.set_bits(half);
        LEVEL_GRID.grid_span.set_bits(span);
        if unchanged {
            return Ok(());
        }
        LEVEL_GRID.grid_x.set_bits(gx);
        LEVEL_GRID.grid_z.set_bits(gz);
        let grid = &LEVEL_GRID.course_grid;
        for k in 0..GRID_SIDE * GRID_SIDE {
            grid[k as usize].set(GRID_EMPTY);
        }
        for n in 0..count {
            let Some((cx, cz)) = record_cell(record_field(n, 0)?) else {
                continue;
            };
            COURSE.record_place[0].set_bits(cx);
            COURSE.record_place[1].set_bits(cz);
            if let Some(cell) = grid_cell(cx, cz, (gx, gz), span) {
                course_grid_cell(cell)?.set(n);
            }
        }
        Ok(())
    }
}

/// `object_bytes`' outputs (byte tables, walked by pointer: absolute addresses).
/// Byte `n` of a face table (an object number: one past the 0x100 is refused, where the
/// original reads on into the next table).
fn byte_at(table: &crate::session::Bytes<256>, n: u32) -> Result<u32, BadInput> {
    if n >= 0x100 {
        return Err(BadInput("an object number past the face tables"));
    }
    Ok(table.get(n as usize) as u32)
}

/// The four face tables (`object_bytes`' outputs).
fn face_tables() -> [&'static crate::session::Bytes<256>; 4] {
    [
        &LEVEL_GRID.face_table_a,
        &LEVEL_GRID.face_table_b,
        &LEVEL_GRID.face_table_c,
        &LEVEL_GRID.face_table_d,
    ]
}

/// A course record's faces (+8 first, +0xC last; -1 none), or None.
fn record_faces(record: u32) -> Result<Option<(u32, u32)>, BadInput> {
    let (first, last) = (record_field(record, 8)?, record_field(record, 0xC)?);
    Ok((first != u32::MAX && first <= last).then_some((first, last)))
}

/// `race::gather_record_faces` (0x454C99): each face of course record esi (+8 .. +0xC),
/// `dword_5CCCD0` the face: a kind (+0) of 0xFF, or below 0x10, not `face_hidden`, goes to
/// `add_depth_entry` when its object's flag says so (`byte_63DD5C` / `byte_63DC5C`), else
/// to `add_sort_entry` with the object's kind byte (`byte_63DF5C` / `byte_63DE5C`). For
/// kinds below 0x10 the object is +1, or +0x21 when +1 is 0.
/// Refuses a last face of -1 (the loop would not end); the full tables and missing points
/// are refused where they are met.
pub fn gather_record_faces(record: u32) -> Result<(), Refused> {
    let Some((first, last)) = record_faces(record)? else {
        return Ok(());
    };
    if last == u32::MAX {
        return Err(BadInput("a course record's last face is -1").into());
    }
    let mut face = first;
    loop {
        COURSE.current_course_face.set_bits(face);
        // The face read once for its kind, objects, switches and first point.
        let read = crate::race::course_places::face_number(face, "a course face")?;
        let kind = read.kind();
        // Converted: `rc2k_core::race::faces::{gathered, face_object}`.
        use rc2k_core::race::faces::{Gathered, face_object, gathered};
        let how = gathered(kind);
        if how != Gathered::No {
            // The kind in eax's low byte (`mov al, [esi]`): only the eax `face_hidden`
            // gives back keeps the rest, and nothing reads that.
            let (_, hidden) = face_hidden(&read, kind as u32)?;
            if !hidden {
                let object = face_object(how, read.object(), || read.second_object());
                let tables = face_tables();
                let (depth_table, kind_table) = if how == Gathered::Lit {
                    (tables[1], tables[3])
                } else {
                    (tables[0], tables[2])
                };
                if byte_at(depth_table, object)? != 0 {
                    add_depth_entry(face)?;
                } else {
                    add_sort_entry(byte_at(kind_table, object)?, face)?;
                }
            }
        }
        face = face.wrapping_add(1);
        if face > record_field(record, 0xC)? {
            break;
        }
    }
    Ok(())
}

/// `race::gather_visible_faces` (0x454B71): `gather_record_faces` of each record in
/// `g_courseGrid` (numbers below 0x0FFFFFFF), then of the last record once more (its
/// "already done" test sets ebp inside `pusha`, so it is lost: `notes/bugs.md`). Refuses
/// as `gather_record_faces`.
pub fn gather_visible_faces() -> Result<(), Refused> {
    let todo = visible_records()?;
    for &record in &todo {
        gather_record_faces(record)?;
    }
    Ok(())
}

/// `race::face_hide_mask` (0x456056): `physics::probe_ground`, then, unless the face under
/// the car (`g_car` +0xE00) is -1, `dword_64167C` = its +2 word's bits 1 and 3 flipped
/// when any is set (0xA), or'd with bits 4 and 5 likewise (0x30).
pub fn face_hide_mask() -> Result<(), Refused> {
    crate::physics::probe_ground::probe_ground()?;
    let ground: u32 = CAR.probe_point_face[4].bits();
    if ground == u32::MAX {
        return Ok(());
    }
    // Converted: `rc2k_core::race::faces::hide_mask`.
    let word: u16 = face_field(ground, 2)?;
    FRAME_WORK
        .face_hide_mask
        .set_bits(rc2k_core::race::faces::hide_mask(word));
    Ok(())
}

/// `race::mark_far_cells` (0x454A5B): every `g_courseGrid` cell's top bits cleared (&
/// 0x0FFFFFFF); then each cell (column x, row z, each less the half `dword_63E15C`) turned
/// by `g_viewTurnMatrix` (into `flt_63F180` / `flt_63F184`, the middle into
/// `dword_62400C`, then its length there): not marked when the length is below 2.0; marked
/// (bit 31) when the turned z is below 0, or when |x / 2| - 2 is not below z, or when the
/// length is not below `dword_63F17C` (as a float, `dword_624020`).
pub fn mark_far_cells() -> Result<(), BadInput> {
    use crate::math::x87_load::{below_or_unordered, float_hypot, scaled_abs_less_below};
    let grid = &LEVEL_GRID.course_grid;
    for k in 0..0x400 {
        grid[k as usize].set(grid[k as usize].get() & 0x0FFF_FFFF);
    }
    let half: u32 = LEVEL_GRID.grid_half.bits();
    // Nothing in the loop writes the matrix: its floats read once.
    let matrix = crate::session::read_bits(&MATRICES.view_turn_matrix);
    let two: u32 = NUMBERS.two.bits();
    let zero: u32 = NUMBERS.zero.bits();
    let point_five: u32 = NUMBERS.half.bits();
    let mut out = [0u32; 8];
    for cell in 0..0x400u32 {
        let x = (cell & 0x1F).wrapping_sub(half);
        let z = (cell >> 5).wrapping_sub(half);
        let point = [x, 0, z];
        crate::scene::project_point::rotate(&point, &matrix, &mut out);
        let (x, z) = (out[4], out[6]);
        LEVEL_GRID.turned_x.set_bits(x);
        LEVEL_GRID.turned_z.set_bits(z);
        let length = float_hypot(x, z);
        DRAW.scratch.set_bits(length);
        let limit = crate::math::x87::from_int(LEVEL_GRID.far_cell_limit.get() as i32);
        DRAW.work_float_b.set_bits(limit);
        if below_or_unordered(length, two) {
            continue;
        }
        let mark = below_or_unordered(z, zero)
            || !scaled_abs_less_below(x, point_five, two, z)
            || !below_or_unordered(length, limit);
        if mark {
            grid[cell as usize].set(grid[cell as usize].get() | 0x8000_0000);
        }
    }
    Ok(())
}

/// `race::prepare_course_view` (0x4548A8): `dword_63F17C` 0x10, `face_hide_mask`,
/// `course_grid`; with `dword_621480` set, `dword_63F17C` 0x10 again, `checkpoint_colours`
/// with two checkpoints or more (`g_checkpointCount`), and `mark_far_cells`. Refuses
/// (before anything) what `course_grid` would.
pub fn prepare_course_view() -> Result<(), Refused> {
    check_course_grid()?;
    // `face_hide_mask` first: its `probe_ground` may refuse, and neither reads
    // `dword_63F17C`, so the 0x10 written before it in the original comes after here.
    face_hide_mask()?;
    LEVEL_GRID.far_cell_limit.set(0x10u32);
    course_grid()?;
    if FRAME_SWITCHES.other_way.bits() == 0 {
        return Ok(());
    }
    LEVEL_GRID.far_cell_limit.set(0x10u32);
    if RACE_VARS.checkpoint_count.get() >= 2 {
        checkpoint_colours()?;
    }
    mark_far_cells()?;
    Ok(())
}

/// `course_grid`'s refusal, before anything.
fn check_course_grid() -> Result<(), Refused> {
    {
        if COURSE.course_record_count.bits() == 0 {
            return Err(BadInput("no course records").into());
        }
        Ok(())
    }
}

/// The records `gather_visible_faces` gathers, as it would.
pub fn visible_records() -> Result<Vec<u32>, BadInput> {
    {
        // Converted: `rc2k_core::race::view::visible_records`.
        let grid = &LEVEL_GRID.course_grid;
        let list = rc2k_core::race::view::visible_records(
            grid.iter().map(super::super::session::Dword::get),
            COURSE.course_record_count.bits(),
        )
        .into_iter()
        .collect();
        Ok(list)
    }
}

/// `race::car_light_modes` (0x455B70): `dword_63DBA4` = the drawn view's car
/// (`dword_63DBA8[g_drawnView]`); every car block's +0x6FC 0; then for each race car with
/// lights (+0x25C) that is drawn (any car in modes 0 and 2, else the shown car only; the
/// shown car not with +0x69C and +0x684 both 0, nor with `dword_63993C` set): +0x6FC's
/// nibbles set step by step: 0x21; for the shown car with `dword_606274`, 0x12, then 0x14
/// unless `dword_606270` is 1, then 0x11 unless it is 2 or 3; then 0x100 in the third
/// nibble, 0x200 with `dword_6238B4` (1: the shown car only). Refuses (before anything) a null block among the eight or more than eight cars.
pub fn car_light_modes() -> Result<(), Refused> {
    {
        let block = |n: u32| RACE_WORK.car_block(n as usize);
        let cars: u32 = RACE_VIEW.race_cars.bits();
        if (0..8).any(|n| block(n) == 0) {
            return Err(BadInput("a null car block").into());
        }
        if cars > 8 {
            return Err(BadInput("more than eight race cars").into());
        }
        let view: u32 = RACE_VIEW.drawn_view.bits();
        let shown = RACE_VIEW
            .view_cars
            .get(view as usize)
            .ok_or(BadInput("a drawn view past the two"))?
            .get();
        RACE_VIEW.shown_car.set_bits(shown);
        for n in 0..8 {
            car_at(block(n))?.state_bits.set(0);
        }
        // Converted: `rc2k_core::race::lights::{lights_drawn, light_word}`.
        use rc2k_core::race::lights::{light_word, lights_drawn};
        let mode: u32 = RACE_VARS.run_kind.bits();
        for n in 0..cars {
            let this_car = car_at(block(n))?;
            let at = &this_car.state_bits;
            let me = n == shown;
            let drawn = lights_drawn(
                this_car.driver_kind.bits() != 0,
                mode,
                me,
                || this_car.camera_switch_share.bits() == 0 && this_car.camera_view.bits() == 0,
                || GROUND_WORK.view_mirrored.get() != 0,
            );
            if !drawn {
                at.set(0);
                continue;
            }
            let word = light_word(
                at.get(),
                me,
                || VIEW_VARS.view_car_flag_b.get() != 0,
                || VIEW_VARS.view_car_stage.bits(),
                RACE_VARS.settings_words[3].bits(),
            );
            at.set(word);
        }
        Ok(())
    }
}
/// Cell `n` of `g_courseGrid` (32 x 32). One past the grid refused (the original reads on).
fn course_grid_cell(n: u32) -> Result<&'static Dword, BadInput> {
    LEVEL_GRID
        .course_grid
        .get(n as usize)
        .ok_or(BadInput("a course grid cell past the 32 x 32"))
}

/// Lamp `n` of `g_lamps` (16 of 0x10 bytes: +0 the point, +4 the strength, +8 the
/// colour). One past the 16 refused (the original reads on).
fn lamp(n: u32) -> Result<&'static [Dword], BadInput> {
    let n = n as usize;
    LEVEL_GRID
        .lamps
        .get(4 * n..4 * n + 4)
        .ok_or(BadInput("a lamp past the sixteen"))
}

/// Depth entry `n` (8 bytes: the key, the second word). One past the 0x1000 refused.
fn depth_entry(n: u32) -> Result<&'static [Dword], BadInput> {
    let n = n as usize;
    FACES
        .depth_entries
        .get(2 * n..2 * n + 2)
        .ok_or(BadInput("a depth entry past the table"))
}
/// `race::add_lamp_entry` (0x4573FA): lamp ebx's point (`g_lamps` +0) through
/// `g_viewMatrix` (the point into `dword_63501C` .. `dword_635024`, then x and y turned
/// there, z rounded into `g_faceDepthKey`, + 0x2000000 and shifted right 10), and a depth
/// entry of that key with `g_lampCount` + 0x3000000; `dword_641590` + 1. The caller checks
/// the room.
fn add_lamp_entry(lamp: u32) -> Result<(), BadInput> {
    let point = self::lamp(lamp)?[0].get();
    let place = crate::race::scene_points::point_number(point)?;
    let turned = &OBJECT_WORK.place;
    for (cell, &v) in turned.iter().zip(&place) {
        cell.set(v);
    }
    let mut out = [0u32; 3];
    let matrix: [u32; 16] = crate::session::read_bits(&MATRICES.view_matrix);
    crate::math::x87_load::rotate_rounding_z(
        place.as_ptr().cast::<i32>(),
        matrix.as_ptr(),
        out.as_mut_ptr(),
    );
    turned[0].set(out[0]);
    turned[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let second = RACE_VIEW.lamp_count.bits().wrapping_add(0x300_0000);
    let n: u32 = RACE_VIEW.depth_entry_count.bits();
    let at = depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    RACE_VIEW.depth_entry_count.set_bits(n.wrapping_add(1));
    FRAME_WORK
        .lamp_entry_count
        .set_bits(FRAME_WORK.lamp_entry_count.bits().wrapping_add(1));
    Ok(())
}

/// `race::gather_lamps` (0x454E08): `g_lampCount` 0; each lit face (`g_litFaces`, the
/// count `g_litFaceCount`) whose object (+1) is 0x3B becomes a lamp while there are fewer
/// than 16: `g_lamps` entry (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF) and
/// its depth entry (`add_lamp_entry`, inside `pusha`). Refuses (before anything) more than 0x100 lit faces, or a depth list without room for 16.
pub fn gather_lamps() -> Result<(), Refused> {
    let count: u32 = CONTACT_WORK.lit_face_count.bits();
    if count > 0x100 {
        return Err(BadInput("more than 0x100 lit faces").into());
    }
    if count != 0 && RACE_VIEW.depth_entry_count.bits() > SORT_MAX - 16 {
        return Err(BadInput("the depth list has no room for the lamps").into());
    }
    RACE_VIEW.lamp_count.set_bits(0u32);
    for k in 0..count {
        let face = CONTACT_WORK.lit_faces[k as usize].bits();
        // Converted: `rc2k_core::race::faces::takes_lamp`.
        let object = face_field::<u8>(face, 1)? as u32;
        if object != rc2k_core::race::faces::LAMP_OBJECT {
            continue;
        }
        let lamp: u32 = RACE_VIEW.lamp_count.bits();
        if !rc2k_core::race::faces::takes_lamp(object, lamp) {
            continue;
        }
        let at = self::lamp(lamp)?;
        at[0].set(face_field(face, 4)?);
        at[1].set(crate::race::course_places::face_number(face, "a lamp's face")?.mark() as u32);
        at[2].set(rc2k_core::race::faces::LAMP_COLOUR);
        add_lamp_entry(lamp)?;
        RACE_VIEW.lamp_count.set_bits(lamp + 1);
    }
    Ok(())
}

/// `race::sort_sort_entries` (0x454F62) / `race::sort_depth_entries` (0x454ED4): with two
/// entries or more, the list's quicksort (`sub_454F85` / its twin `sub_454EF7`: 8-byte
/// entries by their first dword, the largest first) from its first entry to its last.
/// Refuses (before anything) a sort that would go past the
/// list.
/// `entries` the list.
pub fn sort_entries(count: &crate::session::Dword, entries: GamePtr) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::quick_sort::quick_sort` (through `sort_records`).
    use rc2k_core::race::quick_sort::Order;
    {
        let n: u32 = count.get();
        crate::race::sorting::sort_records::<2>(entries, n, 0, Order::Down)
    }
}

/// The game's table root of a float (`sqrt` by `g_rootTable` x `g_exponentRootTable`),
/// times 1/65536 (`flt_5C5DA0`), as stored.
fn table_root(v: u32) -> u32 {
    // g_rootTable: 256 floats by mantissa bits; g_exponentRootTable: by exponent
    let root = COURSE_LISTS.root_table[((v >> 15) & 0xFF) as usize].bits();
    let scale = COURSE_LISTS.exponent_root_table[((v & 0x7F80_0000) >> 23) as usize].bits();
    crate::math::x87_load::product3(root, scale, NUMBERS.one_over_65536.bits())
}

/// A point's x and z less car 0's (`g_carBlockStore` +0x134, +0x13C), and its y.
fn point_from_car(point: u32) -> Result<(u32, u32, u32), BadInput> {
    {
        let [px, py, pz] = crate::race::scene_points::point_number(point)?;
        let x = px.wrapping_sub(CAR_BLOCKS.cars[0].position[0].bits());
        let z = pz.wrapping_sub(CAR_BLOCKS.cars[0].position[2].bits());
        CONTACT_WORK.eax_point_x.set(x);
        CONTACT_WORK.eax_point_z.set(z);
        Ok((x, py, z))
    }
}

/// The weight's end: `dword_62400C` = `v`, held to 16.0 (`flt_5C5D08`; kept when below or
/// not a number), added to `g_eaxEnvironment`; `dword_6D1AF0` + 1.
fn add_eax_weight(v: u32) {
    {
        let sixteen: u32 = NUMBERS.sixteen.bits();
        let w = if crate::math::x87_load::below_or_unordered(v, sixteen) {
            v
        } else {
            sixteen
        };
        DRAW.scratch.set_bits(w);
        ENVIRONMENT.set(crate::math::x87::add(w, ENVIRONMENT.get()));
        RACE_SOUND_SPOTS
            .eax_face_count
            .set_bits(RACE_SOUND_SPOTS.eax_face_count.bits().wrapping_add(1));
    }
}

/// `race::eax_face_weight` (0x45C881): face ebx's weight in the EAX sound environment
/// (`g_eaxEnvironment`), by its distance from car 0 (x and z): kind 0xFF with an object
/// (+1) below 0x38: its +0x18 byte over the distance to its first point; kinds below 0x10
/// with |+0x30| below 0.25: 3.0 over the distance to the middle of its second and third
/// points (`flt_6D1DD8`, `flt_6D1DE0`); each held to 16 (see `add_eax_weight`). Distances
/// by the game's root tables into `flt_6D1DF0`.
pub fn eax_face_weight(face: u32) -> Result<(), BadInput> {
    use crate::math::x87::{div, from_int};
    use crate::math::x87_load::{
        abs_below_or_unordered, float_square_sum, int_over_float, int_plus_times, int_square_sum,
    };
    {
        // The face read once for its kind, objects, switches and first point.
        let read = crate::race::course_places::face_number(face, "a course face")?;
        let kind = read.kind();
        if kind == 0xFF {
            if face_field::<u8>(face, 1)? >= 0x38 {
                return Ok(());
            }
            let (x, _, z) = point_from_car(face_field(face, 4)?)?;
            let distance = table_root(int_square_sum(x as i32, z as i32));
            CONTACT_WORK.eax_distance.set_bits(distance);
            let strength = face_field::<u8>(face, 0x18)? as u32;
            DRAW.scratch.set_bits(strength);
            add_eax_weight(int_over_float(strength as i32, distance));
            return Ok(());
        }
        if kind >= 0x10 {
            return Ok(());
        }
        if !abs_below_or_unordered(face_field(face, 0x30)?, NUMBERS.quarter.bits()) {
            return Ok(());
        }
        let (x1, _, z1) = point_from_car(face_field(face, 8)?)?;
        let (mx, mz) = (from_int(x1 as i32), from_int(z1 as i32));
        CONTACT_WORK.eax_mid_x.set_bits(mx);
        CONTACT_WORK.eax_mid_z.set_bits(mz);
        let (x2, _, z2) = point_from_car(face_field(face, 0xC)?)?;
        let half: u32 = NUMBERS.half.bits();
        let mx = int_plus_times(x2 as i32, mx, half);
        CONTACT_WORK.eax_mid_x.set_bits(mx);
        let mz = int_plus_times(z2 as i32, mz, half);
        CONTACT_WORK.eax_mid_z.set_bits(mz);
        let distance = table_root(float_square_sum(mx, mz));
        CONTACT_WORK.eax_distance.set_bits(distance);
        add_eax_weight(div(NUMBERS.three.bits(), distance));
    }
    Ok(())
}

/// `race::eax_environment` (0x45C7EE): `g_eaxEnvironment` 0.0 and `dword_6D1AF0` 0, then
/// the 3 x 3 grid cells around the middle (from `hold_to_limit(dword_60A87C)` - 1 on each
/// axis; `sub_45C84C` each): every face of each cell's course record (numbers below
/// 0x0FFFFFFF) weighed by `eax_face_weight` (inside `pusha`).
pub fn eax_environment() -> Result<(), BadInput> {
    {
        ENVIRONMENT.set(0);
        RACE_SOUND_SPOTS.eax_face_count.set_bits(0u32);
        // Converted: `rc2k_core::race::view::{cells_from, record_faces}`.
        use rc2k_core::race::view::{NO_RECORD, cells_from, record_faces};
        for cell in cells_from(
            hold_to_limit(STAGE_PREFS.grid_reach.bits()).wrapping_sub(1),
            3,
        ) {
            // `sub_45C84C`.
            let record = course_grid_cell(cell)?.get();
            if record >= NO_RECORD {
                continue;
            }
            let mut face = record_field(record, 8)?;
            if record_faces(face, record_field(record, 0xC)?).is_none() {
                continue;
            }
            loop {
                eax_face_weight(face)?;
                face = face.wrapping_add(1);
                if face > record_field(record, 0xC)? {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Whether the game uses MMX (`g_hasMmx`).
pub fn has_mmx() -> bool {
    MACHINE.has_mmx.get() != 0
}

/// Two colours added byte by byte (`rc2k_core::race::lights::add_colours`), MMX or not
/// as the game.
pub fn add_colours(a: u32, b: u32) -> u32 {
    rc2k_core::race::lights::add_colours(a, b, has_mmx())
}

/// `sub_44F691`: a car's (block `b`) headlights on the lit place, added to
/// `g_lightColour` (see `light_colour_at`). Called inside `pusha`.
fn car_light_on_point(b: u32) -> Result<(), BadInput> {
    use crate::math::x87_load::{int_square_sum_scaled_to_int, rotate_rounding};
    use rc2k_core::race::lights::headlight_index;
    if car_at(b)?.lights_on.bits() == 0 {
        return Ok(());
    }
    let place = &LIGHT_STATE.lit_place;
    let d = [
        place[0].get().wrapping_sub(car_at(b)?.position[0].bits()),
        place[1].get().wrapping_sub(car_at(b)?.position[1].bits()),
        place[2].get().wrapping_sub(car_at(b)?.position[2].bits()),
    ];
    let mut t = [0u32; 3];
    rotate_rounding(
        d.as_ptr().cast::<i32>(),
        crate::session::read_bits(&car_at(b)?.car_inverse_matrix).as_ptr(),
        t.as_mut_ptr(),
    );
    crate::session::write_bits(&LIGHT_STATE.turned_light, t);
    let k: u32 = NUMBERS.one_over_65536.bits();
    let s1 = int_square_sum_scaled_to_int(
        t[2].wrapping_sub(0x20_0000) as i32,
        t[0].wrapping_add(t[0]) as i32,
        k,
        2,
    );
    // The second spread is worked out and then not used (eax is loaded over it).
    let s2 = int_square_sum_scaled_to_int(t[2].wrapping_sub(0x10_0000) as i32, t[0] as i32, k, 1);
    DRAW.scratch.set_bits(s2);
    let index = headlight_index(s1, t[1], car_at(b)?.marks_count.bits());
    let colour = LIGHT_TABLES.car_light_table[index as usize].get();
    LIGHT_WORK
        .light_colour
        .set_bits(add_colours(colour, LIGHT_WORK.light_colour.bits()));
    Ok(())
}

/// `sub_44F601`: the cars' headlights on the lit place: with `dword_52A910` the chosen car
/// (`dword_52A920`) or all with `dword_6238B8` 1; else the players' cars (all with
/// `dword_6238B8` 1), only the drawn view's car outside one player and modes 0 and 2.
fn car_lights_on_point() -> Result<(), BadInput> {
    let cars: u32 = RACE_VIEW.race_cars.bits();
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    let all = RACE_VARS.settings_words[4].get() == 1;
    let one = NET.session.get() != 0;
    for n in 0..cars {
        let trap = Trap::new();
        let lit = rc2k_core::race::lights::headlights_light(
            n,
            one,
            || NET.player.get(),
            all,
            || RACE_VIEW.players.bits(),
            || RACE_VARS.run_kind.bits(),
            || {
                let n = RACE_VIEW.drawn_view.bits() as usize;
                crate::session::read_on_in(&RACE_VIEW, &RACE_VIEW.view_cars, n, "the view cars")
                    .unwrap_or_else(|e| {
                        trap.keep(e);
                        0
                    })
            },
        );
        trap.finish()?;
        if lit {
            car_light_on_point(block(n))?;
        }
    }
    Ok(())
}

/// `sub_44F863`: the lamps' light on the lit place: each lamp with a strength (+4,
/// `dword_639958`) gives ((0x200000 - its approximate distance) x strength) >> 17 (held to
/// 0 .. 0xFF) as a grey, times its colour (+8, `dword_62DA7C`) byte by byte, added to
/// `g_lightColour`.
fn lamp_lights_on_point() -> Result<(), BadInput> {
    {
        let count: u32 = RACE_VIEW.lamp_count.bits();
        if count == 0 {
            return Ok(());
        }
        let place = &LIGHT_STATE.lit_place;
        for n in 0..count {
            let lamp = self::lamp(n)?;
            let strength = lamp[1].get();
            GROUND_WORK.point_light_strength.set(strength);
            if strength == 0 {
                continue;
            }
            let tint = lamp[2].get();
            LIGHT_WORK.lamp_tint.set(tint);
            let point = lamp[0].get();
            let coordinate = |k| crate::race::scene_points::point_word(point, k);
            let d = [
                coordinate(0)?.wrapping_sub(place[0].get()),
                coordinate(1)?.wrapping_sub(place[1].get()),
                coordinate(2)?.wrapping_sub(place[2].get()),
            ];
            let [distance, _, _] = crate::scene::approximate_distance::approximate_distance(d);
            DRAW.scratch.set_bits(distance);
            let (grey, _) = rc2k_core::race::lights::lamp_grey(
                distance,
                GROUND_WORK.point_light_strength.bits(),
            );
            let light = add_colours(
                rc2k_core::race::lights::multiply_colours(grey, tint),
                LIGHT_WORK.light_colour.bits(),
            );
            LIGHT_WORK.light_colour.set_bits(light);
        }
    }
    Ok(())
}

/// What `light_colour_at` refuses, before anything: no cars (its loop tests at the end),
/// more than eight, a null block it would read, more than 16 lamps.
pub fn check_light_colour() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::check_light_colour`.
    {
        Ok(rc2k_core::race::lights::check_light_colour(
            RACE_VIEW.race_cars.bits(),
            |n| RACE_WORK.car_block(n as usize),
            || RACE_VIEW.lamp_count.bits(),
        )?)
    }
}

/// `race::light_colour_at` (0x4537CE): `g_lightColour` 0, then the cars' headlights
/// (`sub_44F601`) and the lamps (`sub_44F863`) on the lit place (`g_litPlace`,
/// `dword_63658C`, `dword_636590`), added up. It reads no register; the ones it leaves are
/// read by no caller (each reads `g_lightColour`). Refuses as `check_light_colour`.
pub fn light_colour_at() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::{check_light_colour, headlights_light,
    // headlight_index, lamp_grey, add_colours, multiply_colours}` (the turned place, the
    // spreads and the lamps' distances the x87's and the scene's).
    check_light_colour()?;
    LIGHT_WORK.light_colour.set_bits(0u32);
    car_lights_on_point()?;
    lamp_lights_on_point()?;
    Ok(())
}

/// `race::view_light` (0x455896): the lit place the work car's (`g_car` +0x134 ..);
/// `light_colour_at`; `dword_6415E4` = `dword_62DAD4` plus the light (`add_colours`, through
/// `dword_62400C`). Its one caller, `race::draw_view_scene`, reads no register it leaves.
pub fn view_light() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::lights::add_colours` (through `light_colour_at`'s).
    check_light_colour()?;
    crate::session::write_bits(
        &LIGHT_STATE.lit_place,
        crate::session::read_bits(&CAR.position),
    );
    light_colour_at()?;
    let light: u32 = LIGHT_WORK.light_colour.bits();
    DRAW.scratch.set_bits(light);
    FRAME_WORK
        .view_light
        .set(add_colours(LIGHT_WORK.view_light_base.bits(), light));
    Ok(())
}

/// `race::draw_distance_settings` (0x43932A): the draw distance's level (`dword_60AC84`,
/// held to 0 ..= 4 by sign) picks from six tables: `dword_60A87C` (at least 1, by sign),
/// `dword_60930C`, `dword_609358`, `dword_609308`, `dword_609320`, `dword_60AC80`.
pub fn draw_distance_settings() {
    // Converted: `rc2k_core::race::view::{draw_distance_level, grid_reach}`.
    use rc2k_core::race::view::{draw_distance_level, grid_reach};
    {
        let level = draw_distance_level(STAGE_PREFS.draw_distance_level.bits());
        let level = level as usize;
        STAGE_PREFS
            .grid_reach
            .set(grid_reach(STAGE_PREFS.reach_by_level[level].bits()));
        STAGE_PREFS
            .fog_end
            .set_bits(STAGE_PREFS.fog_end_by_level[level].bits());
        STAGE_PREFS
            .fog_start
            .set_bits(STAGE_PREFS.fog_start_by_level[level].bits());
        STAGE_PREFS
            .fog_low
            .set_bits(STAGE_PREFS.fog_low_by_level[level].bits());
        STAGE_PREFS
            .fog_high
            .set_bits(STAGE_PREFS.fog_high_by_level[level].bits());
        STAGE_PREFS
            .nearer
            .set_bits(STAGE_PREFS.nearer_by_level[level].bits());
    }
}

/// `race::sample_noise` (0x44FD38): the 64 x 64 byte map at esi sampled at the work car's
/// place (`g_car` +0x6A4 and +0x6AC: the top byte the cell, the next two the fraction):
/// the four corners into `dword_637620` .. `dword_63762C`, blended along x into
/// `dword_637630` / `dword_637634` and then along z, each step `imul` and `shrd` by 16;
/// the result shifted left 8, given (the original's eax; what it leaves in ebx, ecx and edx,
/// the fractions and the last product's high half, its callers do not read).
pub fn sample_noise(map: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::view::sample_noise`.
    {
        let (px, pz) = (CAR.camera_position[0].get(), CAR.camera_position[2].get());
        let trap = Trap::new();
        let s = rc2k_core::race::view::sample_noise(
            |n| trap.read::<u8>(map + n, "a noise map"),
            px,
            pz,
        );
        trap.finish()?;
        let out = &LIGHT_STATE.noise_samples;
        for (cell, &c) in out.iter().zip(&s.corners) {
            cell.set(c);
        }
        out[4].set(s.near);
        out[5].set(s.far);
        Ok(s.value)
    }
}

// g_fogTable: 0x1000 bytes

/// `race::fog_table` (0x44FC33): `draw_distance_settings`; the noise at the car
/// (`sample_noise` of `g_noiseBytes`) spread between `dword_609308` and `dword_609320` into
/// the step `dword_62DAF0`; `dword_6365F0` / `dword_6365F4` = (`dword_6415DC` x
/// `dword_6415D8` >> 4) x `dword_609310` / `dword_609360` >> 8; `g_fogTableLength` =
/// `dword_63DC58` x 8 bytes of `g_fogTable`, each ((sum of steps) >> 16) + `dword_609358` +
/// `dword_6365F0` held to 0 ..= 0xFF; `dword_636610` the next one's value with
/// `dword_60930C` (held) plus `dword_6365F0` (held). Refuses (before anything) a table length of 0 or past 0x1000.
pub fn fog_table() -> Result<(), Refused> {
    let length = RACE_VIEW.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    // Converted: `rc2k_core::race::view::fog_table`.
    draw_distance_settings();
    let noise = sample_noise(ENGINE_TABLES.noise_bytes.addr())?;
    let fog = rc2k_core::race::view::fog_table(
        noise,
        &rc2k_core::race::view::FogSettings {
            low: STAGE_PREFS.fog_low.get(),
            high: STAGE_PREFS.fog_high.get(),
            scale: [FRAME_WORK.sun_size.get(), FRAME_WORK.view_out.get()],
            near: STAGE_PREFS.fog_near.get(),
            far: STAGE_PREFS.fog_far.get(),
            start: STAGE_PREFS.fog_start.get(),
            end: STAGE_PREFS.fog_end.get(),
            length,
        },
    );
    LIGHT_WORK.fog_step.set(fog.step);
    LIGHT_STATE.fog_near.set(fog.near);
    LIGHT_STATE.fog_far.set(fog.far);
    LIGHT_STATE.fog_table_length.set_bits(length);
    // The table's bytes (any length up to 0x1000; a last part-dword keeps its other bytes).
    for (cell, b) in LIGHT_STATE.fog_table.iter().zip(fog.table.chunks(4)) {
        let mut d = cell.get().to_le_bytes();
        d[..b.len()].copy_from_slice(b);
        cell.set(u32::from_le_bytes(d));
    }
    LIGHT_STATE.fog_next.set(fog.next);
    Ok(())
}

/// `race::set_up_fog` (0x44FBD9): `scene::restart_clock`; `dword_624000` = the fog colour
/// from `dword_6092CC` (red), `dword_6092D0`, `dword_6092D4` (each shifted into its byte
/// and added); `fog_table`; the colour with 0xFF000000 to the device. None of it reads a
/// register at entry, and its callers (`race::draw_view_scene`, `race::mirror_view_window`)
/// read none it leaves. Refuses (before anything) what `fog_table` would.
pub fn set_up_fog() -> Result<(), Refused> {
    let length = RACE_VIEW.view_depth[1].get().wrapping_shl(3);
    if length == 0 || length > 0x1000 {
        return Err(BadInput("a fog table length of 0 or past 0x1000").into());
    }
    crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    // Converted: `rc2k_core::race::view::{fog_colour, device_fog_colour}`.
    let colour = rc2k_core::race::view::fog_colour(
        STAGE_PREFS.fog_colour[0].bits(),
        STAGE_PREFS.fog_colour[1].bits(),
        STAGE_PREFS.fog_colour[2].bits(),
    );
    DRAW.fog_colour.set_bits(colour);
    fog_table()?;
    crate::scene::set_fog_colour::set_fog_colour(rc2k_core::race::view::device_fog_colour(
        DRAW.fog_colour.bits(),
    ));
    Ok(())
}

/// `race::place_view` (0x4389A8): the view's place from the camera (`dword_5CCDA0`,
/// `dword_5CCDC0`, `dword_5CCDE0`) and its angles. Flat (`dword_60932C` set): the place as
/// it is, `g_car` +0x6A8 += `dword_5CCD80`, `g_viewAngles` = `dword_5CCD60`,
/// `dword_6062BC` = 0x40000000. Else the place moved by (-`dword_5CCD80`, 0, 0) turned by
/// `dword_5CCD40` (`math::turn_point_xz_ecx`) and that turned again by `dword_5CCD60`
/// (`math::turn_point_xz`, its x 0, y the first turn's z, z its x), `g_viewAngles` and
/// `dword_6062BC` the two angles, and the place also as whole numbers and floats
/// (`dword_6062B0` / `dword_6062B4`, `flt_6062A4` ..). `dword_6062C0` .. `dword_6062CC` 0
/// both ways.
pub fn place_view() -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    let place = &VIEW_VARS.view_place;
    place[0].set(COURSE.place_x.bits());
    place[1].set(COURSE.place_y.bits());
    place[2].set(COURSE.place_z.bits());
    let zero = || {
        for a in [2, 4, 3, 5] {
            VIEW_VARS.view_angles[a].set_bits(0);
        }
    };
    if STAGE_PREFS.flat_depth.get() != 0 {
        let at = &CAR.camera_position[1];
        at.set(at.get().wrapping_add(COURSE.flat_depth.bits()));
        VIEW_VARS.view_angles[0].set_bits(COURSE.place_heading.get());
        VIEW_VARS.view_angles[1].set(0x4000_0000u32);
        zero();
        return Ok(());
    }
    let first: u32 = COURSE.camera_first.get();
    let second: u32 = COURSE.place_heading.get();
    let back = (COURSE.flat_depth.get()).wrapping_neg();
    let [x1, z1, _] = crate::math::turn_point_xz_ecx::turn_point_xz_ecx(back as i32, 0, 0, first);
    let [x2, _, z2, _] = crate::math::turn_point_xz::turn_point_xz(0, z1 as i32, x1 as i32, second);
    place[0].set(place[0].get().wrapping_add(x2));
    place[1].set(place[1].get().wrapping_add(z1));
    place[2].set(place[2].get().wrapping_add(z2));
    VIEW_VARS.view_angles[0].set_bits(second);
    VIEW_VARS.view_angles[1].set(first);
    zero();
    VIEW_VARS.view_x.set(place[0].get());
    VIEW_VARS.view_place_f[0].set_bits(from_int(place[0].get() as i32));
    VIEW_VARS.view_place_f[1].set_bits(from_int(place[1].get() as i32));
    VIEW_VARS.view_z.set(place[2].get());
    VIEW_VARS.view_place_f[2].set_bits(from_int(place[2].get() as i32));
    Ok(())
}

/// `race::sky_matrix` (0x42725E): `g_madeObjectMatrix` for the sky: a turn about y by
/// -(`dword_609334` << 8), placed at (0, `dword_609318` << 8, 0) (into `dword_63501C` ..),
/// the copy with the place (`g_turnedMatrix`) scaled by (`dword_6241D0` << 4) / 16
/// (`dword_635038`, through `dword_62400C`), times `g_viewTurnMatrix`.
pub fn sky_matrix() -> Result<(), BadInput> {
    use crate::scene::matrix;
    let angle = (STAGE_PREFS.sky_turn.get() << 8).wrapping_neg();
    matrix::set_turn_y(angle);
    let height = STAGE_PREFS.sky_height.get() << 8;
    OBJECT_WORK.place[0].set(0u32);
    OBJECT_WORK.place[2].set(0u32);
    OBJECT_WORK.place[1].set(height);
    matrix::set_place([0, height, 0]);
    matrix::copy_matrix_with_place(
        &MATRICES.turn_y_matrix,
        &MATRICES.placed_matrix,
        &MATRICES.turned_matrix,
    );
    let size = DRAW.farthest_scaled.get() << 4;
    OBJECT_WORK.size.set(size);
    let scale = crate::math::x87::int_over_16(size as i32);
    DRAW.scratch.set_bits(scale);
    for cell in &MATRICES.turned_matrix {
        cell.set_bits(crate::math::x87::mul(cell.bits(), scale));
    }
    matrix::multiply_matrices(
        &MATRICES.turned_matrix,
        &MATRICES.view_turn_matrix,
        &MATRICES.made_object_matrix,
    );
    Ok(())
}

/// The end of `view_matrices` (0x4388E5 .. and 0x438950 ..): the place negated into
/// `g_placedMatrix` (kept also at 0x5EA1CC), times the view's turn into `g_viewMatrix`,
/// the turn undone into `g_viewBackMatrix`.
fn view_place_tail() -> Result<(), BadInput> {
    use crate::scene::{matrix, view_matrix};
    let p = crate::session::read_bits(&VIEW_VARS.view_place).map(u32::wrapping_neg);
    matrix::set_place(p);
    crate::math::cells::copy_cells(&MATRICES.placed_matrix, &MATRICES.view_place_matrix, 16);
    matrix::multiply_matrices(
        &MATRICES.placed_matrix,
        &MATRICES.view_turn_matrix,
        &MATRICES.view_matrix,
    );
    view_matrix::invert_turn(&MATRICES.view_turn_matrix, &MATRICES.view_back_matrix);
    Ok(())
}

/// `race::view_matrices` (0x4386A4): the view's matrices for the frame. With
/// `dword_63993C` set: `g_viewTurnMatrix` the kept turn (`flt_5EA18C`). Else for the car's
/// views 0 .. 3 (`g_car` +0x684) the car's turn undone (`g_crashMatrix`); for the others
/// the identity turned by the view's angles (fine y, fine x, z of `g_viewAngles`,
/// `dword_6062BC`, `dword_6062C0`, each negated; then again by `dword_6062C4` ..
/// `dword_6062CC`), each step `turn y`, `turn x`, `turn z` multiplied in. Views 1 and 2
/// then roll with the car (`g_car` +0x648 by its speed, eased into `g_car` +0x6A0, turned
/// about z). Then the place (see `view_place_tail`).
pub fn view_matrices() -> Result<(), BadInput> {
    use crate::scene::matrix::multiply_matrices;
    use crate::scene::view_matrix::{Axis, invert_turn, set_fine_turn};
    let (view_turn, turn_y, turn_x, turn_z) = (
        &MATRICES.view_turn_matrix,
        &MATRICES.turn_y_matrix,
        &MATRICES.turn_x_matrix,
        &MATRICES.turn_z_matrix,
    );
    let (turned, lit) = (&MATRICES.turned_matrix, &MATRICES.lit_matrix);
    if GROUND_WORK.view_mirrored.get() != 0 {
        crate::math::cells::copy_cells(&MATRICES.view_turn_copy, &view_turn, 16);
        view_place_tail()?;
        return Ok(());
    }
    let view: u32 = CAR.camera_view.get();
    if view <= 3 {
        invert_turn(&CAR.crash_matrix, view_turn);
    } else {
        crate::math::cells::copy_cells(&MATRICES.identity_matrix, &view_turn, 16);
        for angles in [[0usize, 1, 2], [3, 4, 5]] {
            set_fine_turn(
                Axis::Y,
                VIEW_VARS.view_angles[angles[0]].bits().wrapping_neg(),
            );
            set_fine_turn(
                Axis::X,
                VIEW_VARS.view_angles[angles[1]].bits().wrapping_neg(),
            );
            set_fine_turn(
                Axis::Z,
                VIEW_VARS.view_angles[angles[2]].bits().wrapping_neg(),
            );
            multiply_matrices(view_turn, turn_y, turned);
            multiply_matrices(turned, turn_x, lit);
            multiply_matrices(lit, turn_z, view_turn);
        }
    }
    if view == 1 || view == 2 {
        use crate::math::x87_load::{below_or_unordered, eased_roll, product4_to_int};
        let mut v = crate::math::x87::div(CAR.local_velocity[2].bits(), NUMBERS.ten.bits());
        if below_or_unordered(v, NUMBERS.zero.bits()) {
            v = NUMBERS.zero.bits();
        }
        if !below_or_unordered(v, NUMBERS.one.bits()) {
            v = NUMBERS.one.bits();
        }
        DRAW.scratch.set_bits(v);
        let roll = eased_roll(
            CAR.steering_raw.get() as i32,
            NUMBERS.k65536.bits(),
            NUMBERS.one_over_two_pi.bits(),
            v,
            NUMBERS.tenth.bits(),
            CAR.camera_roll.bits(),
            NUMBERS.half.bits(),
        );
        CAR.camera_roll.set_bits(roll);
        let angle = product4_to_int(
            roll,
            NUMBERS.k65536.bits(),
            NUMBERS.k4096.bits(),
            NUMBERS.one_over_two_pi.bits(),
        ) << 4;
        DRAW.scratch.set_bits(angle);
        set_fine_turn(Axis::Z, angle);
        multiply_matrices(view_turn, turn_z, turned);
        crate::math::cells::copy_cells(&turned, &view_turn, 16);
    }
    view_place_tail()?;
    Ok(())
}

/// `race::camera_view` (0x438325): the view for the frame: `dword_606274`, `dword_60626C`,
/// `dword_606270` from `g_car` +0x6F0, +0x69C, +0x684; then without `dword_621480`
/// `race::place_view`; else the place from the car's camera (`g_car` +0x6A4 .., the angles
/// from +0x6B0 .., `dword_606294` from +0x6C8) or, with `dword_63993C`, from the crash
/// (the kept turn made from `flt_5E9ECC` and `g_crashMatrix`, the place the car's raised by
/// 0x8000), written as whole numbers and floats (also to `g_car` +0x734 ..), and
/// `scene::set_view_window`; then `view_matrices`. None of it reads a register at entry,
/// and its one caller, `race::draw_view_scene`, reads none it leaves.
pub fn camera_view() -> Result<(), BadInput> {
    use crate::math::x87::from_int;
    VIEW_VARS
        .view_car_flag_b
        .set(CAR.camera_mode_is_2_or_3.get());
    VIEW_VARS
        .view_car_flag_a
        .set(CAR.camera_switch_share.bits());
    VIEW_VARS.view_car_stage.set(CAR.camera_view.get());
    if FRAME_SWITCHES.other_way.get() == 0 {
        place_view()?;
        view_matrices()?;
        return Ok(());
    }
    let place: [u32; 3] = if GROUND_WORK.view_mirrored.get() == 0 {
        VIEW_VARS.view_scale.set_bits(CAR.camera_zoom.bits());
        for (from, to) in [
            (&CAR.camera_heading, 0usize),
            (&CAR.camera_pitch, 1),
            (&CAR.camera_roll_turn, 2),
            (&CAR.camera_offset_turns[1], 3),
            (&CAR.camera_offset_turns[0], 4),
            (&CAR.camera_offset_turns[2], 5),
        ] {
            VIEW_VARS.view_angles[to].set_bits(from.get());
        }
        crate::session::read_bits(&CAR.camera_position)
    } else {
        crate::scene::matrix::multiply_matrices(
            &MATRICES.view_work,
            &CAR.crash_matrix,
            &MATRICES.turned_matrix,
        );
        crate::scene::view_matrix::invert_turn(&MATRICES.turned_matrix, &MATRICES.view_turn_copy);
        let kept = &CAR.unknown_6cc;
        kept[0].set(CAR.position[0].get());
        kept[1].set(CAR.position[1].get().wrapping_add(0x8000));
        kept[2].set(CAR.position[2].get());
        VIEW_VARS
            .view_scale
            .set_bits(VIEW_VARS.view_scale_kept.bits());
        [kept[0].get(), kept[1].get(), kept[2].get()]
    };
    for (k, &p) in place.iter().enumerate() {
        CAR.kept_view_place[k].set(p);
        VIEW_VARS.view_place[k].set(p);
    }
    VIEW_VARS.view_x.set(place[0]);
    VIEW_VARS.view_z.set(place[2]);
    VIEW_VARS.view_place_f[0].set_bits(from_int(place[0] as i32));
    VIEW_VARS.view_place_f[1].set_bits(from_int(place[1] as i32));
    VIEW_VARS.view_place_f[2].set_bits(from_int(place[2] as i32));
    crate::scene::set_view_window::set_view_window();
    view_matrices()?;
    Ok(())
}

/// `race::draw_sky` (0x455540): `race::fetch_view_value`; `dword_5CCCD4` 0; unless the
/// scene's mode (`dword_6365C4`) is 1 or 2: the sky (object 1 through `sky_matrix`, kept
/// at 0x5EA34C, full light, `dword_6365C8` 4 and `dword_621510` 1 while it is drawn by
/// parts 1 .. 4); then with a sun (`dword_609310` + `dword_609360` not 0): its direction
/// from `dword_609334` + `dword_609338` and `dword_60933C` (`flt_6415AC` ..), turned by the
/// sky's matrix (`dword_6415B8` ..), and when in front (z not below `flt_5C5E18`) its place
/// on the screen (`flt_6415D0` / `dword_6415C8` ..), `dword_6415C4` 1 when on the screen,
/// `dword_6415DC` its size (256 less half its distance from the centre, at least 0).
/// None of it reads a register at entry (the drawing gets its esi here), and its one
/// caller, `race::draw_view_scene`, reads none it leaves.
pub fn draw_sky() -> Result<(), Refused> {
    use crate::math::x87_load::{sun_direction, sun_on_screen, sun_size, tbyte_to_int, to_tbyte};
    fetch_view_value()?;
    COURSE.polygon_flags.set(0u32);
    let mode: u32 = LIGHT_STATE.draw_mode.get();
    if mode == 1 || mode == 2 {
        return Ok(());
    }
    sky_matrix()?;
    crate::math::cells::copy_cells(
        &MATRICES.made_object_matrix,
        &MATRICES.object_matrix_slots,
        16,
    );
    FRAME_SWITCHES.object_number.set_bits(1);
    LIGHT_STATE.view_kind.set_bits(4);
    OBJECT_WORK.light_lowest.set_bits(0xFF);
    OBJECT_WORK.light_highest.set_bits(0xFF);
    OBJECT_WORK.light_level.set_bits(0xFF);
    OBJECT_WORK.light_turns[0].set_bits(0);
    OBJECT_WORK.light_turns[1].set_bits(0);
    FRAME_SWITCHES.no_far_clip.set_bits(1);
    crate::scene::glow_render_states::glow_render_states();
    LIGHT_WORK.light_kind.set(0u32);
    // esi as the `rep movsd` of the matrix's copy leaves it: past `g_madeObjectMatrix`.
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        core::mem::offset_of!(SceneMatrices, made_object_matrix) + 0x40,
    )))?;
    crate::race::flush_draw_batch::flush_draw_batch()?;
    crate::scene::depth_buffer_on::depth_buffer(true);
    LIGHT_STATE.view_kind.set_bits(0);
    FRAME_SWITCHES.no_far_clip.set_bits(0);
    FRAME_WORK.sun_on_screen.set_bits(0);
    FRAME_WORK.sun_size.set_bits(0);
    if STAGE_PREFS
        .fog_near
        .get()
        .wrapping_add(STAGE_PREFS.fog_far.bits())
        == 0
    {
        return Ok(());
    }
    let angle = STAGE_PREFS
        .sun_turn
        .get()
        .wrapping_add(STAGE_PREFS.sky_turn.get())
        & 0xFF;
    DRAW.scratch.set_bits(angle);
    let dir = sun_direction(
        angle as i32,
        NUMBERS.sixty_four.bits(),
        FRAME_WORK.sun_depth.bits(),
        STAGE_PREFS.sun_height.get() as i32,
        NUMBERS.k98304.bits(),
    );
    for (n, &d) in dir.iter().enumerate() {
        FRAME_WORK.sun_floats[n].set_bits(d);
    }
    let turned = crate::scene::rotate_point::rotate_point(
        &MATRICES.made_object_matrix,
        [to_tbyte(dir[0]), to_tbyte(dir[1]), to_tbyte(dir[2])],
    );
    let place = turned.map(tbyte_to_int);
    for (n, &p) in place.iter().enumerate() {
        FRAME_WORK.sun_points[n].set_bits(p);
    }
    let z = crate::math::x87::from_int(place[2] as i32);
    if crate::math::x87_load::below_or_unordered(z, NUMBERS.k4096_a.bits()) {
        return Ok(());
    }
    let flat = STAGE_PREFS.flat_depth.get() != 0;
    let v = [
        place[0],
        place[1],
        place[2],
        COURSE.flat_depth.get(),
        DRAW.view_scale.bits(),
        DRAW.view_centre[1].bits(),
        DRAW.view_centre[0].bits(),
    ];
    let [fx, ix, fy, iy] = sun_on_screen(&v, flat, &DRAW.scratch);
    FRAME_WORK.sun_place[0].set(fx);
    FRAME_WORK.sun_hit[0].set(ix);
    FRAME_WORK.sun_place[1].set(fy);
    FRAME_WORK.sun_hit[1].set(iy);
    let on = ix < SCREEN.width.get() && iy < SCREEN.height.get();
    FRAME_WORK.sun_on_screen.set(u32::from(on));
    let size = sun_size(
        fx,
        DRAW.view_centre[0].bits(),
        fy,
        DRAW.view_centre[1].bits(),
        NUMBERS.half.bits(),
        NUMBERS.k256.bits(),
    );
    DRAW.scratch.set_bits(size);
    FRAME_WORK
        .sun_size
        .set(if (size as i32) < 0 { 0 } else { size });
    Ok(())
}

// g_keptReflectionNormals: 0x400 dwords

/// `race::start_kept_reflections` (0x451864): the 0x400 kept reflection normals cleared to
/// -999 (0xFFFFFC19) and `g_reflectionsKept` 1.
pub fn start_kept_reflections() -> Result<(), BadInput> {
    {
        for cell in &GROUND_WORK.kept_reflections[..0x400] {
            cell.set(0xFFFF_FC19);
        }
        RACE_VIEW.reflections_kept.set_bits(1u32);
    }
    Ok(())
}

/// `race::end_kept_reflections` (0x451881): `g_reflectionsKept` 0.
pub fn end_kept_reflections() {
    RACE_VIEW.reflections_kept.set_bits(0u32);
}

/// `race::draw_slot_object` (0x4626D1): race slot esi's object (+0, into `g_objectNumber`)
/// drawn, unless outside one player and modes 0 and 2 its car (object 0x50 + 0x14 x car)
/// is not the drawn view's (`dword_641680`): the light (lowest 0, highest 0xFF, level
/// `dword_6092E4`, strength `dword_60935C`, turns -`dword_609338` / `dword_60933C`,
/// `dword_6365C8` 1, `g_lightKind` 2), its turn (+0x10, 16 dwords to `flt_5EA444`) placed
/// at +4, +8 + (+0x5C), +0xC (into `dword_63501C` ..) and moved by the view (0x5EA1CC,
/// into `flt_5EA484`), times the view's turn into `g_madeObjectMatrix`; the reflections
/// kept around the drawing (`scene::draw_scene_object`, esi `g_turnedMatrix`). Refuses
/// (before anything) as `check_slot_object`.
pub fn draw_slot_object(slot: u32) -> Result<(), Refused> {
    use crate::scene::matrix;
    check_slot_object(slot)?;
    let object = get(slot)?;
    FRAME_SWITCHES.object_number.set(object);
    let mode: u32 = RACE_VARS.run_kind.bits();
    if RACE_VIEW.players.bits() != 1 && mode != 0 && mode != 2 {
        let n = object.wrapping_sub(0x50);
        if FRAME_WORK.listed_car_sounds[((n / 0x14) & 7) as usize].get()
            != RACE_VIEW.drawn_view.bits()
        {
            return Ok(());
        }
    }
    OBJECT_WORK.other_light.set_bits(0);
    OBJECT_WORK.light_lowest.set_bits(0);
    OBJECT_WORK.light_highest.set_bits(0xFF);
    OBJECT_WORK
        .light_level
        .set_bits(STAGE_PREFS.light_level.bits());
    OBJECT_WORK
        .light_strength
        .set_bits(STAGE_PREFS.light_strength.bits());
    OBJECT_WORK.light_turns[0].set_bits(STAGE_PREFS.sun_turn.get().wrapping_neg());
    OBJECT_WORK.light_turns[1].set_bits(STAGE_PREFS.sun_height.bits());
    LIGHT_STATE.view_kind.set_bits(1);
    LIGHT_WORK.light_kind.set_bits(2);
    crate::math::cells::cells_from(
        slot + 0x10,
        &MATRICES.object_turn_matrix,
        16,
        "a face, record or matrix",
    )?;
    let place = [
        get(slot + 4)?,
        get(slot + 8)?.wrapping_add(get(slot + 0x5C)?),
        get(slot + 0xC)?,
    ];
    for (k, &p) in place.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(p);
    }
    matrix::set_place(place);
    matrix::copy_matrix_with_place(
        &MATRICES.placed_matrix,
        &MATRICES.view_place_matrix,
        &MATRICES.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &MATRICES.object_turn_matrix,
        &MATRICES.matrix_scratch,
        &MATRICES.turned_matrix,
    );
    matrix::multiply_matrices(
        &MATRICES.turned_matrix,
        &MATRICES.view_turn_matrix,
        &MATRICES.made_object_matrix,
    );
    start_kept_reflections()?;
    // esi `g_turnedMatrix`, as the matrices' calls leave it.
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        core::mem::offset_of!(SceneMatrices, turned_matrix),
    )))?;
    end_kept_reflections();
    Ok(())
}

/// What `scene::draw_scene_object` would refuse for slot `at`'s object, before anything:
/// a car's object (0x50 .. 0xEF) without its block.
pub fn check_slot_object(at: u32) -> Result<(), Refused> {
    {
        let n = get(at)?.wrapping_sub(0x50);
        if n < 0xA0 && RACE_WORK.car_block((n / 0x14) as usize) == 0 {
            return Err(BadInput("a slot's car without its block").into());
        }
        Ok(())
    }
}

/// `race::draw_race_slots` (0x4621E5): `draw_slot_object` for each of the eight race slots
/// (0x78 bytes at `g_raceSlots`) whose +0 is not 0 (inside `pusha`). Refuses (before anything) as
/// `check_slot_object` for any.
pub fn draw_race_slots() -> Result<(), Refused> {
    let slots = crate::pointers::game_address(RACE_WORK.race_slots.as_ptr());
    for n in 0..8 {
        if get(slots + 0x78 * n)? != 0 {
            check_slot_object(slots + 0x78 * n)?;
        }
    }
    for n in 0..8 {
        let at = slots + 0x78 * n;
        if get(at)? != 0 {
            draw_slot_object(at)?;
        }
    }
    Ok(())
}

/// `race::draw_screen_quad` (0x44B364): a flat rectangle on the screen from x
/// `dword_623E10` .. `dword_623E14`, y `dword_623E20` .. `dword_623E24` (the four vertices
/// at 0x623E40, 0x20 bytes each, depth `flt_5C5E20`) in `dword_623DE8`'s colour: the
/// polygon's four work entries set (0x975510, 0x44 bytes each), clipped
/// (`scene::clip_polygon_to_screen`), and with three vertices or more drawn with the depth
/// buffer off (`g_polygonMode` 0, switches -1, 1, 0; `g_polygonDrawn` 1); the colour back
/// to 0xFFFFFF. Every register kept (its callers return at once). Refuses what the clip
/// refuses (after the entries are set: never seen with four vertices).
pub fn draw_screen_quad() -> Result<(), Refused> {
    DRAW.polygon_drawn.set(0u32);
    let entries = &POLYGON_WORK.entries[..4];
    for (k, e) in (0u32..).zip(entries) {
        // the quad vertex's record (the original's address of it)
        e.record.set(crate::scene::point_records::of_quad(k));
        e.u.set(0u32);
        e.v.set(0u32);
    }
    // Converted: `rc2k_core::race::loading::quad_corners`.
    let corners = rc2k_core::race::loading::quad_corners(
        RACE_VARS.quad_corners[0].bits() as i32,
        RACE_VARS.quad_corners[1].bits() as i32,
        RACE_VARS.quad_corners[4].bits() as i32,
        RACE_VARS.quad_corners[5].bits() as i32,
    );
    for (k, (x, y)) in corners.into_iter().enumerate() {
        let v = &RACE_VARS.quad_vertices[8 * k..8 * k + 8];
        v[3].set_bits(x.to_bits());
        v[7].set_bits(y.to_bits());
        v[6].set_bits(NUMBERS.k4096_b.bits());
    }
    DRAW.scratch.set_bits(corners[2].1.to_bits());
    for e in entries {
        e.point.set(u32::MAX);
        e.light.set(0u32);
    }
    DRAW.polygon_colour.set(RACE_VARS.quad_colour[0].get());
    DRAW.polygon_vertex_count.set(4u32);
    DRAW.polygon_next_record
        .set(crate::scene::polygon::pool_start());
    for k in 0..4u32 {
        POLYGON_WORK.list[k as usize].set(k);
    }
    POLYGON_WORK.list[4].set(0u32);
    DRAW.polygon_next_entry.set(4u32);
    // No callee reads a register (the clip keeps them; the depth buffer's switches and
    // the polygon's drawing take none).
    crate::scene::clip_polygon_to_screen::clip_polygon_to_screen()?;
    if DRAW.polygon_vertex_count.get() >= 3 {
        crate::scene::depth_buffer_on::depth_buffer(false);
        DRAW.polygon_textures[0].set(u32::MAX);
        RACE_VIEW.polygon_mode.set(0u32);
        DRAW.polygon_sides[0].set(1u32);
        DRAW.polygon_sides[1].set(0u32);
        crate::scene::draw_polygon::draw_polygon_in_game()?;
        RACE_VIEW.polygon_mode.set(0u32);
        DRAW.polygon_drawn.set(1u32);
        crate::scene::depth_buffer_on::depth_buffer(true);
    }
    DRAW.polygon_colour.set(0x00FF_FFFFu32);
    Ok(())
}

/// `race::draw_view_tint` (0x456092): the whole view (0 .. `dword_62408C`, 0 ..
/// `dword_624090`) as a screen quad in 0x204080 (`dword_623DE8`), switches -1, 1, 0, by
/// `draw_screen_quad`.
pub fn draw_view_tint() -> Result<(), Refused> {
    RACE_VARS.quad_corners[0].set(0u32);
    RACE_VARS.quad_corners[1].set(DRAW.view_size[0].get());
    RACE_VARS.quad_corners[4].set(0u32);
    RACE_VARS.quad_corners[5].set(DRAW.view_size[1].get());
    RACE_VARS.quad_colour[0].set(0x20_4080u32);
    DRAW.polygon_textures[0].set(u32::MAX);
    DRAW.polygon_sides[0].set(1u32);
    DRAW.polygon_sides[1].set(0u32);
    draw_screen_quad()
}

/// `race::add_car_depth_entry` (0x4574A0): car block esi's place (+0x134 .., into
/// `dword_63501C` ..) through `g_viewMatrix` (rounded into `dword_63502C` ..; x and y back
/// as floats, z the key: plus 0x2000000, shifted right 10), a depth entry of the key with
/// ebx plus 0x2000000. (The registers it leaves are read by no caller.) Refuses (before
/// anything) a depth entry past the 0x1000.
pub fn add_car_depth_entry(esi: u32, ebx: u32) -> Result<(), Refused> {
    use crate::math::x87::from_int;
    let n: u32 = RACE_VIEW.depth_entry_count.bits();
    if n >= SORT_MAX {
        return Err(BadInput("a depth entry past the 0x1000").into());
    }
    let place = crate::session::read_bits(&car_at(esi)?.position);
    for (k, &p) in place.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(p);
    }
    let matrix: [u32; 16] = crate::session::read_bits(&MATRICES.view_matrix);
    let mut t = [0u32; 3];
    crate::math::x87_load::rotate_rounding(
        place.as_ptr().cast::<i32>(),
        matrix.as_ptr(),
        t.as_mut_ptr(),
    );
    for (k, &v) in t.iter().enumerate() {
        OBJECT_WORK.turned_place[k].set_bits(v);
    }
    OBJECT_WORK.place[0].set(from_int(t[0] as i32));
    OBJECT_WORK.place[1].set(from_int(t[1] as i32));
    let key = t[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let second = ebx.wrapping_add(0x200_0000);
    let at = depth_entry(n)?;
    at[0].set(key);
    at[1].set(second);
    RACE_VIEW.depth_entry_count.set_bits(n + 1);
    Ok(())
}

/// The tables' 256 entries. The original counts to 0x400, each table running over the next
/// and the last past its end; flares past the tables are not added.
pub const FLARES_MAX: u32 = 0x100;

/// `sub_457332`: a flare at `dword_63501C` .. (object `g_objectNumber`, size
/// `dword_635038`, colour `dword_635054`) into the six flare tables (0x100 dwords each, from
/// `dword_63F18C`) and, turned by `g_viewMatrix` (x and y back as floats, z rounded into
/// the key), a depth entry with the flare's number + 0x1000000; none past `FLARES_MAX`.
fn add_flare() -> Result<(), BadInput> {
    let n: u32 = FRAME_WORK.flare_count.bits();
    if n == FLARES_MAX {
        return Ok(());
    }
    let place = [
        OBJECT_WORK.place[0].get(),
        OBJECT_WORK.place[1].bits(),
        OBJECT_WORK.place[2].bits(),
    ];
    for (table, v) in [
        (
            &FLARE_TABLES.object_number,
            FRAME_SWITCHES.object_number.get(),
        ),
        (&LEVEL_GRID.flare_place_x, place[0]),
        (&FLARE_TABLES.place_y, place[1]),
        (&FLARE_TABLES.place_z, place[2]),
        (&FLARE_TABLES.size, OBJECT_WORK.size.bits()),
        (&FLARE_TABLES.other_light, OBJECT_WORK.other_light.bits()),
    ] {
        table[n as usize].set(v);
    }
    let mut out = [0u32; 3];
    crate::math::x87_load::rotate_rounding_z(
        place.as_ptr().cast::<i32>(),
        crate::session::read_bits(&MATRICES.view_matrix).as_ptr(),
        out.as_mut_ptr(),
    );
    OBJECT_WORK.place[0].set(out[0]);
    OBJECT_WORK.place[1].set(out[1]);
    let key = out[2].wrapping_add(0x200_0000) >> 10;
    RACE_VIEW.face_depth_key.set_bits(key);
    let entry: u32 = RACE_VIEW.depth_entry_count.bits();
    let at = depth_entry(entry)?;
    at[0].set(key);
    at[1].set(n.wrapping_add(0x100_0000));
    RACE_VIEW.depth_entry_count.set_bits(entry + 1);
    FRAME_WORK.flare_count.set_bits(n + 1);
    Ok(())
}

/// `sub_4572B0`: a flare at `point` (eax, ebx, ecx) in car block `b`'s (esi) frame (its matrix +0xB4,
/// each part rounded, plus its place +0x134 ..), object 0x39, by `add_flare`.
fn place_flare(b: u32, point: [u32; 3]) -> Result<(), BadInput> {
    let mut t = [0u32; 3];
    crate::math::x87_load::rotate_rounding(
        point.as_ptr().cast::<i32>(),
        crate::session::read_bits(&car_at(b)?.crash_matrix).as_ptr(),
        t.as_mut_ptr(),
    );
    DRAW.scratch.set_bits(t[2]);
    for (k, v) in t.iter().enumerate() {
        OBJECT_WORK.place[k].set_bits(v.wrapping_add(car_at(b)?.position[k].bits()));
    }
    FRAME_SWITCHES.object_number.set(0x39u32);
    add_flare()?;
    Ok(())
}

/// `sub_457241`: a pair of lights at the light record's (edi) +`at` (ecx; three dwords:
/// none when all are 0) for block `b` (esi): the right one (x as it is) when the block's
/// byte +`right` (edx) is set, the left (x negated) when +`left` (ebx) is (-1: no byte), each by `place_flare` inside `pusha`;
/// `dword_641758` / `dword_64175C` the two bytes.
fn light_pair(b: u32, record: u32, at: u32, left: u32, right: u32) -> Result<(), BadInput> {
    let byte = |o: u32| -> Result<u32, BadInput> {
        Ok(GamePtr::new(b.wrapping_add(o)).read::<u8>("a car block's light byte")? as u32)
    };
    FRAME_WORK.light_left.set(0u32);
    FRAME_WORK.light_right.set(0u32);
    if left != u32::MAX {
        FRAME_WORK.light_left.set(byte(left)?);
    }
    if right != u32::MAX {
        FRAME_WORK.light_right.set(byte(right)?);
    }
    let (x, y, z) = (
        get(record + at)?,
        get(record + at + 4)?,
        get(record + at + 8)?,
    );
    if x | y | z == 0 {
        return Ok(());
    }
    let y = y.wrapping_sub(PHYSICS.ground_drop.bits());
    if FRAME_WORK.light_right.get() != 0 {
        place_flare(b, [x, y, z])?;
    }
    if FRAME_WORK.light_left.get() != 0 {
        place_flare(b, [x.wrapping_neg(), y, z])?;
    }
    Ok(())
}

/// `race::car_light_flares` (0x457036): car block `b`'s (esi) lights as flares. Refuses (before anything) a flare list or depth
/// list without room for the 24 flares a car can give.
pub fn car_light_flares(b: u32) -> Result<(), Refused> {
    if RACE_VIEW.depth_entry_count.bits() > SORT_MAX - 24 {
        return Err(BadInput("the depth list has no room for a car's flares").into());
    }
    if FRAME_SWITCHES.mask_kind.get() == 8 {
        OBJECT_WORK.size.set(2u32);
        let point = [
            FRAME_WORK.part_place[0].bits(),
            FRAME_WORK.part_place[1]
                .get()
                .wrapping_sub(PHYSICS.ground_drop.bits()),
            FRAME_WORK.part_place[2].bits(),
        ];
        OBJECT_WORK.other_light.set(0x00F0_F0F0u32);
        place_flare(b, point)?;
        // Not the original's: the pair's other light, as `light_pair` (the debug UI).
        if point[0] != 0 {
            place_flare(b, [point[0].wrapping_neg(), point[1], point[2]])?;
        }
        return Ok(());
    }
    if FRAME_SWITCHES.other_way.bits() == 0 {
        return Ok(());
    }
    let kind = car_at(b)?.car_kind.bits();
    FRAME_WORK.part_kind.set(kind);
    let valid = kind < 0x22 && KIND_TABLES.kind_shows_parts[kind as usize].bits() != 0;
    if valid {
        if !crate::math::x87_load::below_or_unordered(
            car_at(b)?.electrics_damage.bits(),
            NUMBERS.hundred.bits(),
        ) {
            return Ok(());
        }
        FRAME_WORK
            .parts_present
            .set(car_at(b)?.parts_present.bits());
        // A second test of the same kind follows; its error branch (a log line) is
        // never reached.
    }
    // `sub_41F4E1`: the kind held to 0 ..= 0x21 (by sign), through `dword_5C59B0`.
    let index = KIND_TABLES.kind_light_records[(kind as i32).clamp(0, 0x21) as usize].bits();
    let record = LIGHT_RECORDS.record_addr(index);
    LIGHT_STATE.view_kind.set(1u32);
    LIGHT_WORK.light_kind.set(3u32);
    OBJECT_WORK.size.set(2u32);
    OBJECT_WORK.other_light.set(0x0060_6060u32);
    let pair = |at: u32, left: u32, right: u32| light_pair(b, record, at, left, right);
    pair(0xA4, 0xAC0, 0xAC1)?;
    if LIGHT_WORK.marks_mask.get() != 0 && FRAME_WORK.parts_present.get() >> 9 & 1 != 0 {
        pair(0xB0, 0xACA, 0xACB)?;
        pair(0xBC, 0xACC, 0xACD)?;
        pair(0xC8, 0xACE, 0xACF)?;
    }
    // `sub_45721A`: no left light for kind 0xD.
    let left = if kind == 0xD { u32::MAX } else { 0xAC6 };
    pair(0xF8, left, 0xAC7)?;
    OBJECT_WORK.other_light.set(0x00C0_0000u32);
    // `sub_457229`: none for kinds 0xD and 0xE.
    let left = if kind == 0xD || kind == 0xE {
        u32::MAX
    } else {
        0xAC8
    };
    pair(0x104, left, 0xAC9)?;
    OBJECT_WORK.other_light.set(0x0080_0000u32);
    pair(0xD4, 0xAC4, 0xAC5)?;
    pair(0xE0, 0xAD0, u32::MAX)?;
    pair(0xEC, 0xAC2, 0xAC3)?;
    Ok(())
}

/// `race::car_lights` (0x455CFF): with `dword_621480` set, for each race car: with +0x6FC
/// bit 5 (lights on, `car_light_modes`) its depth entry (`add_car_depth_entry`, the car
/// in ebx) and then `car_light_flares` (inside `pusha`). Refuses (before
/// anything) no cars (the loop tests at its end) or more than eight, a null block, or a
/// depth list without room for every car's entries.
pub fn car_lights() -> Result<(), Refused> {
    if FRAME_SWITCHES.other_way.bits() == 0 {
        return Ok(());
    }
    // Converted: `rc2k_core::race::lights::{check_car_lights, lights_on}` (the null
    // block's refusal between its two, as before).
    use rc2k_core::race::lights::{check_car_lights, lights_on};
    let cars: u32 = RACE_VIEW.race_cars.bits();
    check_car_lights(cars, 0, SORT_MAX)?;
    let block = |n: u32| RACE_WORK.car_block(n as usize);
    if (0..cars).any(|n| block(n) == 0) {
        return Err(BadInput("a null car block").into());
    }
    check_car_lights(cars, RACE_VIEW.depth_entry_count.bits(), SORT_MAX)?;
    for n in 0..cars {
        let b = block(n);
        // Both only with the lights on: the `jz` skips to the `popa`.
        if lights_on(car_at(b)?.state_bits.bits()) {
            add_car_depth_entry(b, n)?;
            car_light_flares(b)?;
        }
    }
    Ok(())
}

/// The end of `place_object_matrix`'s ways: the place `dword_63501C` .. into
/// `g_placedMatrix`, moved by the view (0x5EA1CC into `flt_5EA484`), `flt_5EA444` with it
/// (`g_turnedMatrix`), times the view's turn into `g_madeObjectMatrix`.
fn placed_object_tail() {
    use crate::scene::matrix;
    let place = [
        OBJECT_WORK.place[0].get(),
        OBJECT_WORK.place[1].bits(),
        OBJECT_WORK.place[2].bits(),
    ];
    matrix::set_place(place);
    matrix::copy_matrix_with_place(
        &MATRICES.placed_matrix,
        &MATRICES.view_place_matrix,
        &MATRICES.matrix_scratch,
    );
    matrix::copy_matrix_with_place(
        &MATRICES.object_turn_matrix,
        &MATRICES.matrix_scratch,
        &MATRICES.turned_matrix,
    );
    matrix::multiply_matrices(
        &MATRICES.turned_matrix,
        &MATRICES.view_turn_matrix,
        &MATRICES.made_object_matrix,
    );
}

/// `flt_5EA444`'s sixteen floats times `dword_635038` / 16 (through `dword_62400C`).
fn scale_object_turn() -> Result<(), BadInput> {
    {
        let scale = crate::math::x87::int_over_16(OBJECT_WORK.size.get() as i32);
        DRAW.scratch.set_bits(scale);
        for cell in &MATRICES.object_turn_matrix {
            cell.set_bits(crate::math::x87::mul(cell.bits(), scale));
        }
    }
    Ok(())
}

/// `race::place_object_matrix` (0x42739A): `g_madeObjectMatrix` for an object placed at
/// `dword_63501C` .. (after `scene::set_light_direction`). With `dword_6365D0` 0: turned
/// about y, x and z (16 bits a turn) by `dword_635040`, `dword_63503C`, `dword_635044`
/// negated, unscaled. Else (nothing more with `dword_6365C8` 4) by the object's
/// material (+0x20 of its kind's 0x80-byte record at `g_objectMaterials`): 1 and 2 facing
/// the view (`g_viewBackMatrix`); 5 turned about y by `dword_6062C4` and the view's y
/// angle (fine turns); else about y by -`dword_63503C`, then with `dword_635040` tilted
/// about x round the y of `dword_635044`, and with `dword_635060` / `dword_635064` again
/// round `dword_63505C`; then scaled by `dword_635038` / 16. (It reads no register, and
/// its callers read none it leaves.)
pub fn place_object_matrix() -> Result<(), BadInput> {
    use crate::scene::matrix::{
        multiply_matrices, set_light_direction, set_turn_x, set_turn_y, set_turn_z_16,
    };
    use crate::scene::view_matrix::{Axis, set_fine_turn};
    let (turn_y, turn_x, turn_z) = (
        &MATRICES.turn_y_matrix,
        &MATRICES.turn_x_matrix,
        &MATRICES.turn_z_matrix,
    );
    let (turned, lit, object) = (
        &MATRICES.turned_matrix,
        &MATRICES.lit_matrix,
        &MATRICES.object_turn_matrix,
    );
    set_light_direction();
    if LIGHT_STATE.light_turn_off.get() == 0 {
        set_turn_y(OBJECT_WORK.angles[1].bits().wrapping_neg());
        set_turn_x(OBJECT_WORK.angles[0].bits().wrapping_neg());
        multiply_matrices(turn_y, turn_x, turned);
        set_turn_z_16(OBJECT_WORK.angles[2].bits().wrapping_neg());
        multiply_matrices(turned, turn_z, object);
        placed_object_tail();
        return Ok(());
    }
    if LIGHT_STATE.view_kind.get() == 4 {
        return Ok(());
    }
    let kind = OBJECT_TABLES.object_kind(FRAME_SWITCHES.object_number.get());
    let material = POLYGON_WORK
        .object_materials
        .get(kind as usize)
        .ok_or(BadInput("an object kind past the 1200 materials"))?
        .words[8]
        .get() as u8; // the record's byte +0x20
    match material {
        1 | 2 => {
            crate::math::cells::copy_cells(&MATRICES.view_back_matrix, &object, 16);
        }
        5 => {
            crate::math::cells::copy_cells(&MATRICES.identity_matrix, &object, 16);
            set_fine_turn(Axis::Y, VIEW_VARS.view_angles[3].get());
            multiply_matrices(object, turn_y, turned);
            set_fine_turn(Axis::Y, VIEW_VARS.view_angles[0].get());
            multiply_matrices(turned, turn_y, object);
        }
        _ => {
            set_turn_y(OBJECT_WORK.angles[0].bits().wrapping_neg());
            crate::math::cells::copy_cells(&turn_y, &object, 16);
            if OBJECT_WORK.angles[1].get() != 0 {
                set_turn_y(OBJECT_WORK.angles[2].bits().wrapping_neg());
                multiply_matrices(object, turn_y, lit);
                set_turn_x(OBJECT_WORK.angles[1].bits().wrapping_neg());
                multiply_matrices(lit, turn_x, turned);
                set_turn_y(OBJECT_WORK.angles[2].get());
                multiply_matrices(turned, turn_y, object);
            }
            if OBJECT_WORK.described_cleared[2].get() != 0
                || OBJECT_WORK.described_cleared[1].get() != 0
            {
                set_turn_y(OBJECT_WORK.described_cleared[2].bits().wrapping_neg());
                multiply_matrices(object, turn_y, lit);
                set_turn_y(OBJECT_WORK.described_cleared[0].bits().wrapping_neg());
                multiply_matrices(lit, turn_y, turned);
                set_turn_x(OBJECT_WORK.described_cleared[1].bits().wrapping_neg());
                multiply_matrices(turned, turn_x, lit);
                set_turn_y(OBJECT_WORK.described_cleared[0].get());
                multiply_matrices(lit, turn_y, object);
            }
        }
    }
    scale_object_turn()?;
    placed_object_tail();
    Ok(())
}

/// `race::draw_flare` (0x455189): flare `flare` (ebx) of the six flare tables back into
/// `dword_63501C` .., `dword_635038`, `dword_635054`, `g_objectNumber`; `dword_6365C8` 2,
/// `place_object_matrix`, `g_lightKind` 0 and the drawing (`scene::draw_scene_object`,
/// `esi` the esi at entry: the list entry). Every register kept.
pub fn draw_flare(flare: u32, esi: GamePtr) -> Result<(), Refused> {
    for (table, to) in [
        (&LEVEL_GRID.flare_place_x, &OBJECT_WORK.place[0]),
        (&FLARE_TABLES.place_y, &OBJECT_WORK.place[1]),
        (&FLARE_TABLES.place_z, &OBJECT_WORK.place[2]),
        (&FLARE_TABLES.size, &OBJECT_WORK.size),
        (&FLARE_TABLES.other_light, &OBJECT_WORK.other_light),
        (&FLARE_TABLES.object_number, &FRAME_SWITCHES.object_number),
    ] {
        to.set_bits(
            table
                .get(flare as usize)
                .ok_or(BadInput("a flare past the tables"))?
                .bits(),
        );
    }
    LIGHT_STATE.view_kind.set(2u32);
    place_object_matrix()?;
    LIGHT_WORK.light_kind.set(0u32);
    // The `popa` and `pusha` between: the drawing gets the esi it had at entry.
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(esi))
}

/// `race::draw_lamp_flare` (0x4551F4): lamp `lamp` (ebx; `g_lamps`): its point (`g_scenePoints`)
/// into `dword_63501C` .., its +4 x 8 the size, +8 the colour, object `dword_621508`;
/// `dword_6365C8` 2, `place_object_matrix`, `g_lightKind` 0 and the drawing. Every
/// register kept.
pub fn draw_lamp_flare(lamp: u32) -> Result<(), Refused> {
    let edi = self::lamp(lamp)?;
    let [x, y, z] = crate::race::scene_points::point_number(edi[0].get())?;
    OBJECT_WORK.place[0].set(x);
    OBJECT_WORK.place[1].set(y);
    OBJECT_WORK.place[2].set(z);
    OBJECT_WORK.size.set(edi[1].get() << 3);
    OBJECT_WORK.other_light.set(edi[2].get());
    FRAME_SWITCHES
        .object_number
        .set(FRAME_SWITCHES.marker_object.get());
    LIGHT_STATE.view_kind.set(2u32);
    place_object_matrix()?;
    LIGHT_WORK.light_kind.set(0u32);
    // esi `g_turnedMatrix`, as `place_object_matrix` leaves it (`dword_6365C8` is 2 here,
    // so it always ends by placing the object).
    crate::scene::draw_scene_object::draw_scene_object_in_game(Some(matrices_at(
        core::mem::offset_of!(SceneMatrices, turned_matrix),
    )))
}

/// `race::draw_list_entry` (0x454FF0): the entry at `at` (esi; its number in ecx is read by
/// no drawing), its second word
/// `ebx` (into `dword_5CCCD4`;
/// the top bit dropped): below 0x1000000 a course face (`g_currentCourseFace`): kind 0xFF
/// part 1, below 0x10 part 2, each with `dword_63E164` .. `dword_63E16C` 1 around it;
/// 0x1000000 .. a flare, 0x2000000 .. a car, 0x3000000 .. a lamp, each by its low 24 bits; then `dword_6365C8` 0. Every register kept. Refuses (before
/// anything) a word of 0x4000000 or more (the original would take it for a face far past
/// the table).
pub fn draw_list_entry(at: GamePtr, ebx: u32) -> Result<(), Refused> {
    let word = ebx & 0x7FFF_FFFF;
    if word >= 0x400_0000 {
        return Err(BadInput("a list entry of 0x4000000 or more").into());
    }
    COURSE.polygon_flags.set(ebx);
    // What the drawings read of the registers the original gives them: the face
    // drawing esi (the face), the car drawing ebx (the word's low 24 bits).
    let low = word & 0xFF_FFFF;
    match word >> 24 {
        0 => {
            COURSE.current_course_face.set_bits(word);
            let face = word;
            let kind = crate::race::course_places::face_field::<u8>(face, 0, "a course face")?;
            if kind == 0xFF || kind < 0x10 {
                LEVEL_GRID.smooth.set_bits(1);
                LEVEL_GRID.plain_first.set_bits(1);
                LEVEL_GRID.plain_second.set_bits(1);
                if kind == 0xFF {
                    // The description is the face (esi in the original).
                    crate::scene::draw_described_object::draw_described_face(face)?;
                } else {
                    crate::race::course_faces::draw_tinted_face(face)?;
                }
                LEVEL_GRID.smooth.set_bits(0);
                LEVEL_GRID.plain_first.set_bits(0);
                LEVEL_GRID.plain_second.set_bits(0);
            }
        }
        1 => draw_flare(low, at)?,
        2 => crate::race::car_draw::draw_car_entry(low)?,
        _ => draw_lamp_flare(low)?,
    }
    LIGHT_STATE.view_kind.set(0u32);
    Ok(())
}

/// `race::draw_sorted_entries` (0x454EAB) / `race::draw_depth_entries` (0x454E84): each
/// entry of the list (`count` its count, `entries` its 8-byte entries) by
/// `draw_list_entry`. Refuses (before anything) what `draw_list_entry` would for any.
/// `entries` the list.
pub fn draw_entries(count: &crate::session::Dword, entries: GamePtr) -> Result<(), Refused> {
    let n: u32 = count.get();
    let list = entries;
    let trap = Trap::new();
    let too_big = (0..n).any(|k| {
        trap.read_at::<u32>(list.offset(8 * k + 4), "a draw list entry") & 0x7FFF_FFFF >= 0x400_0000
    });
    trap.finish()?;
    if too_big {
        return Err(BadInput("a list entry of 0x4000000 or more").into());
    }
    for k in 0..n {
        let at = list.offset(8 * k);
        draw_list_entry(at, at.offset(4).read::<u32>("a draw list entry")?)?;
    }
    Ok(())
}

/// The scene matrices from byte `off` on, as the esi the original leaves for a drawing
/// (`scene::draw_described_object` reads a description there): bounded by the whole
/// store, since a description's 0x3C bytes run past one matrix.
fn matrices_at(off: usize) -> GamePtr {
    GamePtr::to(&MATRICES).offset(off as u32)
}
