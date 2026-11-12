//! The race's frame outside the physics (`drive::race_loop`'s callees): the
//! players' states, the cars' counters, the view cars loaded into the work copy. Here
//! its leaves.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::race::owner::Race;

/// `race::players_all_out` (0x445506): carry when every player is out (+0xB0 not 0): the chosen
/// car's with `session` 1, else each of the `players` players'. Refuses, before anything,
/// a chosen car past 7 or more than 8 players (the original reads past the table).
pub fn players_all_out(race: &Race, net: &crate::net::owner::Net) -> Result<bool, Refused> {
    let vars = &net.vars;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::cars::players_all_out`.
    use rc2k_core::race::cars::{Players, players_all_out};
    {
        let trap = crate::race::trap::Trap::new();
        let out = |n: u32| {
            trap.keep_read(shared.race_car(n as usize).map(|car| car.stop_state.get())) != 0
        };
        let players = if vars.session.get() == 1 {
            Players::Chosen(vars.player.get())
        } else {
            Players::First(shared.view.get().players.get())
        };
        let all = players_all_out(players, out)?;
        trap.finish()?;
        Ok(all)
    }
}

/// `race::set_car_counter` (0x43A96C): a car block's message, written in the original's order
/// (+0x71C, +0x730, +0x72C).
pub fn set_car_counter(car: &Car, value: u32) {
    // Converted: `rc2k_core::race::cars::Message::new`.
    let m = rc2k_core::race::cars::Message::new(value);
    car.notice_id.set(m.id);
    car.notice_time_b.set(m.age);
    car.notice_time.set(m.time_left);
}

/// `race::set_both_counters` (0x43A987): `set_car_counter` of the first two blocks (by their fixed
/// addresses), each with the same value.
pub fn set_both_counters(race: &Race, value: u32) {
    // Converted: through `set_car_counter` (`rc2k_core::race::cars::Message`).
    for car in &race.car_blocks().cars[..2] {
        set_car_counter(car, value);
    }
}

/// The view car `n`'s number: `view_cars`, `RaceView.view_cars[1]`, `local_car`.
pub fn view_car(view: &crate::race::view_store::RaceView, n: usize) -> &crate::session::PlainDword {
    match n {
        0 => &view.view_cars[0],
        1 => &view.view_cars[1],
        2 => &view.local_car,
        _ => panic!("a view car past the third"),
    }
}

/// `race::load_car_a` / `_b` / `_c` (0x45422F, 0x454241, 0x454277): the view car `n`'s
/// block (of `blocks`: the game thread's, or a drawing part's reads) copied into `view`'s car
/// (`ViewScratch::car`). Refuses a null block (the original's copy faults) before anything.
pub fn load_view_car(
    work: &crate::race::work_store::RaceWork,
    blocks: &crate::race::blocks_store::CarBlocks,
    race_view: &crate::race::view_store::RaceView,
    n: usize,
    view: &crate::race::view_scratch::ViewScratch,
) -> Result<(), Refused> {
    use rc2k_core::drive::work_car::{car_block, car_slot};
    let slot = car_slot(view_car(race_view, n).get());
    car_block(u32::from(work.has_car_block(slot as usize)))?;
    crate::cells::copy_whole(
        crate::race::owner_shared::RaceShared::race_car_in(work, blocks, slot as usize)?,
        &view.car,
    );
    Ok(())
}

// The race keys, by PC scan code (set 1) in `g_keysPressed`: F1 .. F5 (0x3B ..), F6 .. F12 for the
// view block's option bits; `g_keysNow`'s 0x1D (`Devices.keys_now`, the Left Ctrl scan code) held makes
// them act on the second view's block (`second` below).
const VIEW_MODE_KEY: usize = 0x1D;
const NEXT_VIEW_KEY: usize = 0x3B; // Devices.keys_pressed
const SPLIT_KEY: usize = 0x3C; // Devices.keys_pressed
const DOWN_KEY: usize = 0x3D; // Devices.keys_pressed
const UP_KEY: usize = 0x3E; // Devices.keys_pressed
const SHARED_KEY: usize = 0x3F; // Devices.keys_pressed
/// The keys that flip a bit of the view block's +0x220 (`rc2k_core::race::keys::BIT_KEYS`
/// by place): their key bytes, in the order tested.
pub const BIT_KEYS: [usize; 7] = [0x58, 0x42, 0x41, 0x44, 0x43, 0x40, 0x57]; // 4F6D34, 4F6D1E, 4F6D1D, 4F6D20, 4F6D1F, 4F6D1C, 4F6D33
fn pressed(devices: &crate::input::store::Devices, key: usize) -> bool {
    devices.keys_pressed.get(key) != 0
}

/// `race::race_keys` (0x43A649): the race's view and sound keys (see its header).
pub fn race_keys(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (audio, sound) = (o.audio, o.sound);
    let devices = &o.input.devices;
    let state = &audio.state;
    let shared = &race.shared;
    crate::drive::choose_shown_cars::choose_shown_cars_in(race, o.net)?;
    let second = devices.keys_now.get(VIEW_MODE_KEY) != 0;
    let view = shared.view_car_or_end(usize::from(second));
    // Converted: `rc2k_core::race::keys::{first_key, split_flip, volume_key,
    // volume_step, flip_bit}`.
    use rc2k_core::race::keys::{
        BIT_KEYS as BITS, COUNTED_VIEWS, Key, SHARED_BIT, first_key, flip_bit, split_flip,
        volume_key, volume_step,
    };
    let key = first_key(|k| {
        pressed(
            devices,
            match k {
                Key::NextView => NEXT_VIEW_KEY,
                Key::Split => SPLIT_KEY,
                Key::VolumeDown => DOWN_KEY,
                Key::VolumeUp => UP_KEY,
                Key::Bit(n) => BIT_KEYS[n],
                Key::Shared => SHARED_KEY,
            },
        )
    });
    // The view block's +0x220 with `bit` flipped; gives the counter.
    let flip = |view: &Car, bit: (u32, u32, u32)| -> u32 {
        let (now, counter) = flip_bit(view.option_bits.get(), bit);
        view.option_bits.set(now);
        counter
    };
    match key {
        Some(Key::NextView) => {
            crate::drive::next_camera_view::next_camera_view(race, view)?;
            let camera = view.camera_blend_view.get();
            if camera < COUNTED_VIEWS {
                set_car_counter(
                    view,
                    shared.physics_tables.get().view_counters[camera as usize].get(),
                );
            }
        }
        Some(Key::Split) => {
            if shared.view.get().players.get() == 1 {
                return Ok(());
            }
            let (split, counter) = split_flip(shared.vars.get().split.get());
            shared.vars.get().split.set(split);
            set_both_counters(race, counter);
        }
        Some(k @ (Key::VolumeDown | Key::VolumeUp)) => {
            let up = k == Key::VolumeUp;
            let (step, counter) = volume_key(up, second);
            set_both_counters(race, counter);
            if second {
                // The "up" step reads `OptionWork.other_volume`, not the music's volume (see bugs).
                let from: u32 = if up {
                    o.menus.screens.option_work.other_volume.get()
                } else {
                    sound.menu.options.race_music_volume.get()
                };
                let volume = volume_step(from, step);
                sound.menu.options.race_music_volume.set(volume);
                crate::sound::set_music_volume::set_music_volume(volume, audio, o.system);
            } else {
                let volume = volume_step(state.master_volume.get(), step);
                state.master_volume.set(volume);
                // The sound command 8 with the volume.
                crate::sound::send_command_8::send_command_8(audio, volume);
            }
        }
        Some(Key::Bit(n)) => {
            let counter = flip(view, BITS[n]);
            set_car_counter(view, counter);
        }
        Some(Key::Shared) => {
            let counter = flip(shared.view_car_or_end(0), SHARED_BIT);
            set_both_counters(race, counter);
        }
        None => {}
    }
    Ok(())
}

/// `race::tyre_change` (0x444F7A): the textures copied (`TyreChange::texture` / `device_texture`
/// from `KindTextureTables.kind_first_textures[599]` / `KindTextureTables.kind_first_slots[599]`); for car 0 (block at 0x70F3EC: +0x268 "c1 tyrec" count
/// `CarsAndReplay.car_blocks.cars[0].stopped_steps`, +0x26C the change's state `CarsAndReplay.car_blocks.cars[0].tyre_change_state`, +0x298 the tyre bits `CarsAndReplay.car_blocks.cars[0].parts_present`,
/// +0x88 the time `CarsAndReplay.car_blocks.cars[0].stage_time`, +0x354 .. +0x360 `CarsAndReplay.car_blocks.cars[0].tyre_wears[0]` .. `CarsAndReplay.car_blocks.cars[0].tyre_wears[3]`, one per bit 1
/// .. 4) `race::tyre_change_gauge` started, then run, its time when done added.
pub fn tyre_change(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (race, scene) = (o.race, o.scene);
    let shared = &race.shared;
    let textures = &scene.textures;
    use crate::race::tyre_change_gauge::tyre_change_gauge;
    let car = &race.car_blocks().cars[0];
    shared
        .tyre_change()
        .texture
        .set(textures.kinds.get().kind_first_textures[599].get());
    shared
        .tyre_change()
        .device_texture
        .set(textures.kinds.get().kind_first_slots[599].get());
    if car.parts_present.get() & 0x1E != 0x1E
        && car.tyre_change_state.get() == 0
        && car.stopped_steps.get() >= 0x12C
    {
        tyre_change_gauge(o, true, 0)?;
        car.tyre_change_state.set(1);
    }
    if car.tyre_change_state.get() != 1 {
        return Ok(());
    }
    let time = tyre_change_gauge(o, false, 0)?;
    if time == 0 {
        return Ok(());
    }
    car.stage_time.set(car.stage_time.get().wrapping_add(time));
    car.tyre_change_state.set(0xFFFF_FED3);
    for bit in 1..=4 {
        let bits = car.parts_present.get();
        car.parts_present.set(bits | 1 << bit);
        if bits & 1 << bit == 0 {
            car.tyre_wears[bit - 1].set(0.0);
        }
    }
    Ok(())
}
