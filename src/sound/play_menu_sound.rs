//! `sound::play_menu_sound`: plays one of the menus' sounds.
//!
//! Nothing for no sound bits (0), and nothing before the time kept (125 ms after the last sound;
//! compared by the sign of the difference). The time is moved on before the bits are looked at. The
//! first bit set of these, in this order, picks the sound and the volume it is played at: 0x80
//! sound 0xE4 at `levels.menu_sounds_a`; 0x10, 0x20, 0x40 sounds 0x163, 0x197, 0x1CB at
//! `ScreenFlow.menu_sounds_volume`; 0x100, 8, 2, 4, 1 sounds 0x1CC, 0x1CA, 0x1C8, 0x1C9, 0x1C7 at
//! `MenuOptions.menu_sounds_level`. Then `sound::play_sound` with the sample's own rate and no place. 25
//! callers (1 is the highlight's click, 2 Enter's sound: `menu::handle_keys`).

use crate::audio::owner::Audio;
use crate::menu::owner::Menus;
use crate::sound::owner::Sound;
use crate::system::tick_count::tick_count;

/// `sound` the menu's sound (0: none); `now` is asked for the clock.
pub fn play_menu_sound(
    sound: u32,
    now: &mut dyn FnMut() -> u32,
    sounds: &Sound,
    audio: &Audio,
    menus: &Menus,
) {
    // The logic is `rc2k_core::sound::senders::menu_sound`.
    use rc2k_core::sound::senders::{MenuVolume, menu_sound};
    if sound == 0 {
        return;
    }
    let Some((next, pick)) = menu_sound(sound, now(), sounds.menu.next_sound_time.get().get())
    else {
        return;
    };
    sounds.menu.next_sound_time.get().set(next);
    let Some((sound, volume)) = pick else {
        return;
    };
    let volume: u32 = match volume {
        MenuVolume::A => audio.levels.menu_sounds_a.get(),
        MenuVolume::B => menus.screens.flow.menu_sounds_volume.get(),
        MenuVolume::C => menus.options.snapshot().menu_sounds_level,
    };
    crate::sound::play_sound::play_sound_in(audio, sound, volume, 0xFFFF_FF91, [0; 3]);
}

/// With the game's clock: what other Rust code calls.
pub fn play_menu_sound_in_game(
    sounds: &Sound,
    audio: &Audio,
    menus: &Menus,
    system: &crate::system::owner::System,
    sound: u32,
) {
    play_menu_sound(
        sound,
        &mut || tick_count(&system.session.picture_clock),
        sounds,
        audio,
        menus,
    );
}
