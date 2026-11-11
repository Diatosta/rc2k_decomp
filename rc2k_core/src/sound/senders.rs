//! What the game's sound senders decide before they queue a command
//! (from `sound::play_menu_sound` 0x40F059, `sound::play_sound` 0x409B36,
//! `sound::set_music_volume` 0x40F83E and `sound::stop_sound` 0x40F1D1), and the music's
//! command (`sound::play_music` 0x40F80E, `stop_music` 0x40F826, `play_tune_2` .. `_9`
//! 0x40F76F ..).

/// The three volume settings the menus' sounds are played at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuVolume {
    A,
    B,
    C,
}

/// The menus' sounds by the first bit set, in this order: (bit, sound, volume).
pub const MENU_SOUNDS: [(u32, u32, MenuVolume); 9] = [
    (0x80, 0xE4, MenuVolume::A),
    (0x10, 0x163, MenuVolume::B),
    (0x20, 0x197, MenuVolume::B),
    (0x40, 0x1CB, MenuVolume::B),
    (0x100, 0x1CC, MenuVolume::C),
    (0x08, 0x1CA, MenuVolume::C),
    (0x02, 0x1C8, MenuVolume::C),
    (0x04, 0x1C9, MenuVolume::C),
    (0x01, 0x1C7, MenuVolume::C),
];
/// The time between two menu sounds, in milliseconds.
pub const MENU_SOUND_GAP: u32 = 0x7D;

/// A menu sound asked for (`which` its bits) at `now`, the next allowed at `next`: None
/// for no bits or before the time (by the sign of the difference); else the time after it
/// (moved on before the bits are looked at) and the sound with its volume, if a bit picks
/// one.
pub fn menu_sound(which: u32, now: u32, next: u32) -> Option<(u32, Option<(u32, MenuVolume)>)> {
    if which == 0 || (now.wrapping_sub(next) as i32) < 0 {
        return None;
    }
    let pick = MENU_SOUNDS
        .iter()
        .find(|&&(bit, ..)| which & bit != 0)
        .map(|&(_, sound, volume)| (sound, volume));
    Some((now.wrapping_add(MENU_SOUND_GAP), pick))
}

/// The sound mode bit of the modes that give a sound a place in space.
pub const MODE_3D: u32 = 0x10_0000;
/// No place: what the three place coordinates are sent as outside the 3D modes.
pub const NO_PLACE: u32 = 0xFF80_0000;

/// The four values command 1 carries besides the sound, from the sender's `place`
/// (a volume or distance value, then two coordinates): None while sounds are held back
/// (1); outside the 3D modes no place and the first value in the second slot, else the
/// place with a zero in the second slot.
pub fn sound_place(held_back: u32, mode: u32, place: [u32; 3]) -> Option<[u32; 4]> {
    if held_back == 1 {
        return None;
    }
    let [first, second, third] = place;
    Some(if mode & MODE_3D == 0 {
        [NO_PLACE, first, NO_PLACE, NO_PLACE]
    } else {
        [first, 0, second, third]
    })
}

/// Whether a music volume is to be sent: not the one in use.
pub fn new_music_volume(volume: u32, in_use: u32) -> bool {
    volume != in_use
}

/// The sound a request stops (its first dword), unless it has bit 0x8000 (none).
pub fn stopped(request: u32) -> Option<u32> {
    (request & 0x8000 == 0).then_some(request)
}

/// Command 0xB, the music's: the tune, the tune to follow, the volume and the action it is sent
/// with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Music {
    pub tune: u32,
    /// The tune to go on with when this one ends (the "then" tune of the music command).
    pub then: u32,
    pub volume: u32,
    /// 2 play, 3 change the volume, 0 stop.
    pub action: u32,
}

impl Music {
    pub fn play(tune: u32, then: u32, volume: u32) -> Self {
        Music {
            tune,
            then,
            volume,
            action: 2,
        }
    }

    /// `sound::stop_music`'s: four zeros.
    pub const STOP: Music = Music {
        tune: 0,
        then: 0,
        volume: 0,
        action: 0,
    };
}

/// The volume a tune is started at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuneVolume {
    /// 0x40.
    Full,
    /// The music's volume setting.
    Music,
    /// Tune 8's own volume setting.
    Other,
}

/// `sound::play_tune_n` (2 .. 9): the tune, the one to go on with and its volume. Tune 8
/// is started only while its switch is 1, else the music is stopped.
pub fn tune(n: u32) -> Option<(u32, u32, TuneVolume)> {
    Some(match n {
        2 => (2, 0, TuneVolume::Full),
        3 => (3, 3, TuneVolume::Music),
        4 => (4, 4, TuneVolume::Music),
        5 => (5, 0, TuneVolume::Music),
        6 => (6, 0, TuneVolume::Music),
        7 => (7, 4, TuneVolume::Music),
        8 => (8, 0xA, TuneVolume::Other),
        9 => (9, 9, TuneVolume::Music),
        _ => return None,
    })
}

/// The command `play_tune_n` sends, with the volumes and tune 8's switch read.
pub fn tune_command(
    n: u32,
    music_volume: u32,
    other_volume: u32,
    tune_8_on: bool,
) -> Option<Music> {
    let (tune, then, volume) = tune(n)?;
    if n == 8 && !tune_8_on {
        return Some(Music::STOP);
    }
    let volume = match volume {
        TuneVolume::Full => 0x40,
        TuneVolume::Music => music_volume,
        TuneVolume::Other => other_volume,
    };
    Some(Music::play(tune, then, volume))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn senders() {
        assert_eq!(
            menu_sound(0x21, 100, 50),
            Some((225, Some((0x197, MenuVolume::B))))
        );
        assert_eq!((menu_sound(1, 49, 50), menu_sound(0, 100, 0)), (None, None));
        assert_eq!(menu_sound(0x200, 100, 0), Some((225, None)));
        assert_eq!(sound_place(0, MODE_3D, [1, 2, 3]), Some([1, 0, 2, 3]));
        assert_eq!(
            sound_place(0, 0, [1, 2, 3]),
            Some([NO_PLACE, 1, NO_PLACE, NO_PLACE])
        );
        assert_eq!(
            (sound_place(1, 0, [0; 3]), stopped(0x8001), stopped(5)),
            (None, None, Some(5))
        );
    }

    #[test]
    fn tunes() {
        assert_eq!(tune_command(2, 9, 9, true), Some(Music::play(2, 0, 0x40)));
        assert_eq!(
            tune_command(7, 0x33, 9, true),
            Some(Music::play(7, 4, 0x33))
        );
        assert_eq!(
            tune_command(8, 1, 0x22, true),
            Some(Music::play(8, 0xA, 0x22))
        );
        assert_eq!(tune_command(8, 1, 0x22, false), Some(Music::STOP));
        assert_eq!(tune_command(1, 1, 1, true), None);
    }
}
