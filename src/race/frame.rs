//! The race's frame outside the physics (`sub_444D46`'s callees, IDA 2026-10-17): the
//! players' states, the cars' counters, the view cars loaded into the work copy. Here
//! its leaves.

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::input::store::DEVICES;
use crate::menu::owner::menus;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PHYSICS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;

fn block(n: u32) -> u32 {
    crate::race::work_store::RACE_WORK.car_block(n as usize)
}

/// `race::players_all_out` (0x445506): carry when every player is out (+0xB0 not 0): the
/// chosen car's with `dword_52A910` 1, else each of the `dword_63DB94` players' (none: no
/// carry). Every register kept. Refuses, before anything, a chosen car past 7 or more
/// than 8 players (the original reads past the table).
pub fn players_all_out() -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::cars::players_all_out`.
    use rc2k_core::race::cars::{Players, players_all_out};
    {
        let trap = crate::race::trap::Trap::new();
        let out = |n: u32| trap.read::<u32>(block(n) + 0xB0, "a car block's out flag") != 0;
        let players = if crate::net::store::NET.session.get() == 1 {
            Players::Chosen(crate::net::store::NET.player.get())
        } else {
            Players::First(RACE_VIEW.players.bits())
        };
        let all = players_all_out(players, out)?;
        trap.finish()?;
        Ok(all)
    }
}

/// `race::set_car_counter` (0x43A96C): block esi's message (+0x71C eax, +0x72C 0x12C,
/// +0x730 0), written in the original's order (+0x71C, +0x730, +0x72C).
pub fn set_car_counter(car: &Car, value: u32) {
    // Converted: `rc2k_core::race::cars::Message::new`.
    let m = rc2k_core::race::cars::Message::new(value);
    car.notice_id.set(m.id);
    car.notice_time_b.set(m.age);
    car.notice_time.set(m.time_left);
}

/// `race::set_both_counters` (0x43A987): `set_car_counter` of the first two blocks (by
/// their fixed addresses), each with eax.
pub fn set_both_counters(value: u32) {
    // Converted: through `set_car_counter` (`rc2k_core::race::cars::Message`).
    for car in &crate::race::blocks_store::CAR_BLOCKS.cars[..2] {
        set_car_counter(car, value);
    }
}

/// The view car `n`'s number: `dword_63DBA8`, `dword_63DBAC`, `dword_63DBA0`.
pub fn view_car(n: usize) -> &'static crate::session::Dword {
    match n {
        0 => &RACE_VIEW.view_cars[0],
        1 => &RACE_VIEW.view_cars[1],
        2 => &RACE_VIEW.local_car,
        _ => panic!("a view car past the third"),
    }
}

/// `race::load_car_a` / `_b` / `_c` (0x45422F, 0x454241, 0x454277): `g_currentCar` the
/// view car's number, `drive::load_car`. (Its callers, `race::draw_race_scene` and
/// `race::scroll_maps`, read no register it leaves.)
pub fn load_view_car(n: usize) -> Result<(), Refused> {
    with_view_car(n, crate::drive::load_car::load_car)
}

/// `g_currentCar` the view car's number, then `then`; on a refusal `g_currentCar` is put
/// back, so a refusing call has written nothing.
fn with_view_car(n: usize, then: impl FnOnce() -> Result<(), Refused>) -> Result<(), Refused> {
    let current = &crate::race::work_store::RACE_WORK.current_car;
    let was = current.get();
    current.set(view_car(n).get());
    let result = then();
    if result.is_err() {
        current.set(was);
    }
    result
}

/// `race::store_car_a` / `_b` / `_c` (0x454253, 0x454265, 0x454289): `g_currentCar` the
/// view car's number, `drive::store_car`. (Its one caller, `race::scroll_maps`, reads no
/// register it leaves.)
pub fn store_view_car(n: usize) -> Result<(), Refused> {
    with_view_car(n, crate::drive::store_car::store_car)
}

// The race keys, by scan code in `g_keysPressed`; the second view's keys is `g_keysNow`'s 0x1D
// (`byte_4F6B79`, a reading).
const VIEW_MODE_KEY: usize = 0x1D;
const NEXT_VIEW_KEY: usize = 0x3B; // byte_4F6D17
const SPLIT_KEY: usize = 0x3C; // byte_4F6D18
const DOWN_KEY: usize = 0x3D; // byte_4F6D19
const UP_KEY: usize = 0x3E; // byte_4F6D1A
const SHARED_KEY: usize = 0x3F; // byte_4F6D1B
/// The keys that flip a bit of the view block's +0x220 (`rc2k_core::race::keys::BIT_KEYS`
/// by place): their key bytes, in the order tested.
pub const BIT_KEYS: [usize; 7] = [0x58, 0x42, 0x41, 0x44, 0x43, 0x40, 0x57]; // 4F6D34, 4F6D1E, 4F6D1D, 4F6D20, 4F6D1F, 4F6D1C, 4F6D33
fn pressed(key: usize) -> bool {
    DEVICES.keys_pressed.get(key) != 0
}

/// `race::race_keys` (0x43A649): the race's view and sound keys (see its header). It
/// reads no register at entry; the ones it leaves are read by none of its callers
/// (`drive::race_loop`, `drive::pause_until_key`).
pub fn race_keys() -> Result<(), Refused> {
    crate::drive::choose_shown_cars::choose_shown_cars()?;
    let second = DEVICES.keys_now.get(VIEW_MODE_KEY) != 0;
    let view_addr = RACE_VIEW.view_block(usize::from(second));
    let view = Car::at(view_addr);
    // Converted: `rc2k_core::race::keys::{first_key, split_flip, volume_key,
    // volume_step, flip_bit}`.
    use rc2k_core::race::keys::{
        BIT_KEYS as BITS, COUNTED_VIEWS, Key, SHARED_BIT, first_key, flip_bit, split_flip,
        volume_key, volume_step,
    };
    let key = first_key(|k| {
        pressed(match k {
            Key::NextView => NEXT_VIEW_KEY,
            Key::Split => SPLIT_KEY,
            Key::VolumeDown => DOWN_KEY,
            Key::VolumeUp => UP_KEY,
            Key::Bit(n) => BIT_KEYS[n],
            Key::Shared => SHARED_KEY,
        })
    });
    // The view block's +0x220 with `bit` flipped; gives the counter.
    let flip = |view: &Car, bit: (u32, u32, u32)| -> u32 {
        let (now, counter) = flip_bit(view.option_bits.get(), bit);
        view.option_bits.set(now);
        counter
    };
    match key {
        Some(Key::NextView) => {
            crate::drive::next_camera_view::next_camera_view(view_addr)?;
            let camera = view.camera_blend_view.get();
            if camera < COUNTED_VIEWS {
                set_car_counter(view, PHYSICS.view_counters[camera as usize].get());
            }
        }
        Some(Key::Split) => {
            if RACE_VIEW.players.bits() == 1 {
                return Ok(());
            }
            let (split, counter) = split_flip(RACE_VARS.split.bits());
            RACE_VARS.split.set_bits(split);
            set_both_counters(counter);
        }
        Some(k @ (Key::VolumeDown | Key::VolumeUp)) => {
            let up = k == Key::VolumeUp;
            let (step, counter) = volume_key(up, second);
            set_both_counters(counter);
            if second {
                // The "up" step reads `dword_75568C`, not the music's volume (see bugs).
                let from: u32 = if up {
                    menus().screens.option_work.other_volume.get()
                } else {
                    crate::sound::owner::sound()
                        .menu
                        .options
                        .race_music_volume
                        .get()
                };
                let volume = volume_step(from, step);
                crate::sound::owner::sound()
                    .menu
                    .options
                    .race_music_volume
                    .set(volume);
                crate::sound::set_music_volume::set_music_volume(volume);
            } else {
                let volume = volume_step(AUDIO.master_volume.get(), step);
                AUDIO.master_volume.set(volume);
                // `pusha`, `push 8`, `send_command`, `pop eax`, `popa`.
                crate::sound::send_command_8::send_command_8(volume);
            }
        }
        Some(Key::Bit(n)) => {
            let counter = flip(view, BITS[n]);
            set_car_counter(view, counter);
        }
        Some(Key::Shared) => {
            let counter = flip(Car::at(RACE_VIEW.view_block(0)), SHARED_BIT);
            set_both_counters(counter);
        }
        None => {}
    }
    Ok(())
}

/// `race::tyre_change` (0x444F7A): the textures copied (`TyreChange::texture` /
/// `device_texture` from `dword_5D1444` / `dword_5D2444`); for car 0 (block at 0x70F3EC:
/// +0x268 "c1 tyrec" count `dword_70F654`, +0x26C the change's state `dword_70F658`,
/// +0x298 the tyre bits `dword_70F684`, +0x88 the time `dword_70F474`, +0x354 .. +0x360
/// `dword_70F740` .. `dword_70F74C`, one per bit 1 .. 4) `race::tyre_change_gauge`
/// started, then run, its time when done added. It reads no register at entry; the eax and
/// esi it leaves are read by no caller (`drive::race_loop`).
pub fn tyre_change() -> Result<(), Refused> {
    use crate::race::blocks_store::CAR_BLOCKS;
    use crate::race::tyre_change_gauge::tyre_change_gauge;
    use crate::race::tyre_store::TYRE_CHANGE as TYRE;
    let car = &CAR_BLOCKS.cars[0];
    TYRE.texture.set(
        crate::scene::owner::scene()
            .textures
            .kinds
            .kind_first_textures[599]
            .get(),
    );
    TYRE.device_texture
        .set(crate::scene::owner::scene().textures.kinds.kind_first_slots[599].get());
    if car.parts_present.get() & 0x1E != 0x1E
        && car.tyre_change_state.get() == 0
        && car.stopped_steps.get() >= 0x12C
    {
        tyre_change_gauge(true, 0)?;
        car.tyre_change_state.set(1);
    }
    if car.tyre_change_state.get() != 1 {
        return Ok(());
    }
    let time = tyre_change_gauge(false, 0)?;
    if time == 0 {
        return Ok(());
    }
    car.stage_time.set(car.stage_time.get().wrapping_add(time));
    car.tyre_change_state.set(0xFFFF_FED3);
    for bit in 1..=4 {
        let bits = car.parts_present.get();
        car.parts_present.set(bits | 1 << bit);
        if bits & 1 << bit == 0 {
            car.part_hit_damage[bit - 1].set_bits(0);
        }
    }
    Ok(())
}
