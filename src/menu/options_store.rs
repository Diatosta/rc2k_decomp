//! The options the player chooses on the options pages: the visual and sound settings and the
//! volumes. The front end owns them (`Menus::options`); the race, the network, the sound and the
//! options file read them through `MenuOptions::snapshot`. The key-entry state of the pages is
//! `menu::entry_store`.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The options as the pages edit and the options file keeps them.
#[derive(Clone, Copy)]
pub struct OptionValues {
    /// Mipmapping, 0 off, 1 on (a copy of `ScreenChoices.mipmapping`; `OptionSwitch::Mipmapping`).
    pub mipmapping: u32,
    /// Buffering, 0 double, 1 triple (a copy of `ScreenChoices.buffering`).
    pub buffering: u32,
    /// Set when the screen mode was changed (`menu::option_steps`).
    pub mode_changed: u32,
    /// The screen modes' switch (`OptionSwitch::ScreenModeCard`).
    pub modes_switch: u32,
    /// The lens flare option (the race's `lens_flare`).
    pub lens_flare: u32,
    /// A 0 ..= 100 level, stepped by dead code only; kept in the options file.
    pub unused_percent_a: u32,
    /// The same, the second level.
    pub unused_percent_b: u32,
    /// The reflections option.
    pub reflections: u32,
    /// The car shadows setting (row 6 of the visual options, strings 0x396 "reduced" / 0x397
    /// "full"); copied to the race's `settings.car_shadows` and `settings_words[2]`.
    pub car_shadows: u32,
    /// The car detail setting, 0 .. 2 (row 7, strings 0x393 "no cars" / "your car" /
    /// "all cars": which cars get the detailed textures; stepped by
    /// `menu::next_texture_setting`); copied to the race's `settings.car_detail` and
    /// `settings_words[3]`.
    pub car_detail: u32,
    /// The headlights option.
    pub headlights: u32,
    /// The audio quality chosen (`menu::next_audio_quality`).
    pub audio_quality: u32,
    /// The DirectSound 3D switch.
    pub ds3d: u32,
    /// The EAX switch.
    pub eax: u32,
    /// The switch of Aureal's A3D as the options files hold it. The game has no A3D: no page
    /// shows this and nothing reads it; it is kept so that a file is written back as it was
    /// read (a file loaded at the start has it 0: `Options::without_a3d`).
    pub retired_a3d: u32,
    /// The Dolby switch.
    pub dolby: u32,
    /// The channels chosen.
    pub channels: u32,
    /// The music level (`menu::option_steps`).
    pub menu_music_volume: u32,
    /// The race music's level, read by the music "up" step (`menu::music_volume_up`).
    pub race_music_level: u32,
    /// The menu sounds' third volume (`sound::play_menu_sound`).
    pub menu_effects_volume: u32,
    /// The speech level (`menu::option_steps`, `sound::play_car_stream`).
    pub menu_voices_volume: u32,
}

/// The options other subsystems read: the race's visual settings and the volumes the sound
/// code starts streams and menu sounds with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionsSnapshot {
    pub lens_flare: u32,
    pub reflections: u32,
    pub car_shadows: u32,
    pub car_detail: u32,
    pub headlights: u32,
    pub menu_music_volume: u32,
    pub race_music_level: u32,
    pub menu_effects_volume: u32,
    pub menu_voices_volume: u32,
}

impl OptionsSnapshot {
    /// The visual options, as the race takes them.
    pub fn visual(&self) -> crate::race::state_store::VisualOptions {
        crate::race::state_store::VisualOptions {
            lens_flare: self.lens_flare,
            reflections: self.reflections,
            car_shadows: self.car_shadows,
            car_detail: self.car_detail,
            headlights: self.headlights,
        }
    }
}

/// The options pages' values behind one `RefCell`: the game's thread alone reads and writes
/// them, each borrow lasting one statement.
pub struct MenuOptions(GameThread<RefCell<OptionValues>>);

impl MenuOptions {
    pub const fn new() -> Self {
        MenuOptions(GameThread::new(RefCell::new(OptionValues {
            mipmapping: 0x0,
            buffering: 0x0,
            mode_changed: 0x0,
            modes_switch: 0x1,
            lens_flare: 0x1,
            unused_percent_a: 0x64,
            unused_percent_b: 0x64,
            reflections: 0x1,
            car_shadows: 0x1,
            car_detail: 0x1,
            headlights: 0x0,
            audio_quality: 0x3,
            ds3d: 0x0,
            eax: 0x0,
            retired_a3d: 0x0,
            dolby: 0x0,
            channels: 0x1,
            menu_music_volume: 0x40,
            race_music_level: 0x40,
            menu_effects_volume: 0x40,
            menu_voices_volume: 0x40,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> OptionValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, OptionValues> {
        self.0.get().borrow_mut()
    }

    /// The values the race, the network and the sound read.
    pub fn snapshot(&self) -> OptionsSnapshot {
        let v = self.read();
        OptionsSnapshot {
            lens_flare: v.lens_flare,
            reflections: v.reflections,
            car_shadows: v.car_shadows,
            car_detail: v.car_detail,
            headlights: v.headlights,
            menu_music_volume: v.menu_music_volume,
            race_music_level: v.race_music_level,
            menu_effects_volume: v.menu_effects_volume,
            menu_voices_volume: v.menu_voices_volume,
        }
    }

    /// Stores the five values of the sound page: the audio quality, DS3D, EAX, Dolby and the
    /// channels (`screens::load_audio_settings`).
    pub fn set_audio_page(&self, values: [u32; 5]) {
        let mut v = self.write();
        v.audio_quality = values[0];
        v.ds3d = values[1];
        v.eax = values[2];
        v.dolby = values[3];
        v.channels = values[4];
    }

    /// The first run of the options file: mipmapping .. headlights, eleven little-endian dwords.
    pub fn visual_run_bytes(&self) -> Vec<u8> {
        let v = self.read();
        le_bytes(&[
            v.mipmapping,
            v.buffering,
            v.mode_changed,
            v.modes_switch,
            v.lens_flare,
            v.unused_percent_a,
            v.unused_percent_b,
            v.reflections,
            v.car_shadows,
            v.car_detail,
            v.headlights,
        ])
    }

    /// Sets the first run from the file's bytes.
    pub fn set_visual_run(&self, bytes: &[u8]) {
        let [a, b, c, d, e, f, g, h, i, j, k] = le_words(bytes);
        let mut v = self.write();
        v.mipmapping = a;
        v.buffering = b;
        v.mode_changed = c;
        v.modes_switch = d;
        v.lens_flare = e;
        v.unused_percent_a = f;
        v.unused_percent_b = g;
        v.reflections = h;
        v.car_shadows = i;
        v.car_detail = j;
        v.headlights = k;
    }

    /// The second run: the audio quality, the 3D switches and the channels, six dwords.
    pub fn audio_run_bytes(&self) -> Vec<u8> {
        let v = self.read();
        le_bytes(&[
            v.audio_quality,
            v.ds3d,
            v.eax,
            v.retired_a3d,
            v.dolby,
            v.channels,
        ])
    }

    /// Sets the second run from the file's bytes.
    pub fn set_audio_run(&self, bytes: &[u8]) {
        let [a, b, c, d, e, f] = le_words(bytes);
        let mut v = self.write();
        v.audio_quality = a;
        v.ds3d = b;
        v.eax = c;
        v.retired_a3d = d;
        v.dolby = e;
        v.channels = f;
    }
}

fn le_bytes(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

/// The first `N` little-endian dwords of `bytes` (the file's pieces have their length).
fn le_words<const N: usize>(bytes: &[u8]) -> [u32; N] {
    std::array::from_fn(|n| {
        let at = n * 4;
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The snapshot holds each option's value and follows a change; the defaults are the
    /// options a fresh install starts with.
    #[test]
    fn snapshot_follows_the_cells() {
        let options = MenuOptions::new();
        let fresh = options.snapshot();
        assert_eq!(
            fresh,
            OptionsSnapshot {
                lens_flare: 1,
                reflections: 1,
                car_shadows: 1,
                car_detail: 1,
                headlights: 0,
                menu_music_volume: 0x40,
                race_music_level: 0x40,
                menu_effects_volume: 0x40,
                menu_voices_volume: 0x40,
            }
        );
        options.write().car_detail = 2;
        options.write().menu_voices_volume = 7;
        let changed = options.snapshot();
        assert_eq!((changed.car_detail, changed.menu_voices_volume), (2, 7));
        assert_eq!(
            OptionsSnapshot {
                car_detail: 1,
                menu_voices_volume: 0x40,
                ..changed
            },
            fresh
        );
    }
}
