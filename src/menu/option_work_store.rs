//! The options screens' work copies, out of RAL.EXE's image (IDA 0x755640 .. 0x7556B4):
//! the switches and settings the options pages change, the sound options and volumes, the menu sound
//! and the key being redefined. The first 0x2C and the next 0x18 bytes are the options file's
//! (`saves::load_options`). A `#[repr(C)]` struct with the image's layout, one static;
//!

use crate::session::{PlainBytes, PlainDword, PlainFlag};

/// The options' work copies (IDA 0x755640 .. 0x7556B4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct OptionWork {
    /// `dword_755640`: A visual page switch (a copy of `ScreenChoices.mipmapping`; `OptionSwitch::Mipmapping`).
    pub switch_0: PlainDword,
    /// `dword_755644`: A visual page switch (a copy of `ScreenChoices.buffering`).
    pub switch_1: PlainDword,
    /// `dword_755648`: Set when the screen mode was changed (`menu::option_steps`).
    pub mode_changed: PlainDword,
    /// `dword_75564C`: The screen modes' switch (`OptionSwitch::ScreenModeCard`).
    pub modes_switch: PlainDword,
    /// `dword_755650`: The graphics page's lens flare option, work copy (the race's `lens_flare`).
    pub lens_flare: PlainDword,
    /// `dword_755654`: a 0 ..= 100 level, stepped by dead code only; copied to `race::state_store`.
    pub unused_percent_a: PlainDword,
    /// `dword_755658`: the same, the second level.
    pub unused_percent_b: PlainDword,
    /// `dword_75565C`: The graphics page's reflections option, work copy.
    pub reflections: PlainDword,
    /// +0x10 (0x755660) `dword_755660`: the options page's car shadows setting (row 6 of the visual options, strings 0x396 "reduzido" / 0x397 "completo"); copied to the race's `settings.car_shadows` (0x75AD10) and `settings_words[2]` (0x6238B0). Probable: the debug tab reads the race copy as "Car shadows".
    pub car_shadows: PlainDword,
    /// +0x14 (0x755664) `dword_755664`: the options page's car detail setting, 0 .. 2 (row 7, strings 0x393 "nenhum carro" / "o seu carro" / "todos os carros": which cars get the detailed textures; stepped by `menu::next_texture_setting`); copied to the race's `settings.car_detail` (0x75AD14) and `settings_words[3]` (0x6238B4). Probable, from the string and the page's row order.
    pub car_detail: PlainDword,
    /// `dword_755668`: The graphics page's headlights option, work copy.
    pub headlights: PlainDword,
    /// `dword_75566C`: The audio quality chosen (`menu::next_audio_quality`).
    pub audio_quality: PlainDword,
    /// `dword_755670`: The DirectSound 3D switch.
    pub ds3d: PlainDword,
    /// `dword_755674`: The EAX switch.
    pub eax: PlainDword,
    /// `dword_755678`: The A3D switch.
    pub a3d: PlainDword,
    /// `dword_75567C`: The Dolby switch.
    pub dolby: PlainDword,
    /// `dword_755680`: The channels chosen.
    pub channels: PlainDword,
    /// 4 bytes of the image's layout: the master volume shown (now `audio::levels::Levels`).
    pub _gap_755684: PlainBytes<4>,
    /// `dword_755688`: The music level (`menu::option_steps`).
    pub music_level: PlainDword,
    /// `dword_75568C`: Read by the music "up" step (`menu::music_volume_up`).
    pub other_volume: PlainDword,
    /// `dword_755690`: The menu sounds' third volume (`sound::play_menu_sound`).
    pub menu_sounds_c: PlainDword,
    /// 4 bytes of the image's layout: the race sounds' volume (now `audio::levels::Levels`).
    pub _gap_755694: PlainBytes<4>,
    /// `dword_755698`: The speech level (`menu::option_steps`, `sound::play_car_stream`).
    pub speech_level: PlainDword,
    /// 4 bytes of the image's layout: the sounds' first level (now `audio::levels::Levels`).
    pub _gap_75569c: PlainBytes<4>,
    /// 4 bytes of the image's layout: the sounds' second level (now `audio::levels::Levels`).
    pub _gap_7556a0: PlainBytes<4>,
    /// `dword_7556A4`: The menu's sound: 0x6F, or 0x14D after Enter (`menu::handle_keys`).
    pub menu_sound: PlainDword,
    /// `dword_7556A8`: 1 while a text is being typed (`menu::text_entry`).
    pub typing: PlainFlag,
    /// `g_redefiningKey`: 1 while a key is being redefined; also 1 = the highlight does not blink (`menu::update_blink`).
    pub redefining_key: PlainFlag,
    /// `g_redefinedKeyPlace`: The place of the key being redefined (999 = none).
    pub redefined_key_place: PlainDword,
}

impl OptionWork {
    /// What the last key looked at was.
    pub fn sound(&self) -> rc2k_core::menu::keys::MenuSound {
        rc2k_core::menu::keys::MenuSound::from_code(self.menu_sound.get())
    }

    pub fn set_sound(&self, sound: rc2k_core::menu::keys::MenuSound) {
        self.menu_sound.set(sound.code());
    }
}

impl OptionWork {
    /// First values the image's (`tests::option_work_starts_as_the_games`).
    pub const fn new() -> Self {
        OptionWork {
            switch_0: PlainDword::new(0x0),
            switch_1: PlainDword::new(0x0),
            mode_changed: PlainDword::new(0x0),
            modes_switch: PlainDword::new(0x1),
            lens_flare: PlainDword::new(0x1),
            unused_percent_a: PlainDword::new(0x64),
            unused_percent_b: PlainDword::new(0x64),
            reflections: PlainDword::new(0x1),
            car_shadows: PlainDword::new(0x1),
            car_detail: PlainDword::new(0x1),
            headlights: PlainDword::new(0x0),
            audio_quality: PlainDword::new(0x3),
            ds3d: PlainDword::new(0x0),
            eax: PlainDword::new(0x0),
            a3d: PlainDword::new(0x0),
            dolby: PlainDword::new(0x0),
            channels: PlainDword::new(0x1),
            _gap_755684: PlainBytes::new([0x40, 0, 0, 0]),
            music_level: PlainDword::new(0x40),
            other_volume: PlainDword::new(0x40),
            menu_sounds_c: PlainDword::new(0x40),
            _gap_755694: PlainBytes::new([0x40, 0, 0, 0]),
            speech_level: PlainDword::new(0x40),
            _gap_75569c: PlainBytes::new([0x40, 0, 0, 0]),
            _gap_7556a0: PlainBytes::new([0x40, 0, 0, 0]),
            menu_sound: PlainDword::new(0x6F),
            typing: PlainFlag::new(false),
            redefining_key: PlainFlag::new(false),
            redefined_key_place: PlainDword::new(0x3E7),
        }
    }
}

impl OptionWork {
    /// Stores the six values of the sound page, in memory order: the audio quality, DS3D, EAX, A3D,
    /// Dolby and the channels (`screens::load_audio_settings`).
    pub fn set_audio_page(&self, values: [u32; 6]) {
        let cells = [
            &self.audio_quality,
            &self.ds3d,
            &self.eax,
            &self.a3d,
            &self.dolby,
            &self.channels,
        ];
        for (cell, value) in cells.into_iter().zip(values) {
            cell.set(value);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn option_work_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().screens.option_work,
        );
    }
}
