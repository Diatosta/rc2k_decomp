//! The options pages' small steps (item and left/right handlers, agent E 2026-10-19):
//! - the volume steps (`menu::music_volume_down` 0x467F48 .. `menu_sounds_b_volume_up`
//!   0x46810A): a work value one less (below 0: 0) or one more (above 0x40: 0x40), signed,
//!   then: the music's (`dword_755688`) into `g_musicVolume` and `sound::set_music_volume`
//!   (eax the value); the speech's (`dword_755698`) heard with `sound::start_stream` (eax
//!   the path ";3files\%speech\rally\st1.wav", ecx the value); two of the menu sounds'
//!   (`dword_75569C`, `dword_7556A0`) into `dword_7554E8` and `sound::play_menu_sound`
//!   with eax 0x10 / 0x20 (the first's) or 0x40 (the second's). Not `pusha`'d: they leave
//!   eax (the value, the path or the sound), ecx (the speech's: the value) as loaded.
//! - `menu::next_screen_mode` (0x46FFEF): `dword_4F50C8` one more, 0 past `dword_4F50E4` -
//!   1 (signed); `dword_755648` flipped; leaves eax `dword_755648`, ebx the last mode.
//! - `menu::previous_texture_setting` (0x47012B), `next_texture_setting` (0x47015B):
//!   `dword_755664` cycled through 0 .. 2 and copied to `dword_75AD14`; leave eax it.
//! - `menu::next_colour_a` .. `previous_colour_b` (0x471AE4, 0x471B16, 0x471B48, 0x471B7A):
//!   `dword_7554C0` (or `dword_7554C4`) cycled through 0 .. 6 and the entry of
//!   `dword_758A66` (or `dword_758A46`) it picks into `unk_758A2E` (or `unk_758A32`).
//!   Every register kept.
//! - `menu::apply_audio_choices` (0x470766): the advanced audio page's six work values
//!   (quality, channels, Dolby, A3D, EAX, DS3D) into `g_audioOptions`' choices,
//!   `audio::choose_modes`, and the six as applied back. Every register kept.
//!
//! The names of the texture and colour steps are readings of their variables.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, Regs, game_addr, game_read, game_write, rva};
use crate::menu::parts::Part;
use crate::screens::harness::Left;

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
    pub value_rva: usize,
    pub up: bool,
    pub then: Then,
}

const MUSIC_RVA: usize = rva(0x5180FC); // g_musicVolume
const SOUNDS_VOLUME_RVA: usize = rva(0x7554E8); // dword_7554E8
const SPEECH_PATH_RVA: usize = rva(0x50962F); // ";3files\%speech\rally\st1.wav"
pub const MUSIC_LEVEL_RVA: usize = rva(0x755688); // dword_755688
pub const SPEECH_LEVEL_RVA: usize = rva(0x755698); // dword_755698
pub const SOUNDS_A_RVA: usize = rva(0x75569C); // dword_75569C
pub const SOUNDS_B_RVA: usize = rva(0x7556A0); // dword_7556A0
/// The Rust version of a volume step: the registers it leaves.
pub unsafe fn volume_step(entry: &Regs, v: Volume, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        // Converted: `rc2k_core::menu::steps::volume`.
        let value = rc2k_core::menu::steps::volume(game_read::<i32>(v.value_rva), v.up);
        game_write(v.value_rva, value);
        match v.then {
            Then::Music => {
                r.eax = value as u32;
                game_write(MUSIC_RVA, r.eax);
            }
            Then::Speech => {
                r.ecx = value as u32;
                r.eax = game_addr(SPEECH_PATH_RVA) as u32;
            }
            Then::Sound(sound) => {
                game_write(SOUNDS_VOLUME_RVA, value);
                r.eax = sound;
            }
        }
        part(0, &mut r)?;
    }
    Ok(r)
}

/// In the game: the senders' and the menu sound's Rust versions.
pub fn volume_part(then: Then) -> fn(u32, &mut Regs) -> Result<(), Refused> {
    fn music(_: u32, r: &mut Regs) -> Result<(), Refused> {
        crate::sound::senders::send_in_game(&crate::sound::set_music_volume::SENDER, r);
        Ok(())
    }
    fn speech(_: u32, r: &mut Regs) -> Result<(), Refused> {
        crate::sound::senders::send_in_game(&crate::sound::start_stream::SENDER, r);
        Ok(())
    }
    fn sound(_: u32, r: &mut Regs) -> Result<(), Refused> {
        crate::sound::play_menu_sound::play_menu_sound_in_game(r);
        Ok(())
    }
    match then {
        Then::Music => music,
        Then::Speech => speech,
        Then::Sound(_) => sound,
    }
}

/// `dispatch_volume` without its hook (typed tables).
pub fn volume_entry(r: &mut Regs, v: Volume) -> Result<(), Refused> {
    crate::menu::parts::parted_entry(
        r,
        &|r, part| unsafe { volume_step(r, v, part) },
        volume_part(v.then),
    )
}

// ---- the plain steps (no calls) ----

const MODE_RVA: usize = rva(0x4F50C8); // dword_4F50C8: the screen mode chosen
const MODES_RVA: usize = rva(0x4F50E4); // dword_4F50E4: how many
const MODE_CHANGED_RVA: usize = rva(0x755648); // dword_755648
const TEXTURE_RVA: usize = rva(0x755664); // dword_755664
const TEXTURE_COPY_RVA: usize = rva(0x75AD14); // dword_75AD14

/// `menu::next_screen_mode`: what it leaves.
pub fn next_screen_mode() -> Left {
    unsafe {
        // Converted: `rc2k_core::menu::steps::next_mode`.
        let last = game_read::<u32>(MODES_RVA).wrapping_sub(1);
        let mode = rc2k_core::menu::steps::next_mode(game_read(MODE_RVA), game_read(MODES_RVA));
        game_write(MODE_RVA, mode);
        let changed = game_read::<u32>(MODE_CHANGED_RVA) ^ 1;
        game_write(MODE_CHANGED_RVA, changed);
        Left::nothing().with(EAX, changed).with(EBX, last)
    }
}

/// `menu::previous_texture_setting` / `next_texture_setting`: what it leaves.
pub fn step_texture_setting(up: bool) -> Left {
    unsafe {
        // Converted: `rc2k_core::menu::steps::texture`.
        let v = rc2k_core::menu::steps::texture(game_read(TEXTURE_RVA), up);
        game_write(TEXTURE_RVA, v);
        game_write(TEXTURE_COPY_RVA, v);
        Left::nothing().with(EAX, v)
    }
}

/// One of the colour choices: (its number, the table, where the pick goes).
#[derive(Clone, Copy)]
pub struct Colour {
    pub number_rva: usize,
    pub table_rva: usize,
    pub pick_rva: usize,
}

pub const COLOUR_A: Colour = Colour {
    number_rva: rva(0x7554C0),
    table_rva: rva(0x758A66),
    pick_rva: rva(0x758A2E),
};
pub const COLOUR_B: Colour = Colour {
    number_rva: rva(0x7554C4),
    table_rva: rva(0x758A46),
    pick_rva: rva(0x758A32),
};

/// `menu::next_colour_a` and the three others.
pub fn step_colour(c: Colour, up: bool) {
    unsafe {
        // Converted: `rc2k_core::menu::steps::colour`.
        let n = rc2k_core::menu::steps::colour(game_read(c.number_rva), up);
        game_write(c.number_rva, n);
        let pick = game_read::<u32>(c.table_rva + 4 * n as usize);
        game_write(c.pick_rva, pick);
    }
}

/// `dispatch_plain` without its hook (typed tables).
pub fn plain_entry(r: &mut Regs, rust: crate::service::harness::Rust) -> Result<(), Refused> {
    crate::service::harness::state_entry(r, rust)
}

// ---- the advanced audio page's choices ----

/// (the work value, the choice, the applied) by `g_audioOptions`' fields.
const AUDIO: [(usize, usize, usize); 6] = [
    (0x75566C, 0x506C8C, 0x506CA4), // quality
    (0x755680, 0x506C90, 0x506CA8), // channels
    (0x75567C, 0x506CA0, 0x506CB8), // Dolby
    (0x755678, 0x506C9C, 0x506CB4), // A3D
    (0x755674, 0x506C98, 0x506CB0), // EAX
    (0x755670, 0x506C94, 0x506CAC), // DS3D
];
/// `menu::apply_audio_choices`: the registers it leaves (`popa`'s).
pub unsafe fn apply_audio_choices(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe {
        for (work, choice, _) in AUDIO {
            game_write(rva(choice), game_read::<u32>(rva(work)));
        }
        // eax at the call: the last value copied (DS3D's).
        let mut g = Regs {
            eax: game_read::<u32>(rva(AUDIO[AUDIO.len() - 1].0)),
            ..*entry
        };
        part(0, &mut g)?;
        for (work, _, applied) in AUDIO {
            game_write(rva(work), game_read::<u32>(rva(applied)));
        }
    }
    Ok(*entry)
}

/// In the game: `audio::choose_modes`' Rust version with the game's log.
pub fn audio_part(_: u32, _: &mut Regs) -> Result<(), Refused> {
    crate::audio::choose_modes::in_game(&mut crate::audio::choose_modes::real_log)
}
