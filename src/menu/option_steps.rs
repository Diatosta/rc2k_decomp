//! The options pages' small steps (item and left/right handlers):
//! - the volume steps (`menu::music_volume_down` 0x467F48 .. `menu_sounds_b_volume_up`
//!   0x46810A): a work value one less (below 0: 0) or one more (above 0x40: 0x40), signed,
//!   then: the music's (`option_work.music_level`) into `g_musicVolume` and `sound::set_music_volume`
//!   (eax the value); the speech's (`option_work.speech_level`) heard with `sound::start_stream` (eax
//!   the path ";3files\%speech\rally\st1.wav", ecx the value); two of the menu sounds'
//!   (`audio.levels.sounds_a`, `sounds_b`) into `screen_flow.menu_sounds_volume` and `sound::play_menu_sound`
//!   with eax 0x10 / 0x20 (the first's) or 0x40 (the second's).
//! - `menu::next_screen_mode` (0x46FFEF): the chosen screen mode one more, 0 past the
//!   last mode (signed); `option_work.mode_changed` flipped; leaves eax that flag, ebx the last mode.
//! - `menu::previous_texture_setting` (0x47012B), `next_texture_setting` (0x47015B):
//!   `option_work.car_detail` cycled through 0 .. 2 and copied to the texture setting; leave eax it.
//! - `menu::next_colour_a` .. `previous_colour_b` (0x471AE4, 0x471B16, 0x471B48, 0x471B7A):
//!   `screen_flow.start_section` (or `start_section_a8`) cycled through 0 .. 6 and the entry
//!   of the colour list it picks into a caption list slot.
//! - `menu::apply_audio_choices` (0x470766): the advanced audio page's six work values
//!   (quality, channels, Dolby, A3D, EAX, DS3D) into `g_audioOptions`' choices,
//!   `audio::choose_modes`, and the six as applied back.
//!
//! The names of the texture and colour steps are readings of their variables.

use crate::audio::names_store::SampleNames;
use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::menu::owner::Screens;
use crate::menu::owner::Tables;
use crate::session::Dword;

// ---- the volume steps ----

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Then {
    Music,
    Speech,
    /// `play_menu_sound` with this.
    Sound(u32),
}

#[derive(Clone, Copy)]
pub struct Volume {
    pub value: &'static Dword,
    pub up: bool,
    pub then: Then,
}

pub const MUSIC_LEVEL_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.music_level;
pub const SPEECH_LEVEL_RVA: &Dword = &crate::menu::owner::MENUS.screens.option_work.speech_level;
pub const SOUNDS_A_RVA: &Dword = &crate::audio::owner::AUDIO_OWNER.levels.sounds_a;
pub const SOUNDS_B_RVA: &Dword = &crate::audio::owner::AUDIO_OWNER.levels.sounds_b;
/// One volume step.
pub fn volume_step(screens: &Screens, v: Volume) {
    let audio = crate::audio::owner::audio();
    // The logic is in `rc2k_core::menu::steps::volume`.
    let value = rc2k_core::menu::steps::volume(v.value.get() as i32, v.up);
    v.value.set(value as u32);
    match v.then {
        Then::Music => {
            crate::sound::owner::sound()
                .menu
                .options
                .music_volume
                .set(value as u32);
            crate::sound::set_music_volume::set_music_volume(value as u32, audio);
        }
        Then::Speech => crate::sound::start_stream::start_stream(
            core::mem::offset_of!(SampleNames, menu_speech_st1) as u32,
            value as u32,
        ),
        Then::Sound(sound) => {
            screens.flow.menu_sounds_volume.set(value as u32);
            crate::sound::play_menu_sound::play_menu_sound_in_game(sound);
        }
    }
}

// ---- the plain steps (no calls) ----

/// `menu::next_screen_mode`: the next screen mode chosen, the changed flag flipped.
pub fn next_screen_mode(screens: &Screens) {
    let display = &gfx().display;
    // The logic is in `rc2k_core::menu::steps::next_mode`.
    let mode = rc2k_core::menu::steps::next_mode(
        display.choices.resolution.get(),
        display.choices.resolutions.get(),
    );
    display.choices.resolution.set(mode);
    let changed = screens.option_work.mode_changed.get() ^ 1;
    screens.option_work.mode_changed.set(changed);
}

/// `menu::previous_texture_setting` / `next_texture_setting`: the texture setting one
/// step down or up.
pub fn step_texture_setting(screens: &Screens, up: bool, race: &crate::race::owner::Race) {
    // The logic is in `rc2k_core::menu::steps::texture`.
    let v = rc2k_core::menu::steps::texture(screens.option_work.car_detail.get(), up);
    screens.option_work.car_detail.set(v);
    race.shared.state.settings.car_detail.set(v);
}

/// One of the two start-section picks (IDA's "colour" choices): (its number, the table, where the pick goes).
#[derive(Clone, Copy)]
pub struct StartPick {
    pub number: &'static Dword,
    /// The table of the pictures' string ids, in `CAPTIONS.caption_lists`, from `table_at`.
    pub table_at: usize,
    pub pick: &'static Dword,
}

pub const START_PICK_A: StartPick = StartPick {
    number: &crate::menu::owner::MENUS.screens.flow.start_section,
    table_at: 21,
    pick: &crate::menu::owner::MENUS.tables.captions.caption_lists[7],
};
pub const START_PICK_B: StartPick = StartPick {
    number: &crate::menu::owner::MENUS.screens.flow.start_section_a8,
    table_at: 13,
    pick: &crate::menu::owner::MENUS.tables.captions.caption_lists[8],
};

/// `menu::next_colour_a` and the three others.
pub fn step_start_pick(tables: &Tables, c: StartPick, up: bool) {
    // The logic is in `rc2k_core::menu::steps::colour`.
    let n = rc2k_core::menu::steps::colour(c.number.get() as i32, up);
    c.number.set(n as u32);
    // `CAPTIONS.caption_lists` from `table_at`: the original does not check `n` (a kept
    // number past 6 steps on from it); the address is resolved by `MOVED`.
    let pick = crate::session::read_on_in(
        &tables.captions,
        &tables.captions.caption_lists,
        c.table_at.wrapping_add(n as usize),
        "a caption list",
    )
    .unwrap_or_else(|e| crate::fatal::raise::end("a caption list", e));
    c.pick.set(pick);
}

// ---- the advanced audio page's choices ----

/// (the work value, its place among `g_audioOptions`' six choices and six applied).
const CHOICES: [(&Dword, usize); 6] = [
    (
        &crate::menu::owner::MENUS.screens.option_work.audio_quality,
        0,
    ), // quality
    (&crate::menu::owner::MENUS.screens.option_work.channels, 1), // channels
    (&crate::menu::owner::MENUS.screens.option_work.dolby, 5),    // Dolby
    (&crate::menu::owner::MENUS.screens.option_work.a3d, 4),      // A3D
    (&crate::menu::owner::MENUS.screens.option_work.eax, 3),      // EAX
    (&crate::menu::owner::MENUS.screens.option_work.ds3d, 2),     // DS3D
];
/// `menu::apply_audio_choices`.
pub fn apply_audio_choices(audio: &crate::audio::owner::Audio) -> Result<(), Refused> {
    for (work, n) in CHOICES {
        let value = work.get();
        audio.state.audio_options.update(|o| o.choice[n] = value);
    }
    // `audio::choose_modes` with the game's log (the original's eax there, the last
    // value copied, is read by none of it).
    crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)?;
    for (work, n) in CHOICES {
        work.set(audio.state.audio_options.get().applied[n]);
    }

    Ok(())
}
