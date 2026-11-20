//! The sound modes chosen: the advanced audio screen's options made the race's mode, and the menus'
//! 0x11221.
//!
//! The first call fills the options from the race's mode (quality: the first of the three
//! quality modes whose low byte the mode has, else 3; the others from the mode's bits, set
//! only) and the three facts, the choices as the applied. Later calls apply the first
//! choice that changed of quality, channels, DS3D, EAX, Dolby, in that order and only
//! one of the three that exclude each other (DS3D on clears Dolby, off clears EAX;
//! EAX on sets DS3D and clears Dolby; Dolby on clears DS3D and EAX), then build the race
//! mode: the quality's mode, 0x400000 channels, DS3D 0x101000 (+ 0x800000 with free hardware
//! 3D buffers and the voice manager on), EAX 0x1000000, Dolby 0x2040000; no way bit: 0x10000;
//! no device bit: 0x1000.

use crate::Invalid;

/// The quality modes.
pub(crate) const QUALITIES: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioOptions {
    pub kept: u32,
    /// The screen's choices: quality, channels, DS3D, EAX, Dolby.
    pub choice: [u32; CHOICES],
    /// The same, as last applied.
    pub applied: [u32; CHOICES],
    pub has_3d: u32,
    pub has_eax: u32,
}

/// How many choices the screen has.
pub(crate) const CHOICES: usize = 5;

/// The choices' places in `choice` and `applied`.
pub const QUALITY: usize = 0;
pub const CHANNELS: usize = 1;
pub const DS3D: usize = 2;
pub const EAX: usize = 3;
pub const DOLBY: usize = 4;

/// What it reads of the machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    pub qualities: [u32; 4],
    pub has_3d: u32,
    pub has_eax: u32,
    pub hw_3d_buffers: u32,
    pub voice_manager_on: u32,
}

/// What the new modes log: "AUD: New modes", the menus' mode (hex), the race's (hex, and
/// the line's end).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logged {
    NewModes,
    Menu(u32),
    Race(u32),
}

/// The menus' sound mode.
pub const MENU_MODE: u32 = 0x1_1221;

pub fn choose_modes(
    o: &mut AudioOptions,
    race: &mut u32,
    menu: &mut u32,
    f: &Facts,
    mut log: impl FnMut(Logged),
) -> Result<(), Invalid> {
    if o.kept != 1 {
        o.kept = 1;
        let mode = *race;
        o.applied[QUALITY] = (0..3u32)
            .find(|&n| (mode ^ f.qualities[n as usize]) & 0xFF == 0)
            .unwrap_or(3);
        let mut set = |n: usize, on: bool| {
            if on {
                o.applied[n] = 1;
            }
        };
        set(CHANNELS, mode & 0x40_0000 != 0);
        set(DS3D, mode & 0x10_1000 == 0x10_1000);
        set(EAX, mode & 0x100_0000 != 0);
        set(DOLBY, mode & 0x200_0000 != 0);
        o.has_3d = f.has_3d;
        o.has_eax = f.has_eax;
        o.choice = o.applied;
        return Ok(());
    }
    if o.choice[QUALITY] >= QUALITIES {
        return Err(Invalid("a sound quality past the table"));
    }
    let (c, a) = (o.choice, &mut o.applied);
    a[QUALITY] = c[QUALITY];
    a[CHANNELS] = c[CHANNELS];
    if c[DS3D] != a[DS3D] {
        a[DS3D] = c[DS3D];
        if c[DS3D] != 0 {
            a[DOLBY] = 0;
        } else {
            a[EAX] = 0;
        }
    } else if c[EAX] != a[EAX] && {
        a[EAX] = c[EAX];
        c[EAX] != 0
    } {
        a[DS3D] = 1;
        a[DOLBY] = 0;
    } else if c[DOLBY] != a[DOLBY] {
        a[DOLBY] = c[DOLBY];
        if c[DOLBY] != 0 {
            a[DS3D] = 0;
            a[EAX] = 0;
        }
    }
    let mut mode = f.qualities[a[QUALITY] as usize];
    if a[CHANNELS] != 0 {
        mode |= 0x40_0000;
    }
    if a[DS3D] != 0 {
        mode |= 0x10_1000;
        if f.hw_3d_buffers != 0 && f.voice_manager_on == 1 {
            mode |= 0x80_0000;
        }
    }
    if a[EAX] != 0 {
        mode |= 0x100_0000;
    }
    if a[DOLBY] != 0 {
        mode |= 0x204_0000;
    }
    if mode & 0xF_0000 == 0 {
        mode |= 0x1_0000;
    }
    if mode & 0xF000 == 0 {
        mode |= 0x1000;
    }
    *race = mode;
    *menu = MENU_MODE;
    log(Logged::NewModes);
    log(Logged::Menu(*menu));
    log(Logged::Race(*race));
    Ok(())
}

/// The mixer channels of a sound mode (`audio::set_mode`): 8, 0x10 with bit 0x400000; held to the
/// free hardware buffers of the mode's kind (`buffers`: the caps' all and static counts, 2D +0x24 /
/// +0x28, 3D with bit 0x100000 +0x3C / +0x40; bit 0x10000 holds to all, else 0x20000 to static)
/// when there are any, and at least 6. Also the count looked at last (the second result).
pub fn channels(mode: u32, buffers_2d: [u32; 2], buffers_3d: [u32; 2]) -> (u32, u32) {
    let mut channels: i32 = if mode & 0x40_0000 != 0 { 0x10 } else { 8 };
    let [all, other] = if mode & 0x10_0000 == 0 {
        buffers_2d
    } else {
        buffers_3d
    };
    let (mut looked, mut held) = (all, mode & 0x1_0000 != 0);
    if !held {
        looked = other;
        held = mode & 0x2_0000 != 0;
    }
    let most = looked as i32;
    if held && most != 0 {
        channels = channels.min(most).max(6);
    }
    (channels as u32, looked)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes() {
        let f = Facts {
            qualities: [0x21, 0x22, 0x24, 0x28],
            has_3d: 1,
            has_eax: 1,
            hw_3d_buffers: 4,
            voice_manager_on: 1,
        };
        let mut o = AudioOptions::default();
        let (mut race, mut menu) = (0x10_1022u32, 0);
        choose_modes(&mut o, &mut race, &mut menu, &f, |_| {}).unwrap();
        assert_eq!(
            (o.kept, o.applied[QUALITY], o.applied[DS3D], o.choice),
            (1, 1, 1, o.applied)
        );
        // EAX chosen: DS3D stays, the mode built.
        o.choice[EAX] = 1;
        let mut logged = Vec::new();
        choose_modes(&mut o, &mut race, &mut menu, &f, |l| logged.push(l)).unwrap();
        assert_eq!(race, 0x22 | 0x10_1000 | 0x80_0000 | 0x100_0000 | 0x1_0000);
        assert_eq!(
            logged,
            [
                Logged::NewModes,
                Logged::Menu(MENU_MODE),
                Logged::Race(race)
            ]
        );
        // Dolby chosen: DS3D and EAX go.
        o.choice[DOLBY] = 1;
        choose_modes(&mut o, &mut race, &mut menu, &f, |_| {}).unwrap();
        assert_eq!(o.applied, [1, 0, 0, 0, 1]);
        assert_eq!(race, 0x22 | 0x204_0000 | 0x1000);
        o.choice[QUALITY] = 4;
        assert!(choose_modes(&mut o, &mut race, &mut menu, &f, |_| {}).is_err());
        assert_eq!(channels(0x1_0000, [4, 9], [0, 0]), (6, 4));
        assert_eq!(channels(0x50_0000, [4, 9], [0, 12]), (0x10, 12));
        assert_eq!(channels(0x12_0000, [4, 9], [0, 12]), (8, 12));
        assert_eq!(channels(0x52_0000, [4, 9], [0, 12]), (12, 12));
    }
}
