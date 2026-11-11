//! The turn's update of the voices on the game's own mixer (from
//! `audio::update_mixer_voices` 0x40BC05, a chunk of `audio::update_voices` 0x40AC77).
//!
//! Before the mix: each playing voice's dirty bits carried out on its channel (the step
//! for 4; the left and right volumes for 1 or 2; for a restart or a voice not yet sounding
//! the channel's position, format, data, length and kind, the voice sounding), a voice
//! that does not loop and whose channel the mixer no longer uses taken to the stopping
//! list's front with its channel freed; then the stopping list emptied, each of its voices
//! marked to be set up again (dirty 0xF, state 0) and its channel freed. After the mix the
//! playing list is looked through once more for voices that ended.

use crate::Invalid;
use crate::audio::voices::{
    DIRTY_FREQUENCY, DIRTY_PAN, DIRTY_RESTART, DIRTY_VOLUME, Link, Lists, MixerChannel, Pool,
    Voice, list,
};

/// `g_soundMode`: the right channel's sign follows the voice's pan sign (surround).
pub const MODE_SURROUND: u32 = 0x200_0000;

/// A voice's dirty bits carried out on its channel; `rate` the mixing rate (not 0 when the
/// frequency is dirty).
fn apply<P: Pool>(pool: &mut P, at: u32, rate: u32, mode: u32) {
    let x = pool.voice(at);
    let restart = x.dirty & DIRTY_RESTART != 0 || x.state != 2;
    pool.update_channel(x.channel, |c| apply_to_channel(c, &x, rate, mode, restart));
    pool.update_voice(at, |v| {
        if restart {
            v.state = 2;
        }
        v.dirty = 0;
    });
}

/// `apply`'s changes to the voice's channel.
fn apply_to_channel(c: &mut MixerChannel, x: &Voice, rate: u32, mode: u32, restart: bool) {
    if x.dirty & DIRTY_FREQUENCY != 0 {
        let f = x.frequency >> (x.format & 0xF);
        c.step = f / rate;
        c.step_fraction = ((((f % rate) as u64) << 32) / rate as u64) as u32;
    }
    if x.dirty & (DIRTY_VOLUME | DIRTY_PAN) != 0 {
        let volume = (x.volume as i32).wrapping_mul(x.volume_factor as i32);
        let (l, r) = if x.pan >= 0 {
            (0x40i32.wrapping_sub(x.pan), 0x40)
        } else {
            (0x40, 0x40i32.wrapping_add(x.pan))
        };
        c.left = l.wrapping_mul(volume) >> 6;
        let mut right = r.wrapping_mul(volume) >> 6;
        if mode & MODE_SURROUND != 0 && x.pan_sign < 0 {
            right = right.wrapping_neg();
        }
        c.right = right;
    }
    if restart {
        c.position = 0;
        c.position_fraction = 0;
        c.format = x.format;
        c.data = x.data;
        c.length = x.frames;
        c.kind = x.kind;
    }
}

/// The playing list's voices that ended (not looping, channel no longer used) to the
/// stopping list, a channel fewer counted; `with_apply`: the dirty bits carried out first.
fn playing_pass<P: Pool>(pool: &mut P, lists: &mut Lists, rate: u32, mode: u32, with_apply: bool) {
    let mut link = Link::Playing;
    loop {
        let at = match link {
            Link::Playing => lists.playing,
            Link::Stopping => lists.stopping,
            Link::After(v) => pool.voice(v).next,
        };
        if at == 0 {
            return;
        }
        if with_apply && pool.voice(at).dirty != 0 {
            apply(pool, at, rate, mode);
        }
        let x = pool.voice(at);
        if x.kind != 1 && pool.channel(x.channel).used != 1 {
            pool.update_channel(x.channel, |c| c.used = 0);
            pool.update_voice(at, |v| v.channel = 0);
            match link {
                Link::Playing => lists.playing = x.next,
                Link::Stopping => lists.stopping = x.next,
                Link::After(v) => pool.update_voice(v, |v| v.next = x.next),
            }
            pool.update_voice(at, |v| v.next = lists.stopping);
            lists.stopping = at;
            lists.channels_used = lists.channels_used.wrapping_sub(1);
            continue;
        }
        link = Link::After(at);
    }
}

/// The update's part before the mix (see the module's doc). Refused, before anything is
/// written: a list without its end, a playing voice without its channel, a dirty frequency
/// with no mixing rate (the original divides by 0), a dirty volume or pan on a voice that
/// ended (the original writes through the pan).
pub fn before_mix<P: Pool>(
    pool: &mut P,
    lists: &mut Lists,
    rate: u32,
    mode: u32,
) -> Result<(), Invalid> {
    for at in list(pool, lists.playing)? {
        let x = pool.voice(at);
        if x.channel == 0 {
            return Err(Invalid("a playing voice without its channel"));
        }
        if x.dirty & DIRTY_FREQUENCY != 0 && rate == 0 {
            return Err(Invalid("no mixing rate (the original divides by 0)"));
        }
        if x.dirty & (DIRTY_VOLUME | DIRTY_PAN) != 0
            && x.kind != 1
            && pool.channel(x.channel).used != 1
        {
            return Err(Invalid(
                "a changed volume or pan on a voice that ended (the original writes through the pan)",
            ));
        }
    }
    list(pool, lists.stopping)?;
    playing_pass(pool, lists, rate, mode, true);
    let mut at = lists.stopping;
    lists.stopping = 0;
    while at != 0 {
        let channel = pool.update_voice(at, |x| {
            x.dirty |= 0xF;
            x.state = 0;
            x.channel
        });
        if channel != 0 {
            pool.update_channel(channel, |c| c.used = 0);
            pool.update_voice(at, |v| v.channel = 0);
        }
        let next = pool.update_voice(at, |v| std::mem::replace(&mut v.next, 0));
        at = next;
    }
    Ok(())
}

/// The update's part after the mix: the voices that ended since. Refused: a playing list
/// without its end.
pub fn after_mix<P: Pool>(
    pool: &mut P,
    lists: &mut Lists,
    rate: u32,
    mode: u32,
) -> Result<(), Invalid> {
    list(pool, lists.playing)?;
    playing_pass(pool, lists, rate, mode, false);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::voices::{MixerChannel, SampleRecord, Voice};

    struct Test {
        voices: Vec<Voice>,
        channels: Vec<MixerChannel>,
    }

    impl Test {
        fn voice_mut(&mut self, at: u32) -> &mut Voice {
            &mut self.voices[((at - 0x1000) / 0x70) as usize]
        }
        fn channel_mut(&mut self, at: u32) -> &mut MixerChannel {
            &mut self.channels[((at - 0x8000) / 0x58) as usize]
        }
    }

    impl Pool for Test {
        fn voice(&self, at: u32) -> Voice {
            self.voices[((at - 0x1000) / 0x70) as usize]
        }
        fn set_voice(&mut self, at: u32, v: Voice) {
            *self.voice_mut(at) = v;
        }
        fn channel(&self, at: u32) -> MixerChannel {
            self.channels[((at - 0x8000) / 0x58) as usize]
        }
        fn set_channel(&mut self, at: u32, c: MixerChannel) {
            *self.channel_mut(at) = c;
        }
        fn channel_at(&self, n: u32) -> u32 {
            0x8000 + 0x58 * n
        }
        fn record(&self, _: u32) -> SampleRecord {
            unreachable!()
        }
    }

    fn v(n: u32) -> u32 {
        0x1000 + 0x70 * n
    }
    fn c(n: u32) -> u32 {
        0x8000 + 0x58 * n
    }

    #[test]
    fn update() {
        let mut t = Test {
            voices: vec![Voice::default(); 4],
            channels: vec![MixerChannel::default(); 4],
        };
        // 0 looping and dirty, 1 ended, 2 on the stopping list
        *t.voice_mut(v(0)) = Voice {
            kind: 1,
            channel: c(0),
            next: v(1),
            dirty: 0x7,
            frequency: 44100,
            format: 0x121,
            volume: 0x40,
            volume_factor: 0x40,
            pan: -0x20,
            pan_sign: -1,
            ..Voice::default()
        };
        t.channel_mut(c(0)).used = 1;
        *t.voice_mut(v(1)) = Voice {
            channel: c(1),
            state: 2,
            ..Voice::default()
        };
        *t.voice_mut(v(2)) = Voice {
            channel: c(2),
            state: 2,
            ..Voice::default()
        };
        t.channel_mut(c(2)).used = 1;
        let mut l = Lists {
            playing: v(0),
            stopping: v(2),
            channels_used: 2,
            channel_count: 4,
        };
        before_mix(&mut t, &mut l, 22050, MODE_SURROUND).unwrap();
        assert_eq!((l.playing, l.stopping, l.channels_used), (v(0), 0, 1));
        let ch = t.channel(c(0));
        assert_eq!(
            (ch.step, ch.step_fraction, ch.left, ch.right),
            (1, 0, 0x1000, -0x800)
        );
        assert_eq!((t.voice(v(0)).state, t.voice(v(0)).dirty), (2, 0));
        assert_eq!((t.voice(v(1)).channel, t.voice(v(1)).next), (0, 0));
        assert_eq!((t.voice(v(2)).dirty, t.channel(c(2)).used), (0xF, 0));
        after_mix(&mut t, &mut l, 22050, 0).unwrap();
        assert_eq!(l.playing, v(0));
        t.voice_mut(v(0)).channel = 0;
        assert!(before_mix(&mut t, &mut l, 22050, 0).is_err());
    }
}
