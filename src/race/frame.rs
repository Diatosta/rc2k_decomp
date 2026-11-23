//! The race's frame outside the physics (`drive::race_loop`'s callees): the players' states, the
//! cars' counters, the view cars loaded into the work copy. This file holds its leaves.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// Carry when every player is out (`stop_state` not 0): the chosen car's with `session` 1, else
/// each of the `players` players'. Refused, before anything: a chosen car past 7 or more than 8
/// players.
pub fn players_all_out(
    race: &Race,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<bool, Refused> {
    let shared = &race.shared;
    use rc2k_core::race::cars::{Players, players_all_out};
    {
        let trap = crate::race::trap::Trap::new();
        let out =
            |n: u32| trap.keep_read(shared.race_car(n as usize).map(|car| car.stop_state)) != 0;
        let players = if let Some(net_race) = net_race.get() {
            Players::Chosen(net_race.player)
        } else {
            Players::First(shared.view.get().players.get())
        };
        let all = players_all_out(players, out)?;
        trap.finish()?;
        Ok(all)
    }
}

/// Writes a car block's message in the order `notice_id`, `notice_time_b`, `notice_time`.
fn set_car_counter(car: &mut Car, value: u32) {
    let m = rc2k_core::race::cars::Message::new(value);
    car.notice_id = m.id;
    car.notice_time_b = m.age;
    car.notice_time = m.time_left;
}

/// Calls `set_car_counter` on the first two blocks, each with the same value.
fn set_both_counters(race: &Race, value: u32) {
    for car in &mut race.car_blocks_mut().cars[..2] {
        set_car_counter(car, value);
    }
}

/// The view car `n`'s number: `view_cars`, `RaceView::view_cars[1]`, `local_car`.
pub fn view_car(
    view: &crate::race::view_store::RaceView,
    n: usize,
) -> &crate::memory::session::PlainDword {
    match n {
        0 => &view.view_cars[0],
        1 => &view.view_cars[1],
        2 => &view.local_car,
        _ => panic!("a view car past the third"),
    }
}

/// Copies view car `n`'s block (of `blocks`: the game thread's, or a drawing part's reads) into
/// `view`'s car (`ViewScratch::car`). Refused: a null block, before anything.
pub(super) fn load_view_car(
    work: &crate::race::work_store::RaceWork,
    blocks: crate::memory::drawn_table::DrawnTable<'_, crate::race::blocks_store::CarBlocks>,
    race_view: crate::race::view_store::DrawnViewRef<'_>,
    n: usize,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(race_view.view_car_number(n)?);
    car_block(u32::from(work.has_car_block(slot as usize)))?;
    let block = blocks.read(|blocks| {
        crate::race::owner_shared::RaceShared::race_car_in(work, blocks, slot as usize).cloned()
    })?;
    *view.car.borrow_mut() = block;
    Ok(())
}

// The race keys, by PC scan code (set 1) in `keys_pressed`: F1 .. F5 (0x3B ..), F6 .. F12 for the
// view block's option bits, or the number row standing in for them. A modifier held in `keys_now`
// picks the player they act on (`player` below): Ctrl the second, Alt the third, Shift the fourth.
const LEFT_CTRL_KEY: usize = 0x1D;
const NEXT_VIEW_KEY: usize = 0x3B;
const SPLIT_KEY: usize = 0x3C;
const DOWN_KEY: usize = 0x3D;
const UP_KEY: usize = 0x3E;
const SHARED_KEY: usize = 0x3F;
/// The keys that flip a bit of the view block's `option_bits` (`rc2k_core::race::keys::BIT_KEYS` by
/// place): their key bytes, in the order tested.
pub const BIT_KEYS: [usize; 7] = [0x58, 0x42, 0x41, 0x44, 0x43, 0x40, 0x57];
fn pressed(devices: &crate::input::store::Devices, key: usize) -> bool {
    devices.keys_pressed.get(key) != 0
}

/// The modifiers that pick a race key's player (`rc2k_core::race::keys::key_player`), either
/// side, by key code.
const CTRL_KEYS: [usize; 2] = [LEFT_CTRL_KEY, 0x61];
const ALT_KEYS: [usize; 2] = [0x38, 0x64]; // right Alt is AltGr
const SHIFT_KEYS: [usize; 2] = [0x2A, 0x36];

/// Player `player`'s car block, which the race keys change: the first two by their views (as the
/// original's Ctrl chose the second view), the third and fourth by their own cars; none for a
/// player the race does not have.
fn player_car(race: &Race, player: usize) -> Result<Option<std::cell::RefMut<'_, Car>>, Refused> {
    let shared = &race.shared;
    if player < 2 {
        return Ok(Some(shared.view_car_or_end_mut(player)));
    }
    if player >= shared.view.get().players.get() as usize {
        return Ok(None);
    }
    Ok(Some(
        shared.race_car_or_mut(player, "a further player's car block")?,
    ))
}

/// The F key of a race key.
fn f_key(key: rc2k_core::race::keys::Key) -> usize {
    use rc2k_core::race::keys::Key;
    match key {
        Key::NextView => NEXT_VIEW_KEY,
        Key::Split => SPLIT_KEY,
        Key::VolumeDown => DOWN_KEY,
        Key::VolumeUp => UP_KEY,
        Key::Bit(n) => BIT_KEYS[n],
        Key::Shared => SHARED_KEY,
    }
}

/// Whether one of the players' keyboard controls uses key `code`.
fn bound_to_controls(menus: &crate::menu::owner::Menus, code: usize) -> bool {
    menus
        .play
        .slots
        .records
        .iter()
        .any(|p| p.get().keys.contains(&(code as u8)))
}

/// The race's view and sound keys.
pub fn race_keys(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (audio, sound) = (o.audio, o.sound);
    let devices = &o.input.devices;
    let state = &audio.state;
    let shared = &race.shared;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    use rc2k_core::race::keys::{
        BIT_KEYS as BITS, COUNTED_VIEWS, Key, SHARED_BIT, first_key, flip_bit, key_player,
        number_row_for, split_flip, volume_key, volume_step,
    };
    let held = |code: usize| devices.keys_now.get(code) != 0;
    // A race key is down by its F key or by the number-row key standing in for it, unless a
    // player's keyboard controls use that number key.
    let by_row = |k: Key| {
        number_row_for(f_key(k))
            .is_some_and(|row| pressed(devices, row) && !bound_to_controls(o.menus, row))
    };
    let key = first_key(|k| pressed(devices, f_key(k)) || by_row(k));
    let any = |keys: [usize; 2]| keys.into_iter().any(held);
    let player = key_player(any(CTRL_KEYS), any(ALT_KEYS), any(SHIFT_KEYS));
    // The view block's `option_bits` with `bit` flipped; gives the counter.
    let flip = |view: &mut Car, bit: (u32, u32, u32)| -> u32 {
        let (now, counter) = flip_bit(view.option_bits, bit);
        view.option_bits = now;
        counter
    };
    match key {
        Some(Key::NextView) => {
            let Some(mut view) = player_car(race, player)? else {
                return Ok(());
            };
            crate::drive::next_camera_view::next_camera_view(race, &mut view)?;
            let camera = view.camera_blend_view;
            if camera < COUNTED_VIEWS {
                set_car_counter(&mut view, shared.run().view_counters[camera as usize]);
            }
        }
        Some(Key::Split) => {
            let players = shared.view.get().players.get();
            if players == 1 {
                return Ok(());
            }
            if players > 2 {
                // Three or four players cycle through their layouts (`race::split_layout`).
                crate::race::split_layout::cycle(players);
                return Ok(());
            }
            let (split, counter) = split_flip(shared.vars.get().split.get());
            shared.vars.get().split.set(split);
            set_both_counters(race, counter);
        }
        Some(k @ (Key::VolumeDown | Key::VolumeUp)) => {
            let up = k == Key::VolumeUp;
            // Any player's modifier turns the music instead of the master volume (the original's
            // Ctrl).
            let music = player != 0;
            let (step, counter) = volume_key(up, music);
            set_both_counters(race, counter);
            if music {
                let from = sound.menu.options.race_music_volume.get();
                let volume = volume_step(from, step);
                sound.menu.options.race_music_volume.set(volume);
                crate::sound::set_music_volume::set_music_volume(volume, o);
            } else {
                let volume = volume_step(state.master_volume.get(), step);
                state.master_volume.set(volume);
                // The sound command 8 with the volume.
                crate::sound::send_command_8::send_command_8(audio, volume);
            }
        }
        Some(Key::Bit(n)) => {
            let Some(mut view) = player_car(race, player)? else {
                return Ok(());
            };
            let counter = flip(&mut view, BITS[n]);
            set_car_counter(&mut view, counter);
        }
        Some(Key::Shared) => {
            let counter = flip(&mut shared.view_car_or_end_mut(0), SHARED_BIT);
            set_both_counters(race, counter);
        }
        None => {}
    }
    Ok(())
}

/// Copies the textures (`TyreChange::texture` / `device_texture` from
/// `Textures::first_textures[599]` / `KindSlots.kind_first_slots[599]`); for
/// car 0 (block: the count `CarBlocks.cars[0].stopped_steps`, the change's state
/// `tyre_change_state`, the tyre bits `parts_present`, the time `stage_time`, and
/// `tyre_wears[0]` .. `tyre_wears[3]`, one per bit 1 .. 4) starts `tyre_change_gauge`,
/// then runs it, adding its time when done.
pub fn tyre_change(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (race, scene) = (o.race, o.scene);
    let shared = &race.shared;
    let textures = &scene.textures;
    use crate::race::tyre_change_gauge::tyre_change_gauge;
    use rc2k_core::scene::texture_levels::TYRE_TITLE_TEXTURE;
    {
        let mut tyre = shared.tyre_change_mut();
        tyre.texture = textures.first_textures()[599];
        tyre.device_texture = textures.kinds.borrow().kind_first_slots[599];
        tyre.title_texture = textures.first_textures()[TYRE_TITLE_TEXTURE as usize];
        tyre.title_device_texture =
            textures.kinds.borrow().kind_first_slots[TYRE_TITLE_TEXTURE as usize];
    }
    let (present, state, stopped) = {
        let blocks = race.car_blocks();
        let car = &blocks.cars[0];
        (car.parts_present, car.tyre_change_state, car.stopped_steps)
    };
    if present & 0x1E != 0x1E && state == 0 && stopped >= 0x12C {
        tyre_change_gauge(o, true, 0)?;
        race.car_blocks_mut().cars[0].tyre_change_state = 1;
    }
    if race.car_blocks().cars[0].tyre_change_state != 1 {
        return Ok(());
    }
    let time = tyre_change_gauge(o, false, 0)?;
    if time == 0 {
        return Ok(());
    }
    let mut blocks = race.car_blocks_mut();
    let car = &mut blocks.cars[0];
    car.stage_time = car.stage_time.wrapping_add(time);
    car.tyre_change_state = 0xFFFF_FED3;
    for bit in 1..=4 {
        let bits = car.parts_present;
        car.parts_present = bits | 1 << bit;
        if bits & 1 << bit == 0 {
            car.tyre_wears[bit - 1] = 0.0;
        }
    }
    Ok(())
}
