//! DirectSound's device, primary buffer and turn way: what the audio thread decides around them.
//! The COM calls stay the platform's.
//!
//! The sound mode's bits used here: 0xF000 the device (0x1000 DirectSound; 0x2000 was
//! Aureal's A3D, which a mode read from an old options file may still name: `without_a3d`),
//! 0xF0000 the turn's way (0x10000 the voices' own buffers, 0x40000 the software mixer),
//! 0x100000 3D.

use super::voices::MODE_3D;

/// The device bits.
pub const DEVICE_BITS: u32 = 0xF000;
pub const DIRECT_SOUND: u32 = 0x1000;
/// The device bit of Aureal's A3D, which is no longer a device.
const A3D2: u32 = 0x2000;

/// A saved sound mode without A3D: one that names A3D becomes the same mode on DirectSound (its
/// quality, channels and 3D kept, so an A3D mode plays as DirectSound 3D).
pub fn without_a3d(mode: u32) -> u32 {
    if mode & A3D2 != 0 {
        (mode & !A3D2) | DIRECT_SOUND
    } else {
        mode
    }
}
/// The way bits.
pub const OWN_BUFFERS: u32 = 0x1_0000;
pub const MIXER: u32 = 0x4_0000;

/// `audio::reopen_sound`: DirectSound is opened again when the mode's device bits differ
/// from those it was opened with.
pub fn needs_reopen(mode: u32, opened: u32) -> bool {
    (mode ^ opened) & DEVICE_BITS != 0
}

/// What `audio::open_sound` is made of.
pub trait Opening {
    type Error;
    fn mode(&mut self) -> u32;
    /// "AUD: open sound" and the GUID logged.
    fn log_open(&mut self, guid: u32) -> Result<(), Self::Error>;
    /// "AUD: open failed rc=" and the code logged.
    fn log_failed(&mut self, code: u32) -> Result<(), Self::Error>;
    /// `DirectSoundCreate`'s result.
    fn create(&mut self, guid: u32) -> i32;
    /// `SetCooperativeLevel` 3 (exclusive) with the window: its result.
    fn cooperate(&mut self) -> i32;
}

/// `audio::open_sound`: logs the GUID; then, only with DirectSound's bit,
/// `DirectSoundCreate` and cooperative level 3. A failure (or no DirectSound bit) logs the
/// code (the GUID, or the call's result) and gives false.
pub fn open_sound<O: Opening>(guid: u32, o: &mut O) -> Result<bool, O::Error> {
    let failed = |o: &mut O, code: u32| o.log_failed(code).map(|_| false);
    o.log_open(guid)?;
    if o.mode() & DIRECT_SOUND == 0 {
        return failed(o, guid);
    }
    let r = o.create(guid);
    if r != 0 {
        return failed(o, r as u32);
    }
    let r = o.cooperate();
    if r != 0 {
        return failed(o, r as u32);
    }
    Ok(true)
}

/// The texts `audio::read_caps` logs between the figures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsText {
    /// "AUD: sound caps: flgs/pri/min/max:\t"
    Caps,
    /// " maxhw: all/static/stream (free)\t"
    MaxHardware,
    /// " max3d: all/static/stream (free)\t"
    Max3d,
}

/// A log entry of `audio::read_caps`: a text, or a field of the caps (its offset) as hex,
/// as a number, or as a number ending the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsLog {
    Text(CapsText),
    Hex(u32),
    Int(u32),
    IntLine(u32),
}

/// The caps' size (`DSCAPS`), its first dword.
pub const CAPS_SIZE: u32 = 0x60;

/// What `audio::read_caps` logs, in order: the flags, the primary buffers' rates (+0x10,
/// +8, +0xC), the hardware buffers (+0x14 .. +0x28) and the 3D ones (+0x2C .. +0x40).
pub const CAPS_LOG: [CapsLog; 19] = [
    CapsLog::Text(CapsText::Caps),
    CapsLog::Hex(4),
    CapsLog::Int(0x10),
    CapsLog::Int(8),
    CapsLog::IntLine(0xC),
    CapsLog::Text(CapsText::MaxHardware),
    CapsLog::Int(0x14),
    CapsLog::Int(0x18),
    CapsLog::Int(0x1C),
    CapsLog::Int(0x20),
    CapsLog::Int(0x24),
    CapsLog::IntLine(0x28),
    CapsLog::Text(CapsText::Max3d),
    CapsLog::Int(0x2C),
    CapsLog::Int(0x30),
    CapsLog::Int(0x34),
    CapsLog::Int(0x38),
    CapsLog::Int(0x3C),
    CapsLog::IntLine(0x40),
];

/// The primary buffer's volume kept at its making: 1 is none.
pub const NO_KEPT_VOLUME: u32 = 1;

/// `audio::release_primary`: the kept volume to put back, if any (the kept one is then
/// `NO_KEPT_VOLUME`).
pub fn kept_volume(kept: u32) -> Option<u32> {
    (kept != NO_KEPT_VOLUME).then_some(kept)
}

/// `audio::make_primary`: the primary buffer's flags (`DSBCAPS_PRIMARYBUFFER |
/// DSBCAPS_CTRLVOLUME`, and `DSBCAPS_CTRL3D` in 3D).
pub fn primary_flags(mode: u32) -> u32 {
    let mut flags = 0x81;
    if mode & MODE_3D != 0 {
        flags |= 0x10;
    }
    flags
}

/// The loudest master volume.
pub const LOUDEST: u32 = 0x40;

/// `audio::make_primary`'s master volume: one past `LOUDEST` (unsigned) becomes the kept
/// volume's index (`index`, asked only then), and is stored back (the second answer).
pub fn master_volume(master: u32, index: impl FnOnce() -> u32) -> (u32, bool) {
    if master > LOUDEST {
        (index(), true)
    } else {
        (master, false)
    }
}

/// `audio::make_primary`: a failed listener takes 3D out of the mode.
pub fn without_3d(mode: u32) -> u32 {
    mode & !MODE_3D
}

/// `audio::open_listener`'s codes when it does not ask: no FPU, the listener off.
pub const NO_FPU: u32 = 0xFFFF_FF91;
pub const LISTENER_OFF: u32 = 0xFFFF_FF90;

/// `audio::open_listener`: the code it fails with before asking for the listener (no FPU,
/// or the listener switched off), or None to ask (`no_listener` read only with an FPU).
pub fn listener_refused(has_fpu: u32, no_listener: impl FnOnce() -> u32) -> Option<u32> {
    if has_fpu != 1 {
        Some(NO_FPU)
    } else if no_listener() == 1 {
        Some(LISTENER_OFF)
    } else {
        None
    }
}

/// The turn's way of a mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// The voices' own buffers (`audio::clear_channels`).
    OwnBuffers,
    /// The software mixer on the primary buffer.
    Mixer,
}

/// `audio::open_way` / `close_way`: the way by the mode's bits (the own buffers first);
/// None for neither.
pub fn way(mode: u32) -> Option<Way> {
    if mode & OWN_BUFFERS != 0 {
        Some(Way::OwnBuffers)
    } else if mode & MIXER != 0 {
        Some(Way::Mixer)
    } else {
        None
    }
}

/// `audio::set_up_primary_mixer`'s formats tried in turn: the mode's, then with eight bits
/// (0x10 in its 0xF0 bits), then mono too (0x100 in its 0xF00 bits).
pub fn mixer_formats(mode: u32) -> [u32; 3] {
    let eight = (mode & !0xF0) | 0x10;
    [mode, eight, (eight & !0xF00) | 0x100]
}

/// The mixer's figures from the format's rate and block align (its low word): the most a
/// turn mixes (rate x 2 / 50 x block: 40 ms, wrapping) and the shift (block / 2).
pub fn mixer_figures(rate: u32, block_align: u32) -> (u32, u32) {
    let block = block_align & 0xFFFF;
    let most = (rate.wrapping_add(rate) / 50).wrapping_mul(block);
    (most, block >> 1)
}

/// What `audio::start` reads of the machine, asked only where it reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    /// The caps' flags (+4).
    CapsFlags,
    /// The free hardware buffers (+0x14).
    FreeHardware,
    /// Whether a 3D primary buffer was made.
    Has3d,
    VoiceManagerOn,
    /// The free hardware 3D buffers (+0x3C).
    Free3d,
    HasEax,
}

/// The menus' mode, the probe's first.
pub const MENU_MODE: u32 = 0x1_1221;
/// The caps' flag of hardware mixing, of emulation.
pub const CAPS_HARDWARE: u32 = 0x100;
pub const CAPS_EMULATED: u32 = 0x20;

/// `audio::start`'s probe of an opened DirectSound: with hardware mixing, free hardware
/// buffers, a 3D primary, the voice manager and six free 3D
/// buffers: the 3D bits (0x900000, and 0x1000000 with EAX); else the menus' mode.
pub fn probed_race_mode(fact: impl Fn(Fact) -> u32) -> u32 {
    let mut mode = MENU_MODE;
    if fact(Fact::CapsFlags) & CAPS_HARDWARE != 0
        && fact(Fact::FreeHardware) != 0
        && fact(Fact::Has3d) == 1
        && fact(Fact::VoiceManagerOn) == 1
        && fact(Fact::Free3d) >= 6
    {
        mode |= 0x90_0000;
        if fact(Fact::HasEax) == 1 {
            mode |= 0x100_0000;
        }
    }
    mode
}

/// `audio::start`: the modes are kept only when both are set (the menus' asked only with a
/// race mode).
pub fn modes_kept(race: u32, menu: impl FnOnce() -> u32) -> bool {
    race != 0 && menu() != 0
}

/// The audio thread's turn, in milliseconds.
pub const TURN_MS: u32 = 20;

/// `audio::thread`'s sleep before a turn: what is left of `TURN_MS` since the last turn's
/// tick (signed), at least 1.
pub fn turn_sleep(last: u32, now: u32) -> u32 {
    let left = last.wrapping_sub(now).wrapping_add(TURN_MS);
    if (left as i32) > 0 { left } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        mode: u32,
        create: i32,
        cooperate: i32,
        log: Vec<(char, u32)>,
    }

    impl Opening for Fake {
        type Error = ();
        fn mode(&mut self) -> u32 {
            self.mode
        }
        fn log_open(&mut self, guid: u32) -> Result<(), ()> {
            self.log.push(('o', guid));
            Ok(())
        }
        fn log_failed(&mut self, code: u32) -> Result<(), ()> {
            self.log.push(('f', code));
            Ok(())
        }
        fn create(&mut self, guid: u32) -> i32 {
            self.log.push(('c', guid));
            self.create
        }
        fn cooperate(&mut self) -> i32 {
            self.log.push(('l', 0));
            self.cooperate
        }
    }

    fn fake(mode: u32) -> Fake {
        Fake {
            mode,
            create: 0,
            cooperate: 0,
            log: Vec::new(),
        }
    }

    #[test]
    fn opening() {
        let mut f = fake(0x1000);
        assert_eq!(open_sound(7, &mut f), Ok(true));
        assert_eq!(f.log, [('o', 7), ('c', 7), ('l', 0)].to_vec());
        // A mode without DirectSound's bit opens nothing.
        let mut f = fake(0x2000);
        assert_eq!(open_sound(7, &mut f), Ok(false));
        assert_eq!(f.log, [('o', 7), ('f', 7)].to_vec());
        let mut f = fake(0x1000);
        f.create = -5;
        assert_eq!(open_sound(0, &mut f), Ok(false));
        assert_eq!(f.log.last(), Some(&('f', 0xFFFF_FFFB)));
        // An A3D mode of an old options file: the same mode on DirectSound, its 3D kept.
        assert_eq!(without_a3d(0x11_2221), 0x11_1221);
        assert_eq!(without_a3d(0x10_3022), 0x10_1022);
        assert_eq!(without_a3d(0x91_1221), 0x91_1221);
        assert!(needs_reopen(0x1000, 0x2000) && !needs_reopen(0x1_1000, 0x1000));
    }

    #[test]
    fn primary_and_way() {
        assert_eq!((primary_flags(0), primary_flags(MODE_3D)), (0x81, 0x91));
        assert_eq!(master_volume(0x40, || 9), (0x40, false));
        assert_eq!(master_volume(0x41, || 9), (9, true));
        assert_eq!(listener_refused(0, || unreachable!()), Some(NO_FPU));
        assert_eq!(listener_refused(1, || 1), Some(LISTENER_OFF));
        assert_eq!(listener_refused(1, || 0), None);
        assert_eq!(way(0x5_0000), Some(Way::OwnBuffers));
        assert_eq!((way(0x4_0000), way(0x2_0000)), (Some(Way::Mixer), None));
        assert_eq!(mixer_formats(0x4_1224), [0x4_1224, 0x4_1214, 0x4_1114]);
        assert_eq!(mixer_figures(22050, 0x7_0004), (3528, 2));
        assert_eq!((kept_volume(1), kept_volume(0)), (None, Some(0)));
    }

    #[test]
    fn probe() {
        let facts = |flags, manager, eax| {
            move |f| match f {
                Fact::CapsFlags => flags,
                Fact::FreeHardware => 4,
                Fact::Has3d => 1,
                Fact::VoiceManagerOn => manager,
                Fact::Free3d => 6,
                Fact::HasEax => eax,
            }
        };
        assert_eq!(probed_race_mode(facts(0, 1, 1)), MENU_MODE);
        assert_eq!(probed_race_mode(facts(0x100, 0, 1)), MENU_MODE);
        assert_eq!(probed_race_mode(facts(0x100, 1, 0)), 0x91_1221);
        assert_eq!(probed_race_mode(facts(0x100, 1, 1)), 0x191_1221);
        assert_eq!((turn_sleep(100, 107), turn_sleep(100, 120)), (13, 1));
        assert_eq!(turn_sleep(100, 90), 30);
        assert!(modes_kept(1, || 1) && !modes_kept(0, || unreachable!()));
    }
}
