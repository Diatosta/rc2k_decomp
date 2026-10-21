//! The audio thread's sounds (converted 2026-10-21 from `audio::start_voice` 0x40A9D1,
//! `audio::update_voices` 0x40AC77's own-buffers way, `audio::stop_all_voices` 0x40AFD9,
//! `audio::clear_channels` 0x40ACA1, `audio::find_shared_voice` 0x40B027,
//! `audio::new_voice` 0x40B04C, `audio::replay_looping` 0x40A839): sample records, voices
//! and mixer channels as the game lays them out, reached through a `Pool` by handle (the
//! game's addresses), and the playing and stopping lists linked by `Voice::next`.
//!
//! A sound number picks a sample record (0x1CD); a loaded record points at its first voice
//! (a pool of 0x200); a sound that can sound more than once at a time has more voices,
//! linked by `next_copy`. A voice that sounds is on the playing list and holds a mixer
//! channel; a voice put out of use goes to the stopping list for the next turn to stop.
//! What a command changes in a voice is marked in its `dirty` bits for the turn.

use crate::Invalid;

pub const SAMPLES: u32 = 0x1CD;
pub const VOICES: u32 = 0x200;
/// The mixer channels that are cleared (`audio::clear_channels`).
pub const CHANNELS_CLEARED: u32 = 0x10;

/// "As the sample record says" in a command.
pub const DEFAULT: u32 = 0xFFFF_FF91;
/// "The voice's own frequency".
pub const OWN_FREQUENCY: u32 = 0xFFFF_FF90;
/// "As the sample record says", for a place's float.
pub const DEFAULT_PLACE: u32 = 0xFF80_0000;
/// `g_soundMode`: places in 3D.
pub const MODE_3D: u32 = 0x10_0000;

pub const DIRTY_VOLUME: u32 = 1;
pub const DIRTY_PAN: u32 = 2;
pub const DIRTY_FREQUENCY: u32 = 4;
pub const DIRTY_PLACE: u32 = 8;
pub const DIRTY_PLAY: u32 = 0x10;
pub const DIRTY_RESTART: u32 = 0x20;
/// `Voice::flags`: the voice's sample is loaded.
pub const LOADED: u32 = 0x10;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SampleRecord {
    pub number: u32,
    pub group: u32,
    /// The first voice (0: not loaded).
    pub first_voice: u32,
    pub data_key: u32,
    pub priority: u32,
    pub voice_kind: u32,
    /// How many voices to make for it.
    pub copies: i32,
    /// The sample's volume scale out of 0x40 (`audio::prepare_sample`).
    pub scale: u32,
    /// Points at a volume factor (0: 0x40).
    pub volume_factor: u32,
    /// Its voices' 3D minimum distance (an f32).
    pub min_distance: u32,
    pub volume: u32,
    pub pan: u32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Voice {
    /// 1 a buffer of its own, 2 the software mixer's; `LOADED`.
    pub flags: u32,
    /// Its sample record's number.
    pub sound: u32,
    pub buffer: u32,
    pub buffer_3d: u32,
    pub data_key: u32,
    /// The voice whose sample data this one shares.
    pub source: u32,
    pub next_copy: u32,
    pub data: u32,
    pub length: u32,
    pub own_frequency: u32,
    /// Low four bits: the sample's shift.
    pub format: u32,
    /// The sample's frames.
    pub frames: u32,
    /// The record's `scale`.
    pub scale: u32,
    pub priority: i32,
    /// 0 once, 1 looping, 3 restarted each time.
    pub kind: u32,
    /// 0 free, 1 started, 2 sounding.
    pub state: u32,
    pub dirty: u32,
    /// Its mixer channel (0: none).
    pub channel: u32,
    /// The playing or stopping list's link.
    pub next: u32,
    pub volume_factor: u32,
    /// The 3D buffer's minimum distance (an f32).
    pub min_distance: u32,
    pub volume: u32,
    pub pan: i32,
    pub pan_sign: i32,
    pub frequency: u32,
    pub place: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MixerChannel {
    pub used: u32,
    pub gap_4: [u32; 2],
    pub voice: u32,
    pub format: u32,
    pub data: u32,
    pub length: u32,
    pub kind: u32,
    pub gap_20: [u32; 3],
    pub position: u32,
    pub position_fraction: u32,
    pub step: u32,
    pub step_fraction: u32,
    pub left: i32,
    pub right: i32,
    pub gap_44: [u32; 5],
}

const _: () = assert!(std::mem::size_of::<SampleRecord>() == 0x40);
const _: () = assert!(std::mem::size_of::<Voice>() == 0x70);
const _: () = assert!(std::mem::size_of::<MixerChannel>() == 0x58);

/// Where voices, channels and records are, by handle.
pub trait Pool {
    fn voice(&mut self, at: u32) -> &mut Voice;
    fn channel(&mut self, at: u32) -> &mut MixerChannel;
    /// Channel `n`'s handle.
    fn channel_at(&self, n: u32) -> u32;
    fn record(&mut self, n: u32) -> &mut SampleRecord;
}

/// The lists' heads and the channels' counts (`g_playingVoices`, `g_stoppingVoices`,
/// `g_channelsUsed`, `g_channelCount`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lists {
    pub playing: u32,
    pub stopping: u32,
    pub channels_used: u32,
    pub channel_count: u32,
}

/// A place that points at a voice: a list's head, or a voice's `next`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Playing,
    Stopping,
    After(u32),
}

fn get<P: Pool>(pool: &mut P, lists: &Lists, link: Link) -> u32 {
    match link {
        Link::Playing => lists.playing,
        Link::Stopping => lists.stopping,
        Link::After(v) => pool.voice(v).next,
    }
}

fn set<P: Pool>(pool: &mut P, lists: &mut Lists, link: Link, value: u32) {
    match link {
        Link::Playing => lists.playing = value,
        Link::Stopping => lists.stopping = value,
        Link::After(v) => pool.voice(v).next = value,
    }
}

/// The voices of a list from `head`. Refused: no end within 0x200.
pub fn list<P: Pool>(pool: &mut P, head: u32) -> Result<Vec<u32>, Invalid> {
    let mut out = Vec::new();
    let mut at = head;
    while at != 0 {
        out.push(at);
        if out.len() > VOICES as usize {
            return Err(Invalid("a list of voices without its end"));
        }
        at = pool.voice(at).next;
    }
    Ok(out)
}

/// The voice at `link` taken off its list to the stopping list's front, its channel freed.
fn to_stopping<P: Pool>(pool: &mut P, lists: &mut Lists, link: Link, at: u32) {
    let channel = pool.voice(at).channel;
    pool.channel(channel).used = 0;
    pool.voice(at).channel = 0;
    let next = pool.voice(at).next;
    set(pool, lists, link, next);
    pool.voice(at).next = lists.stopping;
    lists.stopping = at;
}

/// Command 7: the playing list put in front of the stopping list and emptied, no channel
/// counted (the channels keep their marks: the turn frees them). Refused: a playing list
/// without its end.
pub fn stop_all<P: Pool>(pool: &mut P, lists: &mut Lists) -> Result<(), Invalid> {
    let playing = list(pool, lists.playing)?;
    let Some(&last) = playing.last() else {
        return Ok(());
    };
    pool.voice(last).next = lists.stopping;
    lists.stopping = lists.playing;
    lists.playing = 0;
    lists.channels_used = 0;
    Ok(())
}

/// The first 0x10 channels made free: `used`, `voice` and the five dwords from +0x44 0.
pub fn clear_channels<P: Pool>(pool: &mut P) {
    for n in 0..CHANNELS_CLEARED {
        let at = pool.channel_at(n);
        let c = pool.channel(at);
        c.used = 0;
        c.voice = 0;
        c.gap_44 = [0; 5];
    }
}

/// The first voice (of `count`, handles from `voice_at`) whose `data_key` is `key`.
pub fn find_shared<P: Pool>(
    pool: &mut P,
    count: u32,
    voice_at: impl Fn(u32) -> u32,
    key: u32,
) -> Option<u32> {
    (0..count)
        .map(voice_at)
        .find(|&at| pool.voice(at).data_key == key)
}

/// The game's fatal error when the pool is used up.
pub const POOL_USED_UP: u32 = 0x80A2_8501;

/// The next voice of the pool: its place, the count one more. Used up: `None` (the
/// game's fatal error 0x80A28501).
pub fn new_voice(count: &mut u32) -> Option<u32> {
    if *count >= VOICES {
        return None;
    }
    *count += 1;
    Some(*count - 1)
}

/// The playing voices that were sounding (state 2, the voice's +0x3C) and have a buffer,
/// in the list's order (`audio::replay_looping` plays each again after a lost buffer).
/// Refused (before any is played): a list without its end.
pub fn sounding_to_replay<P: Pool>(pool: &mut P, lists: &Lists) -> Result<Vec<u32>, Invalid> {
    Ok(list(pool, lists.playing)?
        .into_iter()
        .filter(|&at| {
            let v = pool.voice(at);
            v.state == 2 && v.buffer != 0
        })
        .collect())
}

/// A command 1's record: the number, then the sender's eax, the volume, pan, frequency and
/// place.
pub type Command = [u32; 8];

/// Where a voice and a command lead.
enum Plan {
    Nothing,
    FreeChannel { voice: u32, channel: u32 },
    Steal { voice: u32, victim: Link },
    Stop { voice: u32, link: Link },
    Change { voice: u32 },
}

fn find_playing<P: Pool>(pool: &mut P, lists: &Lists, voice: u32) -> Result<Option<Link>, Invalid> {
    let mut link = Link::Playing;
    for _ in 0..=VOICES {
        let at = get(pool, lists, link);
        if at == 0 {
            return Ok(None);
        }
        if at == voice {
            return Ok(Some(link));
        }
        link = Link::After(at);
    }
    Err(Invalid("a playing list without its end"))
}

fn plan_start<P: Pool>(pool: &mut P, lists: &Lists, mut voice: u32) -> Result<Plan, Invalid> {
    while pool.voice(voice).state != 0 {
        voice = pool.voice(voice).next_copy;
        if voice == 0 {
            return Ok(Plan::Nothing);
        }
    }
    if lists.channels_used < lists.channel_count {
        let free = (0..lists.channel_count).find_map(|n| {
            let c = pool.channel_at(n);
            (pool.channel(c).used == 0).then_some(c)
        });
        return Ok(free.map_or(Plan::Nothing, |channel| Plan::FreeChannel {
            voice,
            channel,
        }));
    }
    // The playing voice of the lowest priority, if below this one's.
    let mine = pool.voice(voice).priority;
    let mut lowest = mine;
    let mut victim = None;
    let mut link = Link::Playing;
    let mut at = lists.playing;
    if at == 0 {
        return Err(Invalid("no channel left and no voice playing"));
    }
    for _ in 0..=VOICES {
        if lowest > pool.voice(at).priority {
            lowest = pool.voice(at).priority;
            victim = Some(link);
        }
        link = Link::After(at);
        at = get(pool, lists, link);
        if at == 0 {
            return Ok(match victim {
                Some(victim) if lowest != mine => {
                    let v = get(pool, lists, victim);
                    if pool.voice(v).channel == 0 {
                        Plan::Nothing
                    } else {
                        Plan::Steal { voice, victim }
                    }
                }
                _ => Plan::Nothing,
            });
        }
    }
    Err(Invalid("a playing list without its end"))
}

fn plan<P: Pool>(pool: &mut P, lists: &Lists, sound: i32, cmd: &Command) -> Result<Plan, Invalid> {
    if sound < 0 {
        return Ok(Plan::Nothing);
    }
    if sound as u32 >= SAMPLES {
        return Err(Invalid("a sound of 0x1CD or more"));
    }
    let first = pool.record(sound as u32).first_voice;
    if first == 0 || pool.voice(first).flags & LOADED == 0 {
        return Ok(Plan::Nothing);
    }
    let volume = cmd[2];
    if volume != 0 && pool.voice(first).kind == 0 {
        return plan_start(pool, lists, first);
    }
    match find_playing(pool, lists, first)? {
        None if volume == 0 || cmd[1] & 0x8000_0000 != 0 => Ok(Plan::Nothing),
        None => plan_start(pool, lists, first),
        Some(link) if volume == 0 => Ok(Plan::Stop { voice: first, link }),
        Some(_) => Ok(Plan::Change { voice: first }),
    }
}

/// The command's settings into the voice, each change marked dirty; kind 3 a restart; the
/// play bit.
fn change<P: Pool>(pool: &mut P, at: u32, cmd: &Command, mode: u32) {
    let sound = pool.voice(at).sound;
    let record = *pool.record(sound);
    let v = pool.voice(at);
    let mut dirty = v.dirty;
    let or = |given: u32, default: u32| if given == DEFAULT { default } else { given };
    let volume = or(cmd[2], record.volume).min(0x40);
    if volume != v.volume {
        dirty |= DIRTY_VOLUME;
        v.volume = volume;
    }
    let pan = or(cmd[3], record.pan);
    let (low, high) = (pan as u8, (pan >> 8) as u8);
    v.pan_sign = if (low ^ high) & 0x80 != 0 { -1 } else { 0 };
    let pan = (low as i8 as i32).clamp(-0x40, 0x40);
    if pan != v.pan {
        dirty |= DIRTY_PAN;
        v.pan = pan;
    }
    let mut frequency = or(cmd[4], record.frequency);
    if frequency == OWN_FREQUENCY {
        frequency = v.own_frequency;
    }
    let frequency = (frequency as i32).clamp(100, 200_000) as u32;
    if frequency != v.frequency {
        dirty |= DIRTY_FREQUENCY;
        v.frequency = frequency;
    }
    if mode & MODE_3D != 0 {
        for i in 0..3 {
            let p = if cmd[5 + i] == DEFAULT_PLACE {
                record.place[i]
            } else {
                cmd[5 + i]
            };
            if p != v.place[i] {
                dirty |= DIRTY_PLACE;
                v.place[i] = p;
            }
        }
    }
    if v.kind == 3 {
        dirty |= DIRTY_RESTART;
    }
    v.dirty = dirty | DIRTY_PLAY;
}

fn start<P: Pool>(pool: &mut P, lists: &mut Lists, at: u32) {
    pool.voice(at).next = lists.playing;
    lists.playing = at;
    pool.voice(at).state = 1;
    let channel = pool.voice(at).channel;
    pool.channel(channel).voice = at;
}

/// Command 1: a sound started, changed or stopped. With a volume and a voice of kind 0
/// (or, another kind or no volume, a voice not playing, unless the sender's eax has bit
/// 31) its first free copy gets a channel: a free one while fewer are used than there are
/// (counted), else the channel of the playing voice of the lowest priority below its own
/// (none: nothing), which goes to the stopping list; the voice to the playing list's
/// front, started. A playing voice with no volume is stopped (its channel freed and
/// counted down). Then for a voice started or playing the settings (`change`). Refused,
/// before anything: a number of 0x1CD or more (its own or the voice's); no channels left
/// and none playing; a playing list without its end; a voice to stop without its channel.
pub fn start_voice<P: Pool>(
    pool: &mut P,
    lists: &mut Lists,
    mode: u32,
    sound: i32,
    cmd: &Command,
) -> Result<(), Invalid> {
    let plan = plan(pool, lists, sound, cmd)?;
    let target = match plan {
        Plan::Nothing | Plan::Stop { .. } => None,
        Plan::FreeChannel { voice, .. } | Plan::Steal { voice, .. } | Plan::Change { voice } => {
            Some(voice)
        }
    };
    if let Some(at) = target
        && pool.voice(at).sound >= SAMPLES
    {
        return Err(Invalid("a voice whose sound is 0x1CD or more"));
    }
    if let Plan::Stop { voice, .. } = plan
        && pool.voice(voice).channel == 0
    {
        return Err(Invalid("a playing voice without its channel"));
    }
    match plan {
        Plan::Nothing => {}
        Plan::Stop { voice, link } => {
            to_stopping(pool, lists, link, voice);
            lists.channels_used = lists.channels_used.wrapping_sub(1);
        }
        Plan::FreeChannel { voice, channel } => {
            pool.channel(channel).used = 1;
            pool.voice(voice).channel = channel;
            lists.channels_used = lists.channels_used.wrapping_add(1);
            start(pool, lists, voice);
            change(pool, voice, cmd, mode);
        }
        Plan::Steal { voice, victim } => {
            let v = get(pool, lists, victim);
            let channel = pool.voice(v).channel;
            pool.voice(voice).channel = channel;
            pool.voice(v).channel = 0;
            let next = pool.voice(v).next;
            set(pool, lists, victim, next);
            pool.voice(v).next = lists.stopping;
            lists.stopping = v;
            start(pool, lists, voice);
            change(pool, voice, cmd, mode);
        }
        Plan::Change { voice } => change(pool, voice, cmd, mode),
    }
    Ok(())
}

/// What the turn's update asks of a voice's buffer, or of the 3D sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Whether the voice's buffer still plays (the answer).
    IsPlaying(u32),
    Frequency(u32),
    Volume(u32),
    Pan(u32),
    Place(u32),
    Rewind(u32),
    Play(u32),
    Stop(u32),
    /// The EAX room set.
    Room,
    /// The deferred 3D settings carried out.
    Commit,
}

/// The EAX room's variables (`g_eaxRoom`, `g_eaxRoomSet`, `dword_6D1DD4` and the one kept,
/// `dword_506C80`) and the 3D commit flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Room {
    pub asked: u32,
    pub set: u32,
    pub environment: u32,
    pub kept: u32,
    pub commit: u32,
}

/// `g_soundMode`: with EAX.
pub const MODE_EAX: u32 = 0x100_0000;

fn stopping_pass<P: Pool, E>(
    pool: &mut P,
    lists: &mut Lists,
    step: &mut impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let mut link = Link::Stopping;
    loop {
        let at = get(pool, lists, link);
        if at == 0 {
            return Ok(());
        }
        if pool.voice(at).state == 2 {
            step(Step::Stop(at))?;
        }
        pool.voice(at).state = 0;
        let channel = pool.voice(at).channel;
        if channel != 0 {
            pool.channel(channel).used = 0;
            pool.voice(at).channel = 0;
        }
        set(pool, lists, link, 0);
        link = Link::After(at);
    }
}

fn apply<P: Pool, E>(
    pool: &mut P,
    at: u32,
    mode: u32,
    room: &mut Room,
    step: &mut impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let x = *pool.voice(at);
    if x.buffer != 0 {
        if x.dirty & DIRTY_FREQUENCY != 0 {
            step(Step::Frequency(at))?;
        }
        if x.dirty & DIRTY_VOLUME != 0 {
            step(Step::Volume(at))?;
        }
        if x.dirty & DIRTY_PAN != 0 {
            step(Step::Pan(at))?;
        }
        if mode & MODE_3D != 0 && x.buffer_3d != 0 && x.dirty & DIRTY_PLACE != 0 {
            step(Step::Place(at))?;
            room.commit = 1;
        }
        if x.dirty & DIRTY_RESTART != 0 || x.state != 2 {
            step(Step::Rewind(at))?;
            if pool.voice(at).state != 2 {
                step(Step::Play(at))?;
                pool.voice(at).state = 2;
            }
        }
    }
    pool.voice(at).dirty = 0;
    Ok(())
}

/// The turn's update with voices of their own buffers: the stopping list stopped and
/// emptied (each voice free, its channel freed); each playing voice's marked settings
/// carried to its buffer, one not sounding (or to restart) rewound and played; one that
/// does not loop and has ended goes to the stopping list (a channel fewer); the stopping
/// list again; then in 3D the EAX room when asked or the environment changed, and the
/// deferred settings committed. Refused (before anything, as `Invalid` through `E`): a
/// list without its end; a playing voice that does not loop without its channel or
/// buffer; a marked voice's volume or pan past the volume table; a voice to stop without
/// its buffer.
pub fn update_buffers<P: Pool, E: From<Invalid>>(
    pool: &mut P,
    lists: &mut Lists,
    mode: u32,
    room: &mut Room,
    mut step: impl FnMut(Step) -> Result<bool, E>,
) -> Result<(), E> {
    let playing = list(pool, lists.playing)?;
    let stopping = list(pool, lists.stopping)?;
    for &at in &playing {
        let x = *pool.voice(at);
        if x.kind != 1 && (x.channel == 0 || x.buffer == 0) {
            return Err(Invalid("a playing voice without its channel or buffer").into());
        }
        let level = (x.volume as i32).wrapping_mul(x.volume_factor as i32) >> 6;
        if x.buffer != 0
            && x.dirty != 0
            && (!(0..=0x40).contains(&level) || !(-0x40..=0x40).contains(&x.pan))
        {
            return Err(Invalid("a volume or pan past the volume table").into());
        }
    }
    if stopping.iter().any(|&at| {
        let v = pool.voice(at);
        v.state == 2 && v.buffer == 0
    }) {
        return Err(Invalid("a voice to stop without its buffer").into());
    }
    stopping_pass(pool, lists, &mut step)?;
    let mut link = Link::Playing;
    loop {
        let at = get(pool, lists, link);
        if at == 0 {
            break;
        }
        if pool.voice(at).dirty != 0 {
            apply(pool, at, mode, room, &mut step)?;
        }
        if pool.voice(at).kind != 1 && !step(Step::IsPlaying(at))? {
            to_stopping(pool, lists, link, at);
            lists.channels_used = lists.channels_used.wrapping_sub(1);
            continue;
        }
        link = Link::After(at);
    }
    stopping_pass(pool, lists, &mut step)?;
    if mode & MODE_3D != 0 {
        if mode & MODE_EAX != 0 {
            if room.asked != room.set {
                room.set = room.asked;
                room.kept = room.environment;
                step(Step::Room)?;
            } else if room.environment != room.kept {
                room.kept = room.environment;
                step(Step::Room)?;
            }
        }
        if room.commit == 1 {
            step(Step::Commit)?;
            room.commit = 0;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Handles: voices 0x1000 + 0x70 n, channels 0x8000 + 0x58 n.
    struct Test {
        voices: Vec<Voice>,
        channels: Vec<MixerChannel>,
        records: Vec<SampleRecord>,
    }

    impl Pool for Test {
        fn voice(&mut self, at: u32) -> &mut Voice {
            &mut self.voices[((at - 0x1000) / 0x70) as usize]
        }
        fn channel(&mut self, at: u32) -> &mut MixerChannel {
            &mut self.channels[((at - 0x8000) / 0x58) as usize]
        }
        fn channel_at(&self, n: u32) -> u32 {
            0x8000 + 0x58 * n
        }
        fn record(&mut self, n: u32) -> &mut SampleRecord {
            &mut self.records[n as usize]
        }
    }

    fn v(n: u32) -> u32 {
        0x1000 + 0x70 * n
    }

    #[test]
    fn voices_start_steal_stop() {
        let mut t = Test {
            voices: vec![Voice::default(); 8],
            channels: vec![MixerChannel::default(); 0x10],
            records: vec![SampleRecord::default(); 4],
        };
        for n in 0..4u32 {
            t.records[n as usize].first_voice = v(n);
            t.records[n as usize].volume = 0x20;
            *t.voice(v(n)) = Voice {
                flags: LOADED,
                sound: n,
                priority: n as i32,
                ..Voice::default()
            };
        }
        let mut lists = Lists {
            channel_count: 1,
            ..Lists::default()
        };
        let cmd = |volume| [1, 0, volume, DEFAULT, DEFAULT, DEFAULT, 0, 0];
        start_voice(&mut t, &mut lists, 0, 1, &cmd(DEFAULT)).unwrap();
        assert_eq!((lists.playing, lists.channels_used), (v(1), 1));
        assert_eq!(t.voice(v(1)).volume, 0x20);
        assert_eq!(t.voice(v(1)).dirty & DIRTY_PLAY, DIRTY_PLAY);
        // No channel left: 3 takes 1's (a lower priority).
        start_voice(&mut t, &mut lists, 0, 3, &cmd(5)).unwrap();
        assert_eq!((lists.playing, lists.stopping), (v(3), v(1)));
        assert_eq!(t.voice(v(3)).channel, 0x8000);
        // 0 has the lowest priority: nothing.
        start_voice(&mut t, &mut lists, 0, 0, &cmd(5)).unwrap();
        assert_eq!(lists.playing, v(3));
        // Stopped with no volume.
        start_voice(&mut t, &mut lists, 0, 3, &cmd(0)).unwrap();
        assert_eq!(
            (lists.playing, lists.stopping, lists.channels_used),
            (0, v(3), 0)
        );
        assert!(start_voice(&mut t, &mut lists, 0, 0x1CD, &cmd(5)).is_err());
        // The turn: the stopping list emptied.
        t.voice(v(3)).state = 2;
        t.voice(v(3)).buffer = 9;
        let mut steps = Vec::new();
        update_buffers::<_, Invalid>(&mut t, &mut lists, 0, &mut Room::default(), |s| {
            steps.push(s);
            Ok(true)
        })
        .unwrap();
        assert_eq!(steps, [Step::Stop(v(3))]);
        assert_eq!((lists.stopping, t.voice(v(3)).state), (0, 0));
        // stop_all.
        lists.playing = v(0);
        t.voice(v(0)).next = v(2);
        lists.stopping = v(1);
        stop_all(&mut t, &mut lists).unwrap();
        assert_eq!(
            (lists.playing, lists.stopping, t.voice(v(2)).next),
            (0, v(0), v(1))
        );
        let mut count = 0x1FF;
        assert_eq!(new_voice(&mut count), Some(0x1FF));
        assert_eq!(new_voice(&mut count), None);
    }
}
