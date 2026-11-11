//! `sound::play_menu_sound` (0x40F059): plays one of the menus' sounds.
//!
//! Nothing for no sound bits (0), and nothing before the time kept in `dword_511B0C` (125 ms after
//! the last sound; compared by the sign of the difference). The time is moved on before
//! the bits are looked at. The first bit set of these, in this order, picks the sound
//! and the volume it is played at:
//! 0x80 sound 0xE4 at `dword_755694`; 0x10, 0x20, 0x40 sounds 0x163, 0x197, 0x1CB at
//! `ScreenFlow.menu_sounds_volume`; 0x100, 8, 2, 4, 1 sounds 0x1CC, 0x1CA, 0x1C8, 0x1C9, 0x1C7 at
//! `OptionWork.menu_sounds_c`. Then `sound::play_sound` with the sample's own rate (0xFFFFFF91) and no place.
//! 25 callers (1 is the highlight's click, 2 Enter's sound: `menu::handle_keys`).

use crate::session::Dword;
use crate::session::DwordLike;
use crate::session::PlainDword;
use crate::system::tick_count::tick_count;

// ---- game data ----

const VOLUME_A: &Dword = &crate::audio::owner::AUDIO_OWNER.levels.menu_sounds_a; // dword_755694
const VOLUME_B: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .flow
    .menu_sounds_volume; // ScreenFlow.menu_sounds_volume
const VOLUME_C: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .option_work
    .menu_sounds_c; // OptionWork.menu_sounds_c

/// `sound` the menu's sound (0: none); `now` is asked for the clock.
pub fn play_menu_sound(
    sound: u32,
    now: &mut dyn FnMut() -> u32,
    owner: &crate::sound::owner::Sound,
) {
    // Converted: `rc2k_core::sound::senders::menu_sound`.
    use rc2k_core::sound::senders::{MenuVolume, menu_sound};
    if sound == 0 {
        return;
    }
    let Some((next, pick)) = menu_sound(sound, now(), owner.menu.next_sound_time.get()) else {
        return;
    };
    owner.menu.next_sound_time.set(next);
    let Some((sound, volume)) = pick else {
        return;
    };
    let volume: &dyn DwordLike = match volume {
        MenuVolume::A => VOLUME_A,
        MenuVolume::B => VOLUME_B,
        MenuVolume::C => VOLUME_C,
    };
    crate::sound::play_sound::play_sound(sound, volume.get(), 0xFFFF_FF91, [0; 3]);
}

/// With the game's clock: what other Rust code calls.
pub fn play_menu_sound_in_game(sound: u32) {
    play_menu_sound(sound, &mut tick_count, crate::sound::owner::sound());
}
