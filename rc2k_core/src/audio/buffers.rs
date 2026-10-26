//! A voice's DirectSound buffer: what the leaves around it decide (converted 2026-10-21
//! from `audio::play_buffer` 0x40B922, `audio::set_buffer_place` 0x40C676,
//! `audio::fill_buffer` 0x40B82B, `audio::restore_buffer` 0x40A862,
//! `audio::keep_sample` 0x40B192 and `audio::close_primary_mixer` 0x40BBCB). The COM calls
//! stay the platform's.

use super::voices::Voice;

/// `DSERR_BUFFERLOST`: a failed call of a buffer whose memory went (`g_soundBufferLost` set).
pub const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

/// Whether a buffer call's result says its memory was lost.
pub fn lost(result: i32) -> bool {
    result == BUFFER_LOST
}

/// `GetStatus`' bit: the buffer's memory was lost.
pub const STATUS_LOST: u32 = 2;

/// `audio::restore_buffer`: `Restore` only for a buffer whose status says lost.
pub fn needs_restore(status: u32) -> bool {
    status & STATUS_LOST != 0
}

/// A voice's kind that loops.
pub const KIND_LOOPING: u32 = 1;
/// `Play`'s flag: over and over.
pub const PLAY_LOOPING: u32 = 1;

/// `audio::play_buffer`'s flags: looping for a voice of the looping kind.
pub fn play_flags(v: &Voice) -> u32 {
    if v.kind == KIND_LOOPING {
        PLAY_LOOPING
    } else {
        0
    }
}

/// A voice's state while it sounds.
pub const SOUNDING: u32 = 2;

/// `audio::set_buffer_place`: deferred (1) while the voice sounds, else at once (0).
pub fn place_deferral(v: &Voice) -> u32 {
    u32::from(v.state == SOUNDING)
}

/// `audio::fill_buffer`: only a voice with a buffer of its own (none shared) is filled.
pub fn fills(v: &Voice) -> bool {
    v.buffer != 0 && v.source == 0
}

/// A copy into a locked part: from, to, bytes.
pub type Copy = (u32, u32, u32);

/// `audio::fill_buffer`'s copies from the voice's data into the locked parts (first
/// pointer and size, second pointer and size): all of the first, and the second (on from
/// where the first ended) only when it has a place and bytes.
pub fn fill_copies(data: u32, parts: [u32; 4]) -> (Copy, Option<Copy>) {
    let [p1, n1, p2, n2] = parts;
    let second = (p2 != 0 && n2 != 0).then_some((data.wrapping_add(n1), p2, n2));
    ((data, p1, n1), second)
}

/// `audio::keep_sample`: only a voice with data, a length and a `data_key` has its data
/// copied to the samples' memory.
pub fn keeps_sample(v: &Voice) -> bool {
    v.data != 0 && v.length != 0 && v.data_key != 0
}

/// `audio::close_primary_mixer`: the primary buffer made again only in the mixer's mode
/// with a DirectSound (`direct_sound` asked only in that mode).
pub fn remakes_primary(mode: u32, direct_sound: impl FnOnce() -> u32) -> bool {
    mode & super::device::MIXER != 0 && direct_sound() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves() {
        let mut v = Voice::default();
        assert_eq!(
            (play_flags(&v), place_deferral(&v), fills(&v)),
            (0, 0, false)
        );
        v.kind = 1;
        v.state = 2;
        v.buffer = 5;
        assert_eq!(
            (play_flags(&v), place_deferral(&v), fills(&v)),
            (1, 1, true)
        );
        v.source = 3;
        assert!(!fills(&v) && !keeps_sample(&v));
        assert!(lost(BUFFER_LOST) && !lost(1));
        assert!(needs_restore(3) && !needs_restore(1));
        assert_eq!(fill_copies(100, [7, 10, 0, 5]), ((100, 7, 10), None));
        assert_eq!(
            fill_copies(100, [7, 10, 9, 5]),
            ((100, 7, 10), Some((110, 9, 5)))
        );
        assert!(!remakes_primary(0x1_0000, || unreachable!()));
        assert!(remakes_primary(0x4_0000, || 1) && !remakes_primary(0x4_0000, || 0));
    }
}
